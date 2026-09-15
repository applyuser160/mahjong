use crate::expectation::{evaluate_hand_discards, evaluate_standing_hand, AnalysisContext};
use crate::hand::{Hand, Meld};
use crate::tile::TileName;
use rayon::prelude::*;

enum CallCandidateTask {
    Pass,
    Pon,
    Chii {
        idx1: usize,
        idx2: usize,
        meld: Meld,
        t1: TileName,
        t2: TileName,
    },
}

/// 鳴きのアクション種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallAction {
    /// スルー（鳴かない）
    Pass,
    /// チー（上家からの打牌で順子を形成）
    Chii(TileName, TileName),
    /// ポン（他家からの打牌で刻子を形成）
    Pon,
    /// カン（大明槓）
    Kan,
}

impl CallAction {
    pub fn label_ja(&self) -> String {
        match self {
            Self::Pass => "スルー (見逃し)".to_string(),
            Self::Chii(a, b) => format!("チー [{}-{}]", a.as_str(), b.as_str()),
            Self::Pon => "ポン".to_string(),
            Self::Kan => "カン (大明槓)".to_string(),
        }
    }
}

/// 鳴きの推奨度区分
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallRecommendation {
    /// 積極推奨（鳴く方が大幅に有利）
    AggressiveCall,
    /// 鳴き微有利（鳴いた方がやや有利）
    LeanCall,
    /// スルー微有利（門前維持がやや有利）
    LeanPass,
    /// スルー強く推奨（役なし・大幅打点低下など）
    StrictPass,
}

impl CallRecommendation {
    pub fn label_ja(&self) -> &'static str {
        match self {
            Self::AggressiveCall => "【積極推奨】鳴き推奨",
            Self::LeanCall => "【微有利】鳴き寄り",
            Self::LeanPass => "【微有利】スルー寄り",
            Self::StrictPass => "【厳禁/非推奨】スルー推奨",
        }
    }
}

/// 鳴きの選択肢評価
#[derive(Debug, Clone)]
pub struct CallChoice {
    pub action: CallAction,
    pub meld: Option<Meld>,
    pub post_shanten: i8,
    pub post_acceptance: usize,
    pub estimated_score: f64,
    pub ev: f64,
}

/// 副露（鳴き）判断推奨結果
#[derive(Debug, Clone)]
pub struct CallAdvice {
    pub target_tile: TileName,
    pub is_kamicha: bool,
    pub choices: Vec<CallChoice>,
    pub best_action: CallAction,
    pub recommendation: CallRecommendation,
    pub rationale: String,
}

/// 副露アドバイザー
pub struct CallAdvisor;

impl CallAdvisor {
    /// 他家の捨て牌に対して手牌から鳴き判断をアドバイスします。
    pub fn advise_call(
        hand: &Hand,
        target_tile: TileName,
        is_kamicha: bool,
        ctx: &AnalysisContext<'_>,
    ) -> Option<CallAdvice> {
        let mut tasks = Vec::with_capacity(5);
        tasks.push(CallCandidateTask::Pass);

        // 2. ポン判定（同種牌が2枚以上手牌にあるか）
        let target_idx = target_tile as usize;
        if hand.counts[target_idx] >= 2 {
            tasks.push(CallCandidateTask::Pon);
        }

        // 3. チー判定（上家かつ数牌の場合）
        if is_kamicha && target_idx < 28 {
            let rank = (target_idx - 1) % 9 + 1; // 1..=9

            // パターン A: [target - 2, target - 1] (例: 3 に対して 1, 2)
            if rank >= 3 && hand.counts[target_idx - 2] > 0 && hand.counts[target_idx - 1] > 0 {
                let t1 = TileName::from_usize(target_idx - 2);
                let t2 = TileName::from_usize(target_idx - 1);
                let meld = Meld::Chii {
                    called: target_tile,
                    consumed: [t1, t2],
                };
                tasks.push(CallCandidateTask::Chii {
                    idx1: target_idx - 2,
                    idx2: target_idx - 1,
                    meld,
                    t1,
                    t2,
                });
            }

            // パターン B: [target - 1, target + 1] (例: 3 に対して 2, 4)
            if (2..=8).contains(&rank)
                && hand.counts[target_idx - 1] > 0
                && hand.counts[target_idx + 1] > 0
            {
                let t1 = TileName::from_usize(target_idx - 1);
                let t2 = TileName::from_usize(target_idx + 1);
                let meld = Meld::Chii {
                    called: target_tile,
                    consumed: [t1, t2],
                };
                tasks.push(CallCandidateTask::Chii {
                    idx1: target_idx - 1,
                    idx2: target_idx + 1,
                    meld,
                    t1,
                    t2,
                });
            }

            // パターン C: [target + 1, target + 2] (例: 3 に対して 4, 5)
            if rank <= 7 && hand.counts[target_idx + 1] > 0 && hand.counts[target_idx + 2] > 0 {
                let t1 = TileName::from_usize(target_idx + 1);
                let t2 = TileName::from_usize(target_idx + 2);
                let meld = Meld::Chii {
                    called: target_tile,
                    consumed: [t1, t2],
                };
                tasks.push(CallCandidateTask::Chii {
                    idx1: target_idx + 1,
                    idx2: target_idx + 2,
                    meld,
                    t1,
                    t2,
                });
            }
        }

        // 鳴ける選択肢がスルー以外にない場合は None
        if tasks.len() <= 1 {
            return None;
        }

        // 各候補（スルー・ポン・チー）を並列評価
        let mut choices: Vec<CallChoice> = tasks
            .into_par_iter()
            .filter_map(|task| match task {
                CallCandidateTask::Pass => {
                    let standing_eval = evaluate_standing_hand(hand, None, ctx);
                    Some(CallChoice {
                        action: CallAction::Pass,
                        meld: None,
                        post_shanten: standing_eval.shanten,
                        post_acceptance: standing_eval.speed.remaining_count,
                        estimated_score: standing_eval.value.expected_score,
                        ev: standing_eval.ev,
                    })
                }
                CallCandidateTask::Pon => {
                    let mut post_hand = hand.clone();
                    post_hand.counts[target_idx] -= 2;
                    let meld = Meld::Pon(target_tile);
                    post_hand.open_melds.push(meld);

                    let post_evals = evaluate_hand_discards(&post_hand, None, ctx);
                    if !post_evals.is_empty() {
                        let best_post = &post_evals[0];
                        let is_yakuhai = matches!(
                            target_tile,
                            TileName::Red | TileName::Green | TileName::White
                        ) || ctx.seat_wind == Some(target_tile)
                            || ctx.round_wind == Some(target_tile);
                        let bonus = if is_yakuhai { 600.0 } else { 0.0 };

                        Some(CallChoice {
                            action: CallAction::Pon,
                            meld: Some(meld),
                            post_shanten: best_post.shanten_after,
                            post_acceptance: best_post.speed.remaining_count,
                            estimated_score: best_post.value.expected_score,
                            ev: best_post.ev + bonus,
                        })
                    } else {
                        None
                    }
                }
                CallCandidateTask::Chii {
                    idx1,
                    idx2,
                    meld,
                    t1,
                    t2,
                } => Self::eval_chii_choice(hand, idx1, idx2, meld, t1, t2, ctx),
            })
            .collect();

        // 最善アクションの決定（EV最大）
        choices.sort_by(|a, b| b.ev.partial_cmp(&a.ev).unwrap_or(std::cmp::Ordering::Equal));
        let best_choice = choices[0].clone();
        let pass_choice = choices
            .iter()
            .find(|c| c.action == CallAction::Pass)
            .unwrap();

        let ev_diff = best_choice.ev - pass_choice.ev;

        let (recommendation, rationale) = if best_choice.action == CallAction::Pass {
            let second_call = &choices[1];
            (
                CallRecommendation::StrictPass,
                format!(
                    "門前維持（スルー）が最善です。鳴くと門前役（立直・門前自摸）を失い、打点が大きく低下します（鳴きEV差: -{:.0}点）。",
                    second_call.ev - pass_choice.ev
                ),
            )
        } else if ev_diff >= 500.0 {
            (
                CallRecommendation::AggressiveCall,
                format!(
                    "{} が積極推奨です！向聴数が {} に進み、和了確率が大きく高まります（EV向上: +{:.0}点）。",
                    best_choice.action.label_ja(),
                    match best_choice.post_shanten {
                        0 => "テンパイ".to_string(),
                        s => format!("{s}向聴"),
                    },
                    ev_diff
                ),
            )
        } else if ev_diff > 0.0 {
            (
                CallRecommendation::LeanCall,
                format!(
                    "{} が微有利です。速度優先なら鳴き、高打点を狙うならスルーも選択肢です（EV差: +{:.0}点）。",
                    best_choice.action.label_ja(),
                    ev_diff
                ),
            )
        } else {
            (
                CallRecommendation::LeanPass,
                "スルー（門前維持）がやや有利です。鳴き後の有効牌や役の確定度合いを考慮し、門前でツモを見るのが無難です。".to_string(),
            )
        };

        Some(CallAdvice {
            target_tile,
            is_kamicha,
            choices,
            best_action: best_choice.action,
            recommendation,
            rationale,
        })
    }

    fn eval_chii_choice(
        hand: &Hand,
        idx1: usize,
        idx2: usize,
        meld: Meld,
        t1: TileName,
        t2: TileName,
        ctx: &AnalysisContext<'_>,
    ) -> Option<CallChoice> {
        let mut post_hand = hand.clone();
        post_hand.counts[idx1] -= 1;
        post_hand.counts[idx2] -= 1;
        post_hand.open_melds.push(meld);

        let evs = evaluate_hand_discards(&post_hand, None, ctx);
        if evs.is_empty() {
            return None;
        }

        let best = &evs[0];
        Some(CallChoice {
            action: CallAction::Chii(t1, t2),
            meld: Some(meld),
            post_shanten: best.shanten_after,
            post_acceptance: best.speed.remaining_count,
            estimated_score: best.value.expected_score,
            ev: best.ev,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_advisor_pon_yakuhai() {
        // 白対子持ちの一向聴手牌 (123m 456p 789s 白白 2s 8m)
        // 門前では一向聴だが、白ポンによって役牌確定テンパイへ前進する
        let mut hand = Hand::new();
        for &t in &[
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourM,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenP,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::FourS,
            TileName::FiveS,
            TileName::White,
            TileName::White,
            TileName::NineM,
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext::default();
        // 白が切られた時のアドバイス
        let advice = CallAdvisor::advise_call(&hand, TileName::White, false, &ctx);
        assert!(advice.is_some());

        let adv = advice.unwrap();
        // ポン候補が存在すること
        assert!(adv.choices.iter().any(|c| c.action == CallAction::Pon));
        // 白ポンは役牌確定で強く推奨されること
        assert_eq!(adv.best_action, CallAction::Pon);
        assert!(adv.rationale.contains("ポン"));
    }

    #[test]
    fn test_call_advisor_chii_kamicha() {
        // 2m3m を持っている手牌で上家から 4m が切られた場合
        let mut hand = Hand::new();
        for &t in &[
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::East,
            TileName::East,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::NineM,
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext::default();
        let advice = CallAdvisor::advise_call(&hand, TileName::FourM, true, &ctx);
        assert!(advice.is_some());

        let adv = advice.unwrap();
        assert!(adv
            .choices
            .iter()
            .any(|c| matches!(c.action, CallAction::Chii(..))));
    }

    #[test]
    fn test_call_advisor_otakaze_pon_rejected() {
        // 東場・東家で南（オタ風・役なし）の対子を持っている手牌
        // 123m 456p 789s 23s 南南 (13枚)
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourP,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::South, // オタ風対子
            TileName::South,
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext {
            seat_wind: Some(TileName::East),
            round_wind: Some(TileName::East),
            ..Default::default()
        };

        let advice = CallAdvisor::advise_call(&hand, TileName::South, false, &ctx);
        assert!(advice.is_some());

        let adv = advice.unwrap();
        // 南ポンは候補には挙がるが、役なしになるためスルー（Pass）が推奨される
        assert!(adv.choices.iter().any(|c| c.action == CallAction::Pon));
        assert_eq!(adv.best_action, CallAction::Pass);
    }
}
