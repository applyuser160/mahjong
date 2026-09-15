use crate::acceptance::calculate_acceptance;
use crate::dora::count_dora;
use crate::hand::Hand;
use crate::score::{calculate_hand_fu, calculate_score};
use crate::tile::TileName;
use crate::yaku::{judge_yaku_set, WinContext, YakuId, ALL_YAKU};
use arrayvec::ArrayVec;
use rayon::prelude::*;

/// 速度指標
#[derive(Debug, Clone, PartialEq)]
pub struct SpeedMetric {
    pub accepted_tiles: ArrayVec<TileName, 34>,
    pub remaining_count: usize,
    pub win_probability: f64,
}

/// 打点指標
#[derive(Debug, Clone, PartialEq)]
pub struct ValueMetric {
    pub expected_score: f64,
    pub expected_han: f64,
    pub primary_yaku: ArrayVec<&'static str, 10>,
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

/// 13枚手牌（自摸前・他家打牌時）の評価結果
#[derive(Debug, Clone, PartialEq)]
pub struct StandingHandEvaluation {
    pub shanten: i8,
    pub speed: SpeedMetric,
    pub value: ValueMetric,
    pub ev: f64,
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
    // --- 状況・他家コンテキスト（高度化用） ---
    pub target_player: usize, // 分析対象のプレイヤー座席番号 (0..4, デフォルト: 0)
    pub riichi_status: [bool; 4], // 各プレイヤーのリーチ状態
    pub player_rivers: &'a [&'a [TileName]], // 各プレイヤーの捨て牌 (長さ4、空なら参照なし)
    pub player_melds: &'a [&'a [crate::hand::Meld]], // 各プレイヤーの副露 (長さ4)
    pub player_is_dealer: [bool; 4], // 各プレイヤーが親かどうか
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
            target_player: 0,
            riichi_status: [false; 4],
            player_rivers: &[],
            player_melds: &[],
            player_is_dealer: [true, false, false, false],
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

    // 相手にリーチ者がいる場合は放銃失点ペナルティを重くし（ベタオリの優位性）、
    // リーチ者がいない平時は手作りを阻害しないようペナルティを抑制
    let has_riichi_threat = (0..4).any(|p| p != ctx.target_player && ctx.riichi_status[p]);
    let deal_loss_penalty = if has_riichi_threat {
        let dealer_riichi = (0..4)
            .any(|p| p != ctx.target_player && ctx.riichi_status[p] && ctx.player_is_dealer[p]);
        if dealer_riichi {
            6500.0 // 親リーチに対する失点期待値ペナルティ
        } else {
            5000.0 // 子リーチに対する失点期待値ペナルティ
        }
    } else {
        let turn_factor = (ctx.turn_number as f64 / 18.0).clamp(0.25, 1.0);
        turn_factor * 600.0 // 平時序盤〜中盤の緩やかな失点リスク
    };

    let mut evaluations: Vec<CandidateEvaluation> = (1..=34)
        .into_par_iter()
        .filter(|&i| hand.counts[i] > 0)
        .map(|i| {
            let discard_tile = TileName::from_usize(i);
            let mut working = hand.counts;
            working[i] -= 1;

            // 打牌した牌を含む base_visible を可視牌として渡す
            let acceptance = calculate_acceptance(&working, open_melds_count, Some(&base_visible));
            let shanten_after = acceptance.current_shanten;
            let accepted_tiles: ArrayVec<TileName, 34> =
                acceptance.waits.iter().map(|w| w.tile).collect();

            // 1. 速度評価（和了確率） - 待ち形・巡目・脅威度を反映
            let win_probability = estimate_win_probability(
                shanten_after,
                acceptance.total_remaining,
                accepted_tiles.len(),
                remaining_turns,
                wall_remaining,
                ctx,
            );

            let speed = SpeedMetric {
                accepted_tiles: accepted_tiles.clone(),
                remaining_count: acceptance.total_remaining,
                win_probability,
            };

            // 2. 打点評価（テンパイ・1向聴・2向聴以上の正確な符計算とダマテン想定）
            let value = estimate_hand_value(
                &working,
                shanten_after,
                &accepted_tiles,
                &hand.open_melds,
                ctx,
            );

            // 3. 安全度評価（現物・スジ・生牌・カベ・リーチ状況を反映）
            let safety = evaluate_tile_safety(discard_tile, ctx, &base_visible);

            // 4. 総合期待値 (EV)
            let ev = win_probability * value.expected_score - safety.risk_score * deal_loss_penalty;

            CandidateEvaluation {
                discard_tile,
                shanten_after,
                ev,
                speed,
                value,
                safety,
            }
        })
        .collect();

    // 期待値降順にソート（同値時は向聴数昇順 -> 受入枚数降順 -> 牌種ID昇順で決定論的安定性を確保）
    evaluations.sort_by(|a, b| {
        b.ev.partial_cmp(&a.ev)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.shanten_after.cmp(&b.shanten_after))
            .then_with(|| b.speed.remaining_count.cmp(&a.speed.remaining_count))
            .then_with(|| (a.discard_tile as usize).cmp(&(b.discard_tile as usize)))
    });

    evaluations
}

/// 手牌（13枚相当、自摸前・他家打牌時）の受入・打点・和了期待値を評価します。
pub fn evaluate_standing_hand(
    hand: &Hand,
    visible_counts: Option<&[u8; 35]>,
    ctx: &AnalysisContext<'_>,
) -> StandingHandEvaluation {
    let open_melds_count = hand.open_melds.len();
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

    let acceptance = calculate_acceptance(&hand.counts, open_melds_count, Some(&base_visible));
    let shanten = acceptance.current_shanten;
    let accepted_tiles: ArrayVec<TileName, 34> = acceptance.waits.iter().map(|w| w.tile).collect();

    let win_probability = estimate_win_probability(
        shanten,
        acceptance.total_remaining,
        accepted_tiles.len(),
        remaining_turns,
        wall_remaining,
        ctx,
    );

    let speed = SpeedMetric {
        accepted_tiles: accepted_tiles.clone(),
        remaining_count: acceptance.total_remaining,
        win_probability,
    };

    let value = estimate_hand_value(
        &hand.counts,
        shanten,
        &accepted_tiles,
        &hand.open_melds,
        ctx,
    );

    let ev = win_probability * value.expected_score;

    StandingHandEvaluation {
        shanten,
        speed,
        value,
        ev,
    }
}

/// シャンテン数、受け入れ枚数、待ち牌種数、巡目・他家脅威度から和了確率をモデル化 (Issue #76)
fn estimate_win_probability(
    shanten: i8,
    acceptance_remaining: usize,
    wait_tiles_count: usize,
    remaining_turns: f64,
    wall_remaining: f64,
    ctx: &AnalysisContext<'_>,
) -> f64 {
    if shanten < 0 {
        return 1.0; // 既に和了
    }

    let acc = acceptance_remaining as f64;
    // 1巡あたりの有効牌ツモ確率
    let p_draw_per_turn = (acc / wall_remaining).min(0.9);

    // 残り巡目でのツモ確率: 1 - (1 - p)^R
    let p_advance = 1.0 - (1.0 - p_draw_per_turn).powf(remaining_turns);

    // 1. 待ち形による補正（テンパイ時: 好形待ち vs 愚形待ち）
    let shape_factor = if shanten == 0 {
        if wait_tiles_count >= 2 && acceptance_remaining >= 5 {
            1.0 // 両面・多面張
        } else {
            0.65 // 愚形（カンチャン・ペンチャン・単騎）
        }
    } else if shanten == 1 {
        // 一向聴: 受入種数が豊富なら好形一向聴
        if wait_tiles_count >= 4 && acceptance_remaining >= 10 {
            0.45
        } else {
            0.32
        }
    } else if shanten == 2 {
        0.16
    } else {
        0.04
    };

    // 2. 巡目による減衰（終盤に向かうにつれて他家の守備・ツモ流出により和了率低下）
    let turn = ctx.turn_number.clamp(1, 18);
    let turn_decay = if turn <= 6 {
        1.0
    } else if turn <= 12 {
        1.0 - (turn - 6) as f64 * 0.025 // 0.975〜0.85
    } else {
        0.85 - (turn - 12) as f64 * 0.06 // 0.79〜0.49
    };

    // 3. 他家脅威度による減衰（リーチ・多副露による和了阻止・降ろされ率）
    let riichi_count = (0..4)
        .filter(|&p| p != ctx.target_player && ctx.riichi_status[p])
        .count();
    let riichi_factor = match riichi_count {
        0 => 1.0,
        1 => 0.75,
        2 => 0.50,
        _ => 0.30,
    };

    let high_meld_count = (0..4)
        .filter(|&p| {
            p != ctx.target_player
                && p < ctx.player_melds.len()
                && crate::hand::has_open_meld(ctx.player_melds[p])
                && ctx.player_melds[p].len() >= 2
        })
        .count();
    let meld_factor = (1.0 - high_meld_count as f64 * 0.10).max(0.7);

    let threat_factor = (riichi_factor * meld_factor).clamp(0.25, 1.0);

    let raw_p = p_advance * shape_factor * turn_decay * threat_factor;

    match shanten {
        0 => raw_p.clamp(0.02, 0.95),  // テンパイ
        1 => raw_p.clamp(0.01, 0.65),  // 一向聴
        2 => raw_p.clamp(0.005, 0.35), // 二向聴
        _ => raw_p.clamp(0.001, 0.12), // 三向聴以上
    }
}

/// シャンテン数・有効牌・副露情報から想定和了打点を評価 (Issue #77)
fn estimate_hand_value(
    counts: &[u8; 35],
    shanten: i8,
    accepted_tiles: &[TileName],
    open_melds: &[crate::hand::Meld],
    ctx: &AnalysisContext<'_>,
) -> ValueMetric {
    let is_closed = crate::hand::is_menzen(open_melds);

    // --- ケース 1: テンパイ (shanten == 0) ---
    if shanten == 0 {
        if accepted_tiles.is_empty() {
            return ValueMetric {
                expected_score: 1000.0,
                expected_han: 1.0,
                primary_yaku: ArrayVec::new(),
                has_high_value_potential: false,
            };
        }

        let mut total_score = 0.0;
        let mut total_han = 0.0;
        let mut yaku_names = ArrayVec::<&'static str, 10>::new();
        let sample_count = accepted_tiles.len().min(6);

        let mut working = *counts;

        for &tile in accepted_tiles.iter().take(sample_count) {
            let idx = tile as usize;
            working[idx] += 1;

            // まず役判定（門前・役あり判定のためダマテン状態で判定）
            let base_win_ctx = WinContext {
                is_closed,
                is_tsumo: true,
                seat_wind: ctx.seat_wind,
                round_wind: ctx.round_wind,
                riichi: false, // 一旦ダマテン想定で役判定
                win_tile: Some(tile),
                ..Default::default()
            };

            let yaku_set = judge_yaku_set(&working, open_melds, base_win_ctx);

            let mut base_han: usize = 0;
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
                    base_han += h.max(0) as usize;
                    if !yaku_names.contains(&y_info.name_ja) && yaku_names.len() < 10 {
                        yaku_names.push(y_info.name_ja);
                    }
                }
            }

            let dora_count = count_dora(&working, ctx.dora_indicators);
            let total_base_han = base_han + dora_count;

            // 門前で役なしの場合、リーチ必須
            let needs_riichi = is_closed && base_han == 0;
            let will_riichi = is_closed && (needs_riichi || total_base_han <= 3);

            let effective_han = if is_yakuman {
                13
            } else if will_riichi {
                if needs_riichi && !yaku_names.contains(&"立直") && yaku_names.len() < 10 {
                    yaku_names.push("立直");
                }
                // リーチ時は裏ドラ・一発の期待値として約0.3〜0.5翻を加算
                (total_base_han + 1).max(1)
            } else {
                total_base_han.max(1)
            };

            let is_pinfu = yaku_set.contains(&YakuId::Pinfu);
            let is_chitoitsu = yaku_set.contains(&YakuId::Chitoitsu);

            // 正確な符計算
            let fu = calculate_hand_fu(
                &working,
                open_melds,
                tile,
                true, // ツモ想定
                is_pinfu,
                is_chitoitsu,
                ctx.seat_wind,
                ctx.round_wind,
            );

            let score_res = calculate_score(effective_han, fu, ctx.is_dealer, true, is_yakuman);
            total_score += score_res.total_points as f64;
            total_han += effective_han as f64;

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
    // 有効牌を1枚ツモってテンパイになった先の和了形をシミュレート（最大8種まで拡張）
    if shanten == 1 && !accepted_tiles.is_empty() {
        let mut total_score = 0.0;
        let mut total_han = 0.0;
        let mut yaku_names = ArrayVec::<&'static str, 10>::new();
        let sample_count = accepted_tiles.len().min(8);

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
                working[win_tile as usize] += 1; // 和了牌を追加

                let win_ctx = WinContext {
                    is_closed,
                    is_tsumo: true,
                    seat_wind: ctx.seat_wind,
                    round_wind: ctx.round_wind,
                    riichi: is_closed,
                    win_tile: Some(win_tile),
                    ..Default::default()
                };

                let yaku_set = judge_yaku_set(&working, open_melds, win_ctx);
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
                        if !yaku_names.contains(&y_info.name_ja) && yaku_names.len() < 10 {
                            yaku_names.push(y_info.name_ja);
                        }
                    }
                }

                han += count_dora(&working, ctx.dora_indicators);
                if han == 0 && is_closed {
                    han = 1;
                    if !yaku_names.contains(&"立直") && yaku_names.len() < 10 {
                        yaku_names.push("立直");
                    }
                } else if han == 0 {
                    han = 1;
                }

                let is_pinfu = yaku_set.contains(&YakuId::Pinfu);
                let is_chitoitsu = yaku_set.contains(&YakuId::Chitoitsu);
                let fu = calculate_hand_fu(
                    &working,
                    open_melds,
                    win_tile,
                    true,
                    is_pinfu,
                    is_chitoitsu,
                    ctx.seat_wind,
                    ctx.round_wind,
                );

                let score_res = calculate_score(han, fu, ctx.is_dealer, true, is_yakuman);
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
    let dora_count = count_dora(counts, ctx.dora_indicators);
    let mut estimated_han = if is_closed { 2.0 } else { 1.0 } + dora_count as f64;
    let mut yaku_names = ArrayVec::<&'static str, 10>::new();

    if is_closed {
        yaku_names.push("立直");
        yaku_names.push("門前清自摸和");
    }

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
            if !yaku_names.contains(&"役牌") && yaku_names.len() < 10 {
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

/// 牌の安全度（現物・スジ・生牌・カベ・リーチ状況）を動的に評価 (Issue #78)
pub fn evaluate_tile_safety(
    tile: TileName,
    ctx: &AnalysisContext<'_>,
    visible_counts: &[u8; 35],
) -> SafetyMetric {
    let idx = tile as usize;

    // 他家にリーチ者がいるか？
    let has_riichi_opponents = (0..4).any(|p| p != ctx.target_player && ctx.riichi_status[p]);

    if has_riichi_opponents {
        // --- リーチ者がいる状況での安全度評価 ---

        // 1. 現物判定: 全リーチ者の河にこの牌が含まれているか
        let mut is_genbutsu_all = true;
        let mut is_genbutsu_any = false;

        for p in 0..4 {
            if p != ctx.target_player && ctx.riichi_status[p] {
                if let Some(river) = ctx.player_rivers.get(p) {
                    if river.contains(&tile) {
                        is_genbutsu_any = true;
                    } else {
                        is_genbutsu_all = false;
                    }
                } else {
                    is_genbutsu_all = false;
                }
            }
        }

        // 全リーチ者に対して現物であれば完全安全
        if is_genbutsu_all && is_genbutsu_any {
            return SafetyMetric {
                risk_score: 0.0,
                is_safe: true,
            };
        } else if is_genbutsu_any {
            // 複数リーチ者のうち一部の現物
            return SafetyMetric {
                risk_score: 0.20,
                is_safe: false,
            };
        }

        // 2. 字牌判定 (idx >= 28)
        if idx >= 28 {
            let vis = visible_counts[idx];
            return match vis {
                4 => SafetyMetric {
                    risk_score: 0.0,
                    is_safe: true,
                }, // 4枚見え（完全安牌）
                3 => SafetyMetric {
                    risk_score: 0.05,
                    is_safe: true,
                }, // 3枚見え（地獄単騎以外通る）
                2 => SafetyMetric {
                    risk_score: 0.12,
                    is_safe: true,
                }, // 2枚見え
                _ => SafetyMetric {
                    risk_score: 0.35,
                    is_safe: false,
                }, // 生牌（役牌・単騎の危険大）
            };
        }

        // 3. 数牌判定（スジ・カベ判定）
        let (suit, rank) = if idx <= 9 {
            (0, idx)
        } else if idx <= 18 {
            (1, idx - 9)
        } else {
            (2, idx - 18)
        };

        // リーチ者の河にある同色の数字を収集（rank 1-9 → bit 1-9 の u16 ビットマスク）
        let mut riichi_river_ranks = 0u16;
        for p in 0..4 {
            if p != ctx.target_player && ctx.riichi_status[p] {
                if let Some(river) = ctx.player_rivers.get(p) {
                    for &r_tile in *river {
                        let r_idx = r_tile as usize;
                        let (r_suit, r_rank) = if (1..=9).contains(&r_idx) {
                            (0, r_idx)
                        } else if (10..=18).contains(&r_idx) {
                            (1, r_idx - 9)
                        } else if (19..=27).contains(&r_idx) {
                            (2, r_idx - 18)
                        } else {
                            (99, 99)
                        };
                        if r_suit == suit {
                            riichi_river_ranks |= 1 << r_rank;
                        }
                    }
                }
            }
        }

        let has_rank = |r: usize| (riichi_river_ranks & (1 << r)) != 0;

        // スジ判定
        let is_suji = match rank {
            1 => has_rank(4),
            2 => has_rank(5),
            3 => has_rank(6),
            4 => has_rank(1) && has_rank(7),
            5 => has_rank(2) && has_rank(8),
            6 => has_rank(3) && has_rank(9),
            7 => has_rank(4),
            8 => has_rank(5),
            9 => has_rank(6),
            _ => false,
        };

        let is_half_suji = match rank {
            4 => has_rank(1) || has_rank(7),
            5 => has_rank(2) || has_rank(8),
            6 => has_rank(3) || has_rank(9),
            _ => false,
        };

        if is_suji {
            let risk = match rank {
                1 | 9 => 0.08,
                2 | 8 => 0.18,
                4..=6 => 0.15, // 両スジ
                _ => 0.28,
            };
            SafetyMetric {
                risk_score: risk,
                is_safe: risk < 0.20,
            }
        } else if is_half_suji {
            SafetyMetric {
                risk_score: 0.35,
                is_safe: false,
            }
        } else {
            // 無筋
            let risk = match rank {
                1 | 9 => 0.25,
                2 | 8 => 0.45,
                _ => 0.75, // 3〜7の無筋中張牌は超危険
            };
            SafetyMetric {
                risk_score: risk,
                is_safe: false,
            }
        }
    } else {
        // --- リーチ者がいない平時の安全度評価 ---
        let turn = ctx.turn_number.clamp(1, 18);
        if idx >= 28 {
            let vis = visible_counts[idx];
            let risk = if vis >= 2 { 0.05 } else { 0.12 };
            SafetyMetric {
                risk_score: risk,
                is_safe: true,
            }
        } else {
            let rank = (idx - 1) % 9 + 1;
            let base_risk = match rank {
                1 | 9 => 0.10,
                2 | 8 => 0.20,
                _ => 0.35,
            };
            // 序盤なら中張牌の切り出しリスクをさらに抑える
            let turn_factor = if turn <= 6 {
                0.5
            } else if turn <= 12 {
                0.8
            } else {
                1.0
            };
            let risk = base_risk * turn_factor;
            SafetyMetric {
                risk_score: risk,
                is_safe: risk <= 0.15,
            }
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

    #[test]
    fn test_evaluate_standing_hand() {
        // 13枚のテンパイ手牌: 123m 456p 789s EE 2s (3s単騎待ち)
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
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext::default();
        let eval = evaluate_standing_hand(&hand, None, &ctx);
        // 13枚手牌が正しく評価されること
        assert_eq!(eval.shanten, 0); // テンパイ
        assert!(eval.speed.remaining_count > 0);
        assert!(eval.ev > 1000.0);
    }

    #[test]
    fn test_win_probability_favorable_vs_unfavorable_and_threat_decay() {
        // Issue #76 検証: 好形待ち vs 愚形待ち、他家リーチによる減衰
        let ctx_normal = AnalysisContext::default();

        // テンパイ 好形（両面8枚待ち）
        let p_good = estimate_win_probability(0, 8, 2, 12.0, 50.0, &ctx_normal);
        // テンパイ 愚形（カンチャン4枚待ち）
        let p_bad = estimate_win_probability(0, 4, 1, 12.0, 50.0, &ctx_normal);

        assert!(
            p_good > p_bad,
            "Good wait win probability ({}) must be strictly higher than bad wait ({})",
            p_good,
            p_bad
        );

        // 他家リーチ時の減衰
        let mut ctx_riichi = AnalysisContext::default();
        ctx_riichi.riichi_status[1] = true; // 下家リーチ
        let p_under_riichi = estimate_win_probability(0, 8, 2, 12.0, 50.0, &ctx_riichi);

        assert!(
            p_under_riichi < p_good,
            "Win probability under opponent riichi ({}) must be lower than normal ({})",
            p_under_riichi,
            p_good
        );
    }

    #[test]
    fn test_hand_value_accurate_fu_and_chitoitsu() {
        // Issue #77 検証: 七対子（25符）の想定打点が計算されること
        // 13枚のテンパイ手牌 (4s単騎待ち)
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::OneM,
            TileName::ThreeM,
            TileName::ThreeM,
            TileName::FiveM,
            TileName::FiveM,
            TileName::SevenP,
            TileName::SevenP,
            TileName::NineP,
            TileName::NineP,
            TileName::TwoS,
            TileName::TwoS,
            TileName::FourS, // 13枚目 (4s待ち)
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext {
            is_dealer: false,
            ..Default::default()
        };
        let ev = estimate_hand_value(&hand.counts, 0, &[TileName::FourS], &hand.open_melds, &ctx);

        // 七対子 (2翻25符) -> 1600点 (リーチ想定なら3翻25符 -> 3200点)
        assert!(ev.primary_yaku.contains(&"七対子"));
        assert!(ev.expected_score >= 1600.0);
    }

    #[test]
    fn test_tile_safety_genbutsu_and_suji_defense() {
        // Issue #78 検証: 現物判定（0.0）、スジ判定（低危険度）、無筋（高危険度）
        let river_opp = vec![TileName::FourM, TileName::East]; // 下家河: 4m, 東
        let rivers: [&[TileName]; 4] = [&[], &river_opp, &[], &[]];

        let mut ctx = AnalysisContext::default();
        ctx.riichi_status[1] = true; // 下家リーチ
        ctx.player_rivers = &rivers;

        let visible = [0u8; 35];

        // 1. 現物: 4m
        let s_genbutsu = evaluate_tile_safety(TileName::FourM, &ctx, &visible);
        assert_eq!(s_genbutsu.risk_score, 0.0);
        assert!(s_genbutsu.is_safe);

        // 2. スジ: 1m (4mが河にあるのでスジ)
        let s_suji = evaluate_tile_safety(TileName::OneM, &ctx, &visible);
        assert!(s_suji.risk_score < 0.20);
        assert!(s_suji.is_safe);

        // 3. 無筋中張牌: 5m (無筋中張牌は高危険度)
        let s_danger = evaluate_tile_safety(TileName::FiveM, &ctx, &visible);
        assert!(s_danger.risk_score >= 0.70);
        assert!(!s_danger.is_safe);

        // 4. ベタオリ局面でのEV比較
        // 手牌に 現物(4m) と 無筋(5m) がある時、放銃ペナルティにより現物切りのEVが高くなること
        let mut hand = Hand::new();
        for &t in &[
            TileName::FourM, // 現物
            TileName::FiveM, // 無筋危険牌
            TileName::NineP,
            TileName::NineP,
            TileName::OneS,
            TileName::TwoS,
            TileName::ThreeS,
            TileName::SevenS,
            TileName::EightS,
            TileName::NineS,
            TileName::West,
            TileName::West,
            TileName::North,
            TileName::North,
        ] {
            hand.push(t);
        }

        let evs = evaluate_hand_discards(&hand, None, &ctx);
        let ev_4m = evs
            .iter()
            .find(|e| e.discard_tile == TileName::FourM)
            .unwrap();
        let ev_5m = evs
            .iter()
            .find(|e| e.discard_tile == TileName::FiveM)
            .unwrap();

        assert!(
            ev_4m.ev > ev_5m.ev,
            "Genbutsu 4m EV ({}) must be strictly higher than dangerous 5m EV ({}) under riichi",
            ev_4m.ev,
            ev_5m.ev
        );
    }

    #[test]
    fn test_analysis_context_target_player_non_zero() {
        // Issue #78 レビュー対応: target_player = 2（CPU対面）の分析時
        // 1. 自身のリーチ（player 2）は他家脅威として扱われないこと
        // 2. プレイヤー0（あなた）のリーチ・河が正しく他家脅威・現物として判定されること
        let river_p0 = vec![TileName::FourM, TileName::West];
        let river_p2 = vec![TileName::OneS];
        let all_rivers = vec![river_p0.as_slice(), &[], river_p2.as_slice(), &[]];

        let mut riichi_status = [false; 4];
        riichi_status[0] = true; // プレイヤー0（他家）がリーチ
        riichi_status[2] = true; // プレイヤー2（分析対象の自身）がリーチ

        let ctx = AnalysisContext {
            target_player: 2, // 分析対象はプレイヤー2
            riichi_status,
            player_rivers: &all_rivers,
            ..Default::default()
        };

        // プレイヤー0の河にある 4m は現物（0.0）
        let s_4m = evaluate_tile_safety(TileName::FourM, &ctx, &[0u8; 35]);
        assert_eq!(
            s_4m.risk_score, 0.0,
            "プレイヤー0の河にある4mは他家現物として0.0であるべき"
        );
        assert!(s_4m.is_safe);

        // プレイヤー2自身の河にある 1s は他家現物ではない（無筋端牌 0.25）
        let s_1s = evaluate_tile_safety(TileName::OneS, &ctx, &[0u8; 35]);
        assert!(
            s_1s.risk_score > 0.0,
            "自身の河にある1sは他家リーチに対する現物とはみなされないべき"
        );

        // 和了確率において、他家リーチは player 0 の「1件」のみとして減衰（0.75倍）されること
        // （自身のリーチを入れて2件減衰 0.50倍 になったり、0件 1.0倍 になったりしない）
        let p_with_p0_riichi = estimate_win_probability(0, 8, 2, 10.0, 50.0, &ctx);

        let ctx_no_riichi = AnalysisContext {
            target_player: 2,
            riichi_status: [false, false, true, false], // 自身のみリーチ（他家ノーリーチ）
            player_rivers: &all_rivers,
            ..Default::default()
        };
        let p_self_only = estimate_win_probability(0, 8, 2, 10.0, 50.0, &ctx_no_riichi);

        assert!(
            p_self_only > p_with_p0_riichi,
            "プレイヤー0のリーチにより和了確率は減衰するべき (self_only: {}, with_p0: {})",
            p_self_only,
            p_with_p0_riichi
        );
    }

    #[test]
    fn test_estimate_hand_value_ankan_preserves_menzen() {
        // Issue #77 レビュー対応: 暗槓（Meld::Ankan）のみを持つ手牌は門前清として打点評価されること
        // 手牌: 234p 456p 78s 55m (10枚門前牌) + 1m暗槓 (Meld::Ankan(1m))
        // 待ち: 6s, 9s (両面テンパイ)
        let mut counts = [0u8; 35];
        counts[TileName::TwoP as usize] = 1;
        counts[TileName::ThreeP as usize] = 1;
        counts[TileName::FourP as usize] = 2;
        counts[TileName::FiveP as usize] = 1;
        counts[TileName::SixP as usize] = 1;
        counts[TileName::SevenS as usize] = 1;
        counts[TileName::EightS as usize] = 1;
        counts[TileName::FiveM as usize] = 2; // 雀頭

        let open_melds = vec![crate::hand::Meld::Ankan(TileName::OneM)];
        let accepted = vec![TileName::SixS, TileName::NineS];

        let ctx = AnalysisContext {
            is_dealer: false,
            seat_wind: Some(TileName::South),
            round_wind: Some(TileName::East),
            ..Default::default()
        };

        let val = estimate_hand_value(&counts, 0, &accepted, &open_melds, &ctx);
        // 暗槓は門前清であるため立直・ツモ・ピンフ等が評価され、副露手（1000点）ではなく高打点となる
        assert!(
            val.expected_score >= 3000.0,
            "暗槓のみのテンパイは門前清として立直込みで高打点評価されるべき (actual: {})",
            val.expected_score
        );
        assert!(
            val.primary_yaku.contains(&"立直") || val.expected_han >= 2.0,
            "門前役（立直等）が評価に含まれるべき"
        );
    }

    #[test]
    fn test_evaluate_hand_discards_riichi_threat_target_player_alignment() {
        // レビュー指摘事項対応: target_player = 2 で「座席0のみがリーチ」と「座席2自身のみがリーチ」を比較
        // 前者のみ他家リーチ脅威となり、後者は自身のリーチなので平時ペナルティとなることを検証
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
            TileName::FiveM, // 無筋中張牌（危険牌）
            TileName::West,  // 字牌
            TileName::North, // 余剰牌
        ] {
            hand.push(t);
        }

        let ctx_base = AnalysisContext {
            target_player: 2,
            turn_number: 9,
            remaining_wall_tiles: 50,
            seat_wind: Some(TileName::West),
            round_wind: Some(TileName::East),
            ..Default::default()
        };

        // 1. 座席2自身のみがリーチしている局面
        let mut ctx_self_riichi = ctx_base;
        ctx_self_riichi.riichi_status = [false, false, true, false];

        // 2. 全員ノーリーチの平時局面
        let mut ctx_no_riichi = ctx_base;
        ctx_no_riichi.riichi_status = [false, false, false, false];

        // 3. 座席0（他家）のみがリーチしている局面
        let mut ctx_p0_riichi = ctx_base;
        ctx_p0_riichi.riichi_status = [true, false, false, false];

        let evs_self_riichi = evaluate_hand_discards(&hand, None, &ctx_self_riichi);
        let evs_no_riichi = evaluate_hand_discards(&hand, None, &ctx_no_riichi);
        let evs_p0_riichi = evaluate_hand_discards(&hand, None, &ctx_p0_riichi);

        // 自身のみリーチ（ctx_self_riichi）の場合、他家脅威がないため平時（ctx_no_riichi）と同一の打牌評価となること
        let cand_5m_self = evs_self_riichi
            .iter()
            .find(|e| e.discard_tile == TileName::FiveM)
            .unwrap();
        let cand_5m_no = evs_no_riichi
            .iter()
            .find(|e| e.discard_tile == TileName::FiveM)
            .unwrap();
        assert!(
            (cand_5m_self.ev - cand_5m_no.ev).abs() < 1e-4,
            "自身のみリーチ時のEV ({}) は平時EV ({}) と一致するべき（自身のリーチは他家脅威ではない）",
            cand_5m_self.ev,
            cand_5m_no.ev
        );

        // 座席0がリーチ（ctx_p0_riichi）の場合、他家脅威ペナルティ（5000点）が適用され、危険牌（5m）のEVが大幅に低下すること
        let cand_5m_p0 = evs_p0_riichi
            .iter()
            .find(|e| e.discard_tile == TileName::FiveM)
            .unwrap();
        assert!(
            cand_5m_p0.ev < cand_5m_self.ev - 1000.0,
            "座席0リーチ時の危険牌EV ({}) は自身のみリーチ時 ({}) より大幅に低いペナルティを受けるべき",
            cand_5m_p0.ev,
            cand_5m_self.ev
        );
    }

    #[test]
    fn test_parallel_evaluate_hand_discards_deterministic() {
        // 14種すべての候補が存在する手牌（最大並列度14タスク）
        let mut hand = Hand::new();
        for &t in &[
            TileName::OneM,
            TileName::NineM,
            TileName::OneP,
            TileName::NineP,
            TileName::OneS,
            TileName::NineS,
            TileName::East,
            TileName::South,
            TileName::West,
            TileName::North,
            TileName::White,
            TileName::Green,
            TileName::Red,
            TileName::FiveM,
        ] {
            hand.push(t);
        }

        let ctx = AnalysisContext::default();
        let evs1 = evaluate_hand_discards(&hand, None, &ctx);
        let evs2 = evaluate_hand_discards(&hand, None, &ctx);

        assert_eq!(evs1.len(), 14);
        assert_eq!(evs2.len(), 14);

        for (e1, e2) in evs1.iter().zip(evs2.iter()) {
            assert_eq!(e1.discard_tile, e2.discard_tile);
            assert_eq!(e1.shanten_after, e2.shanten_after);
            assert_eq!(e1.speed.remaining_count, e2.speed.remaining_count);
            assert!((e1.ev - e2.ev).abs() < 1e-6);
        }
    }
}
