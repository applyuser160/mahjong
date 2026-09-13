use crate::acceptance::calculate_acceptance;
use crate::dora::count_dora;
use crate::hand::Hand;
use crate::score::calculate_score;
use crate::tile::TileName;
use crate::yaku::{judge_yaku, WinContext, ALL_YAKU};

/// 速度指標
#[derive(Debug, Clone, PartialEq)]
pub struct SpeedMetric {
    pub accepted_tiles: Vec<TileName>,
    pub remaining_count: usize,
    pub win_probability: f64,
}

/// 打点指標
#[derive(Debug, Clone, PartialEq)]
pub struct ValueMetric {
    pub expected_score: f64,
    pub expected_han: f64,
    pub primary_yaku: Vec<&'static str>,
    pub has_high_value_potential: bool,
}

/// 安全指標
#[derive(Debug, Clone, PartialEq)]
pub struct SafetyMetric {
    pub risk_score: f64,
    pub is_safe: bool,
}

/// 打牌候補の総合評価
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateEvaluation {
    pub discard_tile: TileName,
    pub shanten_after: i8,
    pub ev: f64,
    pub speed: SpeedMetric,
    pub value: ValueMetric,
    pub safety: SafetyMetric,
}

/// 局の分析コンテキスト
#[derive(Debug, Clone, Copy)]
pub struct AnalysisContext {
    pub turn_number: usize,              // 現在の巡目 (1..=18)
    pub remaining_wall_tiles: usize,     // 山の残り枚数 (目安: 70 - 巡目*4)
    pub seat_wind: Option<TileName>,     // 自風
    pub round_wind: Option<TileName>,    // 場風
    pub dora_indicators: &'static [TileName], // ドラ表示牌
    pub is_dealer: bool,                 // 親かどうか
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self {
            turn_number: 6,
            remaining_wall_tiles: 50,
            seat_wind: Some(TileName::East),
            round_wind: Some(TileName::East),
            dora_indicators: &[],
            is_dealer: true,
        }
    }
}

/// 手牌（14枚）から全打牌候補を評価し、期待値の高い順にランキングします。
pub fn evaluate_hand_discards(
    hand: &Hand,
    visible_counts: Option<&[u8; 35]>,
    ctx: &AnalysisContext,
) -> Vec<CandidateEvaluation> {
    let open_melds_count = hand.open_melds.len();
    let mut working = hand.counts;
    let mut evaluations = Vec::new();

    let remaining_turns = (18usize.saturating_sub(ctx.turn_number)).max(1) as f64;
    let wall_remaining = (ctx.remaining_wall_tiles).max(1) as f64;

    for i in 1..=34 {
        if working[i] == 0 {
            continue;
        }

        let discard_tile = TileName::from_usize(i);
        working[i] -= 1;

        let acceptance = calculate_acceptance(&working, open_melds_count, visible_counts);
        let shanten_after = acceptance.current_shanten;

        // 1. 速度評価（和了確率）
        let win_probability = estimate_win_probability(
            shanten_after,
            acceptance.total_remaining,
            remaining_turns,
            wall_remaining,
        );

        let accepted_tiles: Vec<TileName> = acceptance.waits.iter().map(|w| w.tile).collect();
        let speed = SpeedMetric {
            accepted_tiles: accepted_tiles.clone(),
            remaining_count: acceptance.total_remaining,
            win_probability,
        };

        // 2. 打点評価（推定打点・主な役）
        let value = estimate_hand_value(&working, &accepted_tiles, open_melds_count, ctx);

        // 3. 安全度評価
        let safety = evaluate_tile_safety(discard_tile);

        // 4. 総合期待値 (EV)
        // 基本式: EV = 和了確率 * 想定打点 - 放銃リスク
        let ev = win_probability * value.expected_score - safety.risk_score * 800.0;

        evaluations.push(CandidateEvaluation {
            discard_tile,
            shanten_after,
            ev,
            speed,
            value,
            safety,
        });

        working[i] += 1;
    }

    // 期待値降順にソート
    evaluations.sort_by(|a, b| {
        b.ev.partial_cmp(&a.ev)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.shanten_after.cmp(&b.shanten_after))
            .then_with(|| b.speed.remaining_count.cmp(&a.speed.remaining_count))
    });

    evaluations
}

/// シャンテン数と受け入れ枚数から和了確率をモデル化
fn estimate_win_probability(
    shanten: i8,
    acceptance_remaining: usize,
    remaining_turns: f64,
    wall_remaining: f64,
) -> f64 {
    if shanten < 0 {
        return 1.0; // 既に和了
    }

    let acc = acceptance_remaining as f64;
    // 1巡あたりの有効牌ツモ確率
    let p_draw_per_turn = (acc / wall_remaining).min(0.9);

    // 残り巡目でのツモ確率: 1 - (1 - p)^R
    let p_advance = 1.0 - (1.0 - p_draw_per_turn).powf(remaining_turns);

    match shanten {
        0 => p_advance.clamp(0.05, 0.95), // テンパイ -> 和了
        1 => (p_advance * 0.45).clamp(0.02, 0.70), // 一向聴 -> テンパイ -> 和了
        2 => (p_advance * 0.18).clamp(0.01, 0.40), // 二向聴
        _ => (p_advance * 0.05).clamp(0.001, 0.15), // 三向聴以上
    }
}

/// 有効牌をツモった場合の想定打点と役を評価
fn estimate_hand_value(
    counts: &[u8; 35],
    accepted_tiles: &[TileName],
    open_melds_count: usize,
    ctx: &AnalysisContext,
) -> ValueMetric {
    if accepted_tiles.is_empty() {
        return ValueMetric {
            expected_score: 1000.0,
            expected_han: 1.0,
            primary_yaku: vec![],
            has_high_value_potential: false,
        };
    }

    let mut total_score = 0.0;
    let mut total_han = 0.0;
    let mut yaku_names = Vec::new();
    let sample_count = accepted_tiles.len().min(4);

    let mut working = *counts;

    for &tile in accepted_tiles.iter().take(sample_count) {
        let idx = tile as usize;
        working[idx] += 1;

        let win_ctx = WinContext {
            is_closed: open_melds_count == 0,
            is_tsumo: true,
            seat_wind: ctx.seat_wind,
            round_wind: ctx.round_wind,
            riichi: open_melds_count == 0, // 門前ならリーチ想定
            win_tile: Some(tile),
            ..Default::default()
        };

        let yaku_set = judge_yaku(&working, &[], win_ctx);

        let mut han: usize = 0;
        let mut is_yakuman = false;

        for y_info in ALL_YAKU {
            if yaku_set.contains(&y_info.id) {
                if y_info.yakuman {
                    is_yakuman = true;
                }
                han += y_info.han_closed.max(0) as usize;
                if !yaku_names.contains(&y_info.name_ja) {
                    yaku_names.push(y_info.name_ja);
                }
            }
        }

        // ドラ加算
        let dora_count = count_dora(&working, ctx.dora_indicators);
        han += dora_count;

        // 役がない場合でも門前ならリーチ役1翻を最低保証
        if han == 0 && open_melds_count == 0 {
            han = 1;
            if !yaku_names.contains(&"立直") {
                yaku_names.push("立直");
            }
        } else if han == 0 {
            han = 1;
        }

        let score_res = calculate_score(han, 30, ctx.is_dealer, true, is_yakuman);
        total_score += score_res.total_points as f64;
        total_han += han as f64;

        working[idx] -= 1;
    }

    let count_f = sample_count as f64;
    let expected_score = total_score / count_f;
    let expected_han = total_han / count_f;

    let has_high_value_potential = expected_score >= 5000.0;

    ValueMetric {
        expected_score,
        expected_han,
        primary_yaku: yaku_names,
        has_high_value_potential,
    }
}

/// 牌の安全度（孤立度・危険度）を評価
fn evaluate_tile_safety(tile: TileName) -> SafetyMetric {
    let idx = tile as usize;
    // 字牌 (28..=34)
    if idx >= 28 {
        return SafetyMetric {
            risk_score: 0.1,
            is_safe: true,
        };
    }

    let rank = (idx - 1) % 9 + 1;
    // 1, 9 牌
    if rank == 1 || rank == 9 {
        SafetyMetric {
            risk_score: 0.2,
            is_safe: true,
        }
    } else if rank == 2 || rank == 8 {
        SafetyMetric {
            risk_score: 0.4,
            is_safe: false,
        }
    } else {
        // 3〜7 中張牌
        SafetyMetric {
            risk_score: 0.6,
            is_safe: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::TileName;

    #[test]
    fn test_evaluate_hand_discards() {
        // 1m2m3m 4p5p6p 7s8s9s 東東 2s3s + 9p (余剰牌 9p を切れば両面テンパイで最高EV)
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
            TileName::East,
            TileName::East,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::NineP, // 余剰牌
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext::default();
        let evs = evaluate_hand_discards(&hand, None, &ctx);
        assert!(!evs.is_empty());

        let best = &evs[0];
        assert_eq!(best.discard_tile, TileName::NineP);
        assert_eq!(best.shanten_after, 0); // テンパイ
        assert!(best.ev > 2000.0);
    }
}
