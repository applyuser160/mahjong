use crate::expectation::CandidateEvaluation;
use crate::tile::TileName;

/// 悪手・疑問手の深刻度区分
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlunderSeverity {
    /// 疑問手: 軽微な期待値損失 (150点 〜 400点未満)
    Inaccuracy,
    /// 悪手: 明確な期待値損失 (400点 〜 1000点未満)
    Mistake,
    /// 大悪手: 致命的な期待値損失 (1000点以上)
    Blunder,
}

impl BlunderSeverity {
    pub fn label_ja(&self) -> &'static str {
        match self {
            Self::Inaccuracy => "疑問手",
            Self::Mistake => "悪手",
            Self::Blunder => "大悪手",
        }
    }
}

/// 毎巡の手番打牌決定記録
#[derive(Debug, Clone)]
pub struct TurnDecisionRecord {
    pub turn: usize,
    pub chosen_tile: TileName,
    pub chosen_ev: f64,
    pub best_tile: TileName,
    pub best_ev: f64,
    pub ev_loss: f64,
    pub is_optimal: bool,
    pub candidates: Vec<CandidateEvaluation>,
}

/// 悪手・疑問手の記録
#[derive(Debug, Clone)]
pub struct BlunderRecord {
    pub turn: usize,
    pub chosen_tile: TileName,
    pub chosen_ev: f64,
    pub best_tile: TileName,
    pub best_ev: f64,
    pub ev_loss: f64,
    pub severity: BlunderSeverity,
    pub explanation: String,
}

/// 対局全体の学習振り返りレポート
#[derive(Debug, Clone)]
pub struct MatchReviewReport {
    pub total_turns: usize,
    pub optimal_picks_count: usize,
    pub accuracy_rate: f64,
    pub total_ev_loss: f64,
    pub average_ev_loss: f64,
    pub blunders: Vec<BlunderRecord>,
}

/// 対局中の打牌意思決定を記録・分析するトラッカー
#[derive(Debug, Default, Clone)]
pub struct ReviewTracker {
    records: Vec<TurnDecisionRecord>,
}

impl ReviewTracker {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// 各巡のユーザー打牌とその時点の候補評価一覧を記録
    pub fn record_decision(
        &mut self,
        turn: usize,
        chosen_tile: TileName,
        candidates: &[CandidateEvaluation],
    ) {
        if candidates.is_empty() {
            return;
        }

        let best = &candidates[0];
        let chosen_eval = candidates.iter().find(|c| c.discard_tile == chosen_tile);

        let (chosen_ev, is_optimal) = match chosen_eval {
            Some(c) => (c.ev, c.discard_tile == best.discard_tile),
            None => (0.0, false),
        };

        let ev_loss = (best.ev - chosen_ev).max(0.0);

        self.records.push(TurnDecisionRecord {
            turn,
            chosen_tile,
            chosen_ev,
            best_tile: best.discard_tile,
            best_ev: best.ev,
            ev_loss,
            is_optimal,
            candidates: candidates.to_vec(),
        });
    }

    /// 記録から終局学習レポートを生成
    pub fn generate_report(&self) -> MatchReviewReport {
        let total_turns = self.records.len();
        if total_turns == 0 {
            return MatchReviewReport {
                total_turns: 0,
                optimal_picks_count: 0,
                accuracy_rate: 0.0,
                total_ev_loss: 0.0,
                average_ev_loss: 0.0,
                blunders: Vec::new(),
            };
        }

        let mut optimal_picks_count = 0;
        let mut total_ev_loss = 0.0;
        let mut blunders = Vec::new();

        for rec in &self.records {
            if rec.is_optimal {
                optimal_picks_count += 1;
            }
            total_ev_loss += rec.ev_loss;

            // 損失 150点以上を疑問手・悪手として抽出
            if rec.ev_loss >= 150.0 {
                let severity = if rec.ev_loss >= 1000.0 {
                    BlunderSeverity::Blunder
                } else if rec.ev_loss >= 400.0 {
                    BlunderSeverity::Mistake
                } else {
                    BlunderSeverity::Inaccuracy
                };

                let explanation = Self::diagnose_loss_reason(rec);

                blunders.push(BlunderRecord {
                    turn: rec.turn,
                    chosen_tile: rec.chosen_tile,
                    chosen_ev: rec.chosen_ev,
                    best_tile: rec.best_tile,
                    best_ev: rec.best_ev,
                    ev_loss: rec.ev_loss,
                    severity,
                    explanation,
                });
            }
        }

        // EV損失が大きい順にソート
        blunders.sort_by(|a, b| {
            b.ev_loss
                .partial_cmp(&a.ev_loss)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let accuracy_rate = optimal_picks_count as f64 / total_turns as f64;
        let average_ev_loss = total_ev_loss / total_turns as f64;

        MatchReviewReport {
            total_turns,
            optimal_picks_count,
            accuracy_rate,
            total_ev_loss,
            average_ev_loss,
            blunders,
        }
    }

    /// 最善手と選択手の要因差から、なぜ損失が発生したかを診断
    fn diagnose_loss_reason(rec: &TurnDecisionRecord) -> String {
        let best = &rec.candidates[0];
        let chosen = rec
            .candidates
            .iter()
            .find(|c| c.discard_tile == rec.chosen_tile);

        let Some(chosen) = chosen else {
            return format!(
                "推奨打牌 [{}] に対し無効な打牌を選択しました",
                best.discard_tile.as_str()
            );
        };

        let mut reasons = Vec::new();

        // 1. シャンテン数の差
        if chosen.shanten_after > best.shanten_after {
            reasons.push(format!(
                "推奨打牌 [{}] は {} ですが、選択した打牌 [{}] は {} に後退します",
                best.discard_tile.as_str(),
                match best.shanten_after {
                    0 => "テンパイ".to_string(),
                    s => format!("{s}向聴"),
                },
                chosen.discard_tile.as_str(),
                match chosen.shanten_after {
                    0 => "テンパイ".to_string(),
                    s => format!("{s}向聴"),
                }
            ));
        }

        // 2. 受け入れ枚数の差
        let rem_diff = best.speed.remaining_count as isize - chosen.speed.remaining_count as isize;
        if rem_diff >= 4 {
            reasons.push(format!(
                "受け入れ枚数が {} 枚減少します（推奨: {}枚 vs 選択: {}枚）",
                rem_diff, best.speed.remaining_count, chosen.speed.remaining_count
            ));
        }

        // 3. 想定打点の差
        let score_diff = best.value.expected_score - chosen.value.expected_score;
        if score_diff >= 1500.0 {
            let best_yaku_str = if !best.value.primary_yaku.is_empty() {
                format!("（{}）", best.value.primary_yaku.join("・"))
            } else {
                String::new()
            };
            reasons.push(format!(
                "想定打点が約 {:.0} 点低下します（推奨 {}: {:.0}点 vs 選択: {:.0}点）",
                score_diff, best_yaku_str, best.value.expected_score, chosen.value.expected_score
            ));
        }

        // 4. 危険度の差（守備面）
        let risk_diff = chosen.safety.risk_score - best.safety.risk_score;
        if risk_diff >= 0.2 {
            reasons.push(format!(
                "放銃危険度が上昇します（推奨: {:.0}% vs 選択: {:.0}%）",
                best.safety.risk_score * 100.0,
                chosen.safety.risk_score * 100.0
            ));
        }

        if reasons.is_empty() {
            format!(
                "推奨打牌 [{}] の方が速度・打点・安全度のバランスで勝っています（EV差: +{:.0}）",
                best.discard_tile.as_str(),
                rec.ev_loss
            )
        } else {
            reasons.join("。")
        }
    }

    /// ターミナル表示用のアスキーレポート整形
    pub fn format_report(&self, report: &MatchReviewReport) -> String {
        let mut out = String::new();
        out.push_str(
            "\n================================================================================\n",
        );
        out.push_str(
            "                   📊 局後学習振り返りレポート (Match Review)                   \n",
        );
        out.push_str(
            "================================================================================\n",
        );

        out.push_str(&format!(
            "  総打牌数:           {} 巡\n",
            report.total_turns
        ));
        out.push_str(&format!(
            "  AI最善手一致率:     {:.1}% ({} / {})\n",
            report.accuracy_rate * 100.0,
            report.optimal_picks_count,
            report.total_turns
        ));
        out.push_str(&format!(
            "  総EV損失 (ロス):    {:.0} 点\n",
            report.total_ev_loss
        ));
        out.push_str(&format!(
            "  平均EV損失 (1打):   {:.1} 点/巡\n",
            report.average_ev_loss
        ));

        // 評価ランクの算出
        let grade = if report.accuracy_rate >= 0.90 && report.average_ev_loss <= 100.0 {
            "S (牌効率マスター・プロ級)"
        } else if report.accuracy_rate >= 0.75 && report.average_ev_loss <= 250.0 {
            "A (高精度・上級者レベル)"
        } else if report.accuracy_rate >= 0.60 && report.average_ev_loss <= 450.0 {
            "B (基本形習得・中級者レベル)"
        } else {
            "C (要改善・見落とし多め)"
        };
        out.push_str(&format!("  総合プレイ評価:     {}\n", grade));
        out.push_str(
            "--------------------------------------------------------------------------------\n",
        );

        if report.blunders.is_empty() {
            out.push_str("🎉 素晴らしい！大きな悪手や疑問手は検出されませんでした。\n");
        } else {
            out.push_str(&format!(
                "⚠️  改善点・悪手診断 (ワースト {} 件):\n\n",
                report.blunders.len().min(5)
            ));

            for (i, b) in report.blunders.iter().take(5).enumerate() {
                out.push_str(&format!(
                    "【第{}位: {}】 巡目: {}巡目  (EV損失: -{:.0}点)\n",
                    i + 1,
                    b.severity.label_ja(),
                    b.turn,
                    b.ev_loss
                ));
                out.push_str(&format!(
                    "  選択打牌: [{}] (EV: {:.0})  -->  推奨打牌: [{}] (EV: {:.0})\n",
                    b.chosen_tile.as_str(),
                    b.chosen_ev,
                    b.best_tile.as_str(),
                    b.best_ev
                ));
                out.push_str(&format!("  理由分析: {}\n\n", b.explanation));
            }
        }

        out.push_str(
            "================================================================================\n",
        );
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expectation::{SafetyMetric, SpeedMetric, ValueMetric};

    fn make_dummy_candidates() -> Vec<CandidateEvaluation> {
        vec![
            CandidateEvaluation {
                discard_tile: TileName::OneM,
                shanten_after: 0,
                ev: 3500.0,
                speed: SpeedMetric {
                    accepted_tiles: vec![TileName::TwoM, TileName::FiveM],
                    remaining_count: 8,
                    win_probability: 0.7,
                },
                value: ValueMetric {
                    expected_score: 5000.0,
                    expected_han: 3.0,
                    primary_yaku: vec!["立直", "ピンフ"],
                    has_high_value_potential: true,
                },
                safety: SafetyMetric {
                    risk_score: 0.0,
                    is_safe: true,
                },
            },
            CandidateEvaluation {
                discard_tile: TileName::NineP,
                shanten_after: 1,
                ev: 2200.0,
                speed: SpeedMetric {
                    accepted_tiles: vec![TileName::ThreeM],
                    remaining_count: 4,
                    win_probability: 0.4,
                },
                value: ValueMetric {
                    expected_score: 3000.0,
                    expected_han: 2.0,
                    primary_yaku: vec!["立直"],
                    has_high_value_potential: false,
                },
                safety: SafetyMetric {
                    risk_score: 0.1,
                    is_safe: false,
                },
            },
        ]
    }

    #[test]
    fn test_review_tracker_optimal_decision() {
        let mut tracker = ReviewTracker::new();
        let candidates = make_dummy_candidates();

        // 1位の OneM を選択
        tracker.record_decision(1, TileName::OneM, &candidates);
        let report = tracker.generate_report();

        assert_eq!(report.total_turns, 1);
        assert_eq!(report.optimal_picks_count, 1);
        assert_eq!(report.accuracy_rate, 1.0);
        assert_eq!(report.total_ev_loss, 0.0);
        assert!(report.blunders.is_empty());
    }

    #[test]
    fn test_review_tracker_blunder_detection() {
        let mut tracker = ReviewTracker::new();
        let candidates = make_dummy_candidates();

        // 2位の NineP を選択 (EV差: 3500 - 2200 = 1300点 -> Blunder)
        tracker.record_decision(1, TileName::NineP, &candidates);
        let report = tracker.generate_report();

        assert_eq!(report.total_turns, 1);
        assert_eq!(report.optimal_picks_count, 0);
        assert_eq!(report.accuracy_rate, 0.0);
        assert_eq!(report.total_ev_loss, 1300.0);
        assert_eq!(report.blunders.len(), 1);

        let b = &report.blunders[0];
        assert_eq!(b.severity, BlunderSeverity::Blunder);
        assert_eq!(b.chosen_tile, TileName::NineP);
        assert_eq!(b.best_tile, TileName::OneM);
        assert!(b.explanation.contains("テンパイ"));
        assert!(b.explanation.contains("1向聴"));

        let output = tracker.format_report(&report);
        assert!(output.contains("局後学習振り返りレポート"));
        assert!(output.contains("大悪手"));
    }
}
