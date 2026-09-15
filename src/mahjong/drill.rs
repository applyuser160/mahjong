use rand::Rng;

use crate::expectation::{evaluate_hand_discards, AnalysisContext, CandidateEvaluation};
use crate::explanation::Explainer;
use crate::hand::Hand;
use crate::shanten::calculate_shanten;
use crate::tile::TileName;
use crate::wall::Wall;

/// 何切る問題
#[derive(Debug, Clone)]
pub struct DrillProblem {
    pub hand: Hand,
    pub dora_indicator: TileName,
    pub turn_number: usize,
    pub candidates: Vec<CandidateEvaluation>,
    pub best_tile: TileName,
    pub rationale: String,
}

/// 何切る解答結果
#[derive(Debug, Clone)]
pub struct DrillAnswerResult {
    pub is_correct: bool,
    pub chosen_tile: TileName,
    pub chosen_ev: f64,
    pub best_tile: TileName,
    pub best_ev: f64,
    pub ev_loss: f64,
    pub feedback: String,
}

/// 何切るセッション成績
#[derive(Debug, Clone)]
pub struct DrillSessionReport {
    pub total_problems: usize,
    pub correct_count: usize,
    pub accuracy_rate: f64,
    pub total_ev_loss: f64,
    pub average_ev_loss: f64,
    pub results: Vec<DrillAnswerResult>,
}

/// 何切るドリルエンジン
pub struct DrillEngine;

impl DrillEngine {
    /// 指定された向聴数（または任意）の14枚手牌から何切る問題を生成します。
    pub fn generate_problem<R: Rng>(
        target_shanten: Option<i8>,
        max_attempts: usize,
        rng: &mut R,
    ) -> Option<DrillProblem> {
        for _ in 0..max_attempts {
            let (hand, dora_indicator) = match target_shanten {
                Some(0) => Self::generate_tenpai_hand(rng),
                Some(1) => Self::generate_one_shanten_hand(rng),
                _ => Self::generate_random_hand(rng),
            };

            // 14枚手牌の向聴数チェック
            let current_shanten = calculate_shanten(&hand);
            if current_shanten.min_shanten < 0 {
                continue;
            }

            // 打牌後の向聴数が target_shanten に合致するかを判定
            let ctx = AnalysisContext {
                turn_number: rng.gen_range(3..=10),
                remaining_wall_tiles: rng.gen_range(30..=65),
                seat_wind: Some(TileName::East),
                round_wind: Some(TileName::East),
                dora_indicators: &[dora_indicator],
                is_dealer: true,
                ..Default::default()
            };

            let candidates = evaluate_hand_discards(&hand, None, &ctx);
            if candidates.is_empty() {
                continue;
            }

            let best = &candidates[0];

            if let Some(expected_shanten) = target_shanten {
                if best.shanten_after != expected_shanten {
                    continue;
                }
            }

            let best_tile = best.discard_tile;
            let rationale = Explainer::generate_rationale(&candidates);

            return Some(DrillProblem {
                hand,
                dora_indicator,
                turn_number: ctx.turn_number,
                candidates,
                best_tile,
                rationale,
            });
        }
        None
    }

    fn generate_tenpai_hand<R: Rng>(rng: &mut R) -> (Hand, TileName) {
        let mut counts = [0u8; 35];
        let mut hand = Hand::new();

        // 3面子（順子）をランダムな色で作る
        let suits = [0usize, 9, 18];
        for _ in 0..3 {
            let base = suits[rng.gen_range(0..suits.len())];
            let r = rng.gen_range(1..=7);
            for offset in 0..3 {
                let idx = base + r + offset;
                if counts[idx] < 4 {
                    counts[idx] += 1;
                    hand.push(TileName::from_usize(idx));
                }
            }
        }

        // 雀頭（対子）を1つ
        let head_idx = rng.gen_range(1..=34);
        for _ in 0..2 {
            if counts[head_idx] < 4 {
                counts[head_idx] += 1;
                hand.push(TileName::from_usize(head_idx));
            }
        }

        // 塔子（両面）を1つ
        let base = suits[rng.gen_range(0..suits.len())];
        let r = rng.gen_range(2..=7);
        for offset in 0..2 {
            let idx = base + r + offset;
            if counts[idx] < 4 {
                counts[idx] += 1;
                hand.push(TileName::from_usize(idx));
            }
        }

        // 余剰牌（孤立牌）を14枚になるまで補充
        while hand.tiles().len() < 14 {
            let rand_tile = rng.gen_range(1..=34);
            if counts[rand_tile] < 4 {
                counts[rand_tile] += 1;
                hand.push(TileName::from_usize(rand_tile));
            }
        }

        let dora = TileName::from_usize(rng.gen_range(1..=34));
        (hand, dora)
    }

    fn generate_one_shanten_hand<R: Rng>(rng: &mut R) -> (Hand, TileName) {
        let mut counts = [0u8; 35];
        let mut hand = Hand::new();

        // 2面子
        let suits = [0usize, 9, 18];
        for _ in 0..2 {
            let base = suits[rng.gen_range(0..suits.len())];
            let r = rng.gen_range(1..=7);
            for offset in 0..3 {
                let idx = base + r + offset;
                if counts[idx] < 4 {
                    counts[idx] += 1;
                    hand.push(TileName::from_usize(idx));
                }
            }
        }

        // 雀頭（対子）
        let head_idx = rng.gen_range(1..=34);
        for _ in 0..2 {
            if counts[head_idx] < 4 {
                counts[head_idx] += 1;
                hand.push(TileName::from_usize(head_idx));
            }
        }

        // 2塔子
        for _ in 0..2 {
            let base = suits[rng.gen_range(0..suits.len())];
            let r = rng.gen_range(2..=7);
            for offset in 0..2 {
                let idx = base + r + offset;
                if counts[idx] < 4 {
                    counts[idx] += 1;
                    hand.push(TileName::from_usize(idx));
                }
            }
        }

        // 余剰牌を14枚になるまで補充
        while hand.tiles().len() < 14 {
            let rand_tile = rng.gen_range(1..=34);
            if counts[rand_tile] < 4 {
                counts[rand_tile] += 1;
                hand.push(TileName::from_usize(rand_tile));
            }
        }

        let dora = TileName::from_usize(rng.gen_range(1..=34));
        (hand, dora)
    }

    fn generate_random_hand<R: Rng>(rng: &mut R) -> (Hand, TileName) {
        let mut wall = Wall::new();
        wall.shuffle(rng);
        let dora_indicator = wall.tiles()[135];
        let mut hand = Hand::new();
        for _ in 0..14 {
            if let Some(t) = wall.draw() {
                hand.push(t);
            }
        }
        (hand, dora_indicator)
    }

    /// 解答の採点と要因フィードバックの生成
    pub fn evaluate_answer(problem: &DrillProblem, chosen_tile: TileName) -> DrillAnswerResult {
        let best = &problem.candidates[0];
        let chosen_eval = problem
            .candidates
            .iter()
            .find(|c| c.discard_tile == chosen_tile);

        let (chosen_ev, is_correct) = match chosen_eval {
            Some(c) => (c.ev, c.discard_tile == best.discard_tile),
            None => (0.0, false),
        };

        let ev_loss = (best.ev - chosen_ev).max(0.0);

        let feedback = if is_correct {
            format!(
                "🎉 【正解！】AI推奨の最善打牌 [{}] です。\n  根拠: {}",
                best.discard_tile.as_str(),
                problem.rationale
            )
        } else if let Some(chosen) = chosen_eval {
            let mut diff_reasons = Vec::new();

            if chosen.shanten_after > best.shanten_after {
                diff_reasons.push(format!(
                    "向聴数が後退します（最善 [{}] は {}向聴 vs 選択 [{}] は {}向聴）",
                    best.discard_tile.as_str(),
                    best.shanten_after,
                    chosen.discard_tile.as_str(),
                    chosen.shanten_after
                ));
            }

            let rem_diff =
                best.speed.remaining_count as isize - chosen.speed.remaining_count as isize;
            if rem_diff > 0 {
                diff_reasons.push(format!(
                    "受け入れ枚数が {} 枚少なくなります（最善: {}枚 vs 選択: {}枚）",
                    rem_diff, best.speed.remaining_count, chosen.speed.remaining_count
                ));
            }

            let score_diff = best.value.expected_score - chosen.value.expected_score;
            if score_diff >= 1000.0 {
                diff_reasons.push(format!(
                    "想定打点が約 {:.0} 点低下します（最善: {:.0}点 vs 選択: {:.0}点）",
                    score_diff, best.value.expected_score, chosen.value.expected_score
                ));
            }

            let reason_text = if diff_reasons.is_empty() {
                format!(
                    "速度・打点のバランスで [{}] が優れています",
                    best.discard_tile.as_str()
                )
            } else {
                diff_reasons.join("。")
            };

            format!(
                "❌ 【不正解】最善打牌は [{}] です。(EV損失: -{:.0}点)\n  理由: {}\n  AI判断: {}",
                best.discard_tile.as_str(),
                ev_loss,
                reason_text,
                problem.rationale
            )
        } else {
            format!(
                "❌ 【無効】手牌にない牌 [{}] を選択しました。最善打牌は [{}] です。",
                chosen_tile.as_str(),
                best.discard_tile.as_str()
            )
        };

        DrillAnswerResult {
            is_correct,
            chosen_tile,
            chosen_ev,
            best_tile: best.discard_tile,
            best_ev: best.ev,
            ev_loss,
            feedback,
        }
    }
}

/// 何切るドリルセッションマネージャ
#[derive(Debug, Default, Clone)]
pub struct DrillSession {
    results: Vec<DrillAnswerResult>,
}

impl DrillSession {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn record_result(&mut self, res: DrillAnswerResult) {
        self.results.push(res);
    }

    pub fn generate_report(&self) -> DrillSessionReport {
        let total_problems = self.results.len();
        if total_problems == 0 {
            return DrillSessionReport {
                total_problems: 0,
                correct_count: 0,
                accuracy_rate: 0.0,
                total_ev_loss: 0.0,
                average_ev_loss: 0.0,
                results: Vec::new(),
            };
        }

        let mut correct_count = 0;
        let mut total_ev_loss = 0.0;

        for r in &self.results {
            if r.is_correct {
                correct_count += 1;
            }
            total_ev_loss += r.ev_loss;
        }

        let accuracy_rate = correct_count as f64 / total_problems as f64;
        let average_ev_loss = total_ev_loss / total_problems as f64;

        DrillSessionReport {
            total_problems,
            correct_count,
            accuracy_rate,
            total_ev_loss,
            average_ev_loss,
            results: self.results.clone(),
        }
    }

    pub fn format_report(&self, report: &DrillSessionReport) -> String {
        let mut out = String::new();
        out.push_str(
            "\n================================================================================\n",
        );
        out.push_str(
            "                  🎯 何切るドリル 成績レポート (Drill Summary)                  \n",
        );
        out.push_str(
            "================================================================================\n",
        );
        out.push_str(&format!("  総問題数:       {} 問\n", report.total_problems));
        out.push_str(&format!(
            "  正答数:         {} 問 (正答率: {:.1}%)\n",
            report.correct_count,
            report.accuracy_rate * 100.0
        ));
        out.push_str(&format!(
            "  総EV損失:       {:.0} 点\n",
            report.total_ev_loss
        ));
        out.push_str(&format!(
            "  1問平均EV損失:  {:.1} 点/問\n",
            report.average_ev_loss
        ));

        let grade = if report.accuracy_rate >= 0.90 && report.average_ev_loss <= 100.0 {
            "S (雀聖・達人級：抜群の牌効率)"
        } else if report.accuracy_rate >= 0.70 && report.average_ev_loss <= 300.0 {
            "A (上級者：安定した構想力)"
        } else if report.accuracy_rate >= 0.50 && report.average_ev_loss <= 600.0 {
            "B (中級者：基本手役と受入を再確認)"
        } else {
            "C (初級者：枚数最大を意識して復習)"
        };
        out.push_str(&format!("  牌効率総合判定: {}\n", grade));
        out.push_str(
            "================================================================================\n",
        );
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_drill_problem_generation() {
        let mut rng = SmallRng::seed_from_u64(42);

        // テンパイまたは1向聴の問題が生成できること
        let problem = DrillEngine::generate_problem(Some(0), 100, &mut rng);
        assert!(problem.is_some(), "Should generate a tenpai drill problem");

        let p = problem.unwrap();
        assert_eq!(p.hand.tiles().len(), 14);
        assert!(!p.candidates.is_empty());
        assert_eq!(p.candidates[0].shanten_after, 0);

        // 正解の採点
        let res_correct = DrillEngine::evaluate_answer(&p, p.best_tile);
        assert!(res_correct.is_correct);
        assert_eq!(res_correct.ev_loss, 0.0);
        assert!(res_correct.feedback.contains("正解"));

        // 不正解の採点（最善手以外の候補を選択）
        if p.candidates.len() >= 2 {
            let second_tile = p.candidates[1].discard_tile;
            let res_wrong = DrillEngine::evaluate_answer(&p, second_tile);
            assert!(!res_wrong.is_correct);
            assert!(res_wrong.feedback.contains("不正解"));
        }
    }

    #[test]
    fn test_drill_session_reporting() {
        let mut session = DrillSession::new();
        session.record_result(DrillAnswerResult {
            is_correct: true,
            chosen_tile: TileName::OneM,
            chosen_ev: 3000.0,
            best_tile: TileName::OneM,
            best_ev: 3000.0,
            ev_loss: 0.0,
            feedback: "正解".to_string(),
        });
        session.record_result(DrillAnswerResult {
            is_correct: false,
            chosen_tile: TileName::NineP,
            chosen_ev: 2200.0,
            best_tile: TileName::OneM,
            best_ev: 3000.0,
            ev_loss: 800.0,
            feedback: "不正解".to_string(),
        });

        let rep = session.generate_report();
        assert_eq!(rep.total_problems, 2);
        assert_eq!(rep.correct_count, 1);
        assert_eq!(rep.accuracy_rate, 0.5);
        assert_eq!(rep.total_ev_loss, 800.0);
        assert_eq!(rep.average_ev_loss, 400.0);

        let text = session.format_report(&rep);
        assert!(text.contains("成績レポート"));
    }
}
