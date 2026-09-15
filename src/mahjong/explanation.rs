use crate::expectation::CandidateEvaluation;

/// 要因分析モデルに基づく解説生成器
pub struct Explainer;

impl Explainer {
    /// 打牌評価ランキングから「なぜ1位が最善なのか」の意思決定理由（Rationale）を日本語で生成します。
    pub fn generate_rationale(candidates: &[CandidateEvaluation]) -> String {
        if candidates.is_empty() {
            return "打牌候補がありません。".to_string();
        }

        let best = &candidates[0];

        if candidates.len() == 1 {
            return format!(
                "選択肢は [{}] のみです（打牌後向聴数: {}）。",
                best.discard_tile.as_str(),
                best.shanten_after
            );
        }

        let second = &candidates[1];

        let best_tile = best.discard_tile.as_str();
        let second_tile = second.discard_tile.as_str();

        let shanten_diff = second.shanten_after - best.shanten_after;
        let count_diff =
            best.speed.remaining_count as isize - second.speed.remaining_count as isize;
        let score_diff = best.value.expected_score - second.value.expected_score;

        let best_yaku_str = if !best.value.primary_yaku.is_empty() {
            best.value.primary_yaku.join("・")
        } else {
            "役なし".to_string()
        };

        // パターン1: シャンテン数自体が進む場合
        if shanten_diff > 0 {
            return format!(
                "[{best_tile}] を切ることで向聴数が最も早く進みます（[{best_tile}]切り: {}向聴 vs [{second_tile}]切り: {}向聴）。速度最優先で [{best_tile}] 切りが圧倒的に優位です。",
                best.shanten_after,
                second.shanten_after
            );
        }

        // パターン2: 枚数が多く、かつ打点も同等以上の場合（完全優位）
        if count_diff >= 0 && score_diff >= -200.0 {
            return format!(
                "[{best_tile}] 切りが受け入れ枚数最大（{}種 {}枚）かつ想定打点（{:.0}点: {best_yaku_str}）ともに [{second_tile}] 切り（{}枚 / {:.0}点）を上回っており、迷わず [{best_tile}] 切りが最善手です。",
                best.speed.accepted_tiles.len(),
                best.speed.remaining_count,
                best.value.expected_score,
                second.speed.remaining_count,
                second.value.expected_score
            );
        }

        // パターン3: 枚数は2位の方が多いが、打点（役・ドラ）の高さでEV逆転している場合（打点重視の判断）
        if count_diff < 0 && score_diff > 500.0 {
            let abs_count_diff = count_diff.abs();
            return format!(
                "[{second_tile}] 切りの方が受け入れ枚数は {abs_count_diff}枚多い（{}枚 vs {}枚）ですが、[{best_tile}] 切りにすることで [{best_yaku_str}] による打点上昇（想定打点: {:.0}点 vs {:.0}点、約 +{:.0}点差）が期待でき、総合期待値で [{best_tile}] が上回ります。",
                second.speed.remaining_count,
                best.speed.remaining_count,
                best.value.expected_score,
                second.value.expected_score,
                score_diff
            );
        }

        // パターン4: 安全度やその他の要因で選ばれた場合
        format!(
            "[{best_tile}] 切り（EV: {:.0}点, 受入: {}枚, 想定打点: {:.0}点）が [{second_tile}] 切り（EV: {:.0}点, 受入: {}枚）に対して総合的なバランス（速度・打点・守備力）で最も優れています。",
            best.ev,
            best.speed.remaining_count,
            best.value.expected_score,
            second.ev,
            second.speed.remaining_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expectation::{CandidateEvaluation, SafetyMetric, SpeedMetric, ValueMetric};
    use crate::tile::TileName;
    use arrayvec::ArrayVec;

    #[test]
    fn test_generate_rationale_speed_dominant() {
        let candidates = vec![
            CandidateEvaluation {
                discard_tile: TileName::NineP,
                shanten_after: 0,
                ev: 3500.0,
                speed: SpeedMetric {
                    accepted_tiles: ArrayVec::from_iter([TileName::OneS, TileName::FourS]),
                    remaining_count: 8,
                    win_probability: 0.6,
                },
                value: ValueMetric {
                    expected_score: 5800.0,
                    expected_han: 3.0,
                    primary_yaku: ArrayVec::from_iter(["立直", "平和"]),
                    has_high_value_potential: true,
                },
                safety: SafetyMetric {
                    risk_score: 0.2,
                    is_safe: true,
                },
            },
            CandidateEvaluation {
                discard_tile: TileName::East,
                shanten_after: 0,
                ev: 2000.0,
                speed: SpeedMetric {
                    accepted_tiles: ArrayVec::from_iter([TileName::OneS]),
                    remaining_count: 4,
                    win_probability: 0.35,
                },
                value: ValueMetric {
                    expected_score: 5800.0,
                    expected_han: 3.0,
                    primary_yaku: ArrayVec::from_iter(["立直", "平和"]),
                    has_high_value_potential: true,
                },
                safety: SafetyMetric {
                    risk_score: 0.1,
                    is_safe: true,
                },
            },
        ];

        let rationale = Explainer::generate_rationale(&candidates);
        assert!(rationale.contains("迷わず [9p] 切りが最善手です"));
    }
}
