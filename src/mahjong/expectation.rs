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
pub struct AnalysisContext<'a> {
    pub turn_number: usize,              // 現在の巡目 (1..=18)
    pub remaining_wall_tiles: usize,     // 山の残り枚数 (目安: 70 - 巡目*4)
    pub seat_wind: Option<TileName>,     // 自風
    pub round_wind: Option<TileName>,    // 場風
    pub dora_indicators: &'a [TileName], // ドラ表示牌
    pub is_dealer: bool,                 // 親かどうか
}

impl Default for AnalysisContext<'static> {
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
    ctx: &AnalysisContext<'_>,
) -> Vec<CandidateEvaluation> {
    let open_melds_count = hand.open_melds.len();
    let mut working = hand.counts;
    let mut evaluations = Vec::new();

    // 候補評価では、打牌前の14枚の手牌（および外部指定の可視牌）をすべて可視牌として含める
    let mut base_visible = [0u8; 35];
    if let Some(v) = visible_counts {
        for i in 1..=34 {
            base_visible[i] = v[i].max(hand.counts[i]);
        }
    } else {
        base_visible = hand.counts;
    }

    let remaining_turns = (18usize.saturating_sub(ctx.turn_number)).max(1) as f64;
    let wall_remaining = (ctx.remaining_wall_tiles).max(1) as f64;

    for i in 1..=34 {
        if working[i] == 0 {
            continue;
        }

        let discard_tile = TileName::from_usize(i);
        working[i] -= 1;

        // 打牌した牌を含む base_visible を可視牌として渡す
        let acceptance = calculate_acceptance(&working, open_melds_count, Some(&base_visible));
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

        // 2. 打点評価（テンパイ・1向聴・2向聴以上の遷移を正しくモデル化）
        let value = estimate_hand_value(
            &working,
            shanten_after,
            &accepted_tiles,
            &hand.open_melds,
            ctx,
        );

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
        0 => p_advance.clamp(0.05, 0.95),           // テンパイ -> 和了
        1 => (p_advance * 0.45).clamp(0.02, 0.70),  // 一向聴 -> テンパイ -> 和了
        2 => (p_advance * 0.18).clamp(0.01, 0.40),  // 二向聴
        _ => (p_advance * 0.05).clamp(0.001, 0.15), // 三向聴以上
    }
}

/// シャンテン数・有効牌・副露情報から想定和了打点を評価
fn estimate_hand_value(
    counts: &[u8; 35],
    shanten: i8,
    accepted_tiles: &[TileName],
    open_melds: &[crate::hand::Meld],
    ctx: &AnalysisContext<'_>,
) -> ValueMetric {
    let is_closed = open_melds.is_empty();

    // --- ケース 1: テンパイ (shanten == 0) ---
    // accepted_tiles はまさに「和了牌（待ち牌）」。各牌を足すことで和了形になり、即座に厳密な得点計算が可能。
    if shanten == 0 {
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
                is_closed,
                is_tsumo: true,
                seat_wind: ctx.seat_wind,
                round_wind: ctx.round_wind,
                riichi: is_closed, // 門前ならリーチ想定
                win_tile: Some(tile),
                ..Default::default()
            };

            // 実際の副露 (open_melds) を渡して役判定を行う
            let yaku_set = judge_yaku(&working, open_melds, win_ctx);

            let mut han: usize = 0;
            let mut is_yakuman = false;

            for y_info in ALL_YAKU {
                if yaku_set.contains(&y_info.id) {
                    if y_info.yakuman {
                        is_yakuman = true;
                    }
                    let h = if is_closed {
                        y_info.han_closed
                    } else {
                        y_info.han_open
                    };
                    han += h.max(0) as usize;
                    if !yaku_names.contains(&y_info.name_ja) {
                        yaku_names.push(y_info.name_ja);
                    }
                }
            }

            // ドラ加算
            let dora_count = count_dora(&working, ctx.dora_indicators);
            han += dora_count;

            if han == 0 && is_closed {
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

        return ValueMetric {
            expected_score,
            expected_han,
            primary_yaku: yaku_names,
            has_high_value_potential: expected_score >= 5000.0,
        };
    }

    // --- ケース 2: 一向聴 (shanten == 1) ---
    // 有効牌を1枚ツモってテンパイになった先の和了形をシミュレート
    if shanten == 1 && !accepted_tiles.is_empty() {
        let mut total_score = 0.0;
        let mut total_han = 0.0;
        let mut yaku_names = Vec::new();
        let sample_count = accepted_tiles.len().min(3);

        let mut working = *counts;

        for &adv_tile in accepted_tiles.iter().take(sample_count) {
            working[adv_tile as usize] += 1;

            // テンパイになる打牌を探す
            let mut best_move: Option<(usize, TileName)> = None;
            for d in 1..=34 {
                if working[d] == 0 {
                    continue;
                }
                working[d] -= 1;
                let sub_acc = calculate_acceptance(&working, open_melds.len(), None);
                if sub_acc.current_shanten == 0 && !sub_acc.waits.is_empty() {
                    best_move = Some((d, sub_acc.waits[0].tile));
                    working[d] += 1;
                    break;
                }
                working[d] += 1;
            }

            if let Some((discard_idx, win_tile)) = best_move {
                working[discard_idx] -= 1; // テンパイ打牌で余剰牌を除去
                working[win_tile as usize] += 1; // 和了牌を追加（これで正規の和了枚数）

                let win_ctx = WinContext {
                    is_closed,
                    is_tsumo: true,
                    seat_wind: ctx.seat_wind,
                    round_wind: ctx.round_wind,
                    riichi: is_closed,
                    win_tile: Some(win_tile),
                    ..Default::default()
                };

                let yaku_set = judge_yaku(&working, open_melds, win_ctx);
                let mut han = 0;
                let mut is_yakuman = false;

                for y_info in ALL_YAKU {
                    if yaku_set.contains(&y_info.id) {
                        if y_info.yakuman {
                            is_yakuman = true;
                        }
                        let h = if is_closed {
                            y_info.han_closed
                        } else {
                            y_info.han_open
                        };
                        han += h.max(0) as usize;
                        if !yaku_names.contains(&y_info.name_ja) {
                            yaku_names.push(y_info.name_ja);
                        }
                    }
                }

                han += count_dora(&working, ctx.dora_indicators);
                if han == 0 && is_closed {
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

                working[win_tile as usize] -= 1; // 和了牌を除去
                working[discard_idx] += 1; // テンパイ打牌を復元
            } else {
                total_score += if is_closed { 3000.0 } else { 1500.0 };
                total_han += if is_closed { 2.0 } else { 1.0 };
            }

            working[adv_tile as usize] -= 1;
        }

        let count_f = sample_count as f64;
        let expected_score = total_score / count_f;
        let expected_han = total_han / count_f;

        return ValueMetric {
            expected_score,
            expected_han,
            primary_yaku: yaku_names,
            has_high_value_potential: expected_score >= 5000.0,
        };
    }

    // --- ケース 3: 二向聴以上 (shanten >= 2) ---
    // 手牌内の役要素（ドラ、役牌、タンヤオ適合、染め手）からベース打点を推定
    let dora_count = count_dora(counts, ctx.dora_indicators);
    let mut estimated_han = if is_closed { 2.0 } else { 1.0 } + dora_count as f64;
    let mut yaku_names = Vec::new();

    if is_closed {
        yaku_names.push("立直");
        yaku_names.push("門前清自摸和");
    }

    // 役牌の対子・暗刻
    let honor_tiles = [
        TileName::White,
        TileName::Green,
        TileName::Red,
        ctx.seat_wind.unwrap_or(TileName::East),
        ctx.round_wind.unwrap_or(TileName::East),
    ];
    for &h in &honor_tiles {
        let c = counts[h as usize];
        if c >= 2 {
            estimated_han += 1.0;
            if !yaku_names.contains(&"役牌") {
                yaku_names.push("役牌");
            }
        }
    }

    let han_usize = (estimated_han as usize).max(1);
    let score_res = calculate_score(han_usize, 30, ctx.is_dealer, true, false);

    ValueMetric {
        expected_score: score_res.total_points as f64,
        expected_han: estimated_han,
        primary_yaku: yaku_names,
        has_high_value_potential: estimated_han >= 4.0,
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

    #[test]
    fn test_open_meld_yaku_reflected_in_evaluation() {
        // 白をポンした状態 (役牌 白)
        // 手牌: 2m3m4m 5p6p7p 東東 2s3s + 9s (11枚: 10枚 + ツモ1枚)
        let mut hand = Hand::new();
        for &t in &[
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourM,
            TileName::FiveP,
            TileName::SixP,
            TileName::SevenP,
            TileName::East,
            TileName::East,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::NineS, // 余剰牌 (切ると 1s, 4s の両面テンパイ)
        ] {
            hand.push(t);
        }
        hand.open_melds
            .push(crate::hand::Meld::Pon(TileName::White));

        let ctx = AnalysisContext::default();
        let evs = evaluate_hand_discards(&hand, None, &ctx);

        let best = &evs[0];
        assert_eq!(best.discard_tile, TileName::NineS);
        assert_eq!(best.shanten_after, 0);

        // 白ポンの役牌が主要役に反映されていること！
        assert!(
            best.value.primary_yaku.contains(&"役牌 白"),
            "Open meld White Pon must be recognized in yaku list: {:?}",
            best.value.primary_yaku
        );
    }

    #[test]
    fn test_one_shanten_simulation_evaluates_correctly() {
        // 1向聴の手牌:
        // 2m3m4m (面子1), 2p3p4p (面子2), 東東 (雀頭), 4s5s (塔子1), 7s8s (塔子2), 9m (余剰), 9p (余剰) (計14枚)
        // 9m または 9p を切ると 2面子1雀頭2塔子 の一向聴
        let mut hand = Hand::new();
        for &t in &[
            TileName::TwoM,
            TileName::ThreeM,
            TileName::FourM,
            TileName::TwoP,
            TileName::ThreeP,
            TileName::FourP,
            TileName::East,
            TileName::East,
            TileName::FourS,
            TileName::FiveS,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineM, // 余剰牌1
            TileName::NineP, // 余剰牌2
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext::default();
        let evs = evaluate_hand_discards(&hand, None, &ctx);

        let cand_9m = evs
            .iter()
            .find(|e| e.discard_tile == TileName::NineM)
            .expect("Discard 9m should be a valid candidate");

        assert_eq!(cand_9m.shanten_after, 1); // 打9mで一向聴

        // 1向聴のシミュレーション（有効牌ツモ→テンパイ打牌→和了牌ツモ）が手牌14枚で評価され、
        // 想定打点・翻が正しく算出されていること
        assert!(cand_9m.value.expected_score >= 1000.0);
        assert!(cand_9m.value.expected_han >= 1.0);
    }
}
