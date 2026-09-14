use std::io::{self, Write};

use mahjong::call_advisor::{CallAction, CallAdvisor, CallRecommendation};
use mahjong::dora::indicator_to_dora;
use mahjong::expectation::{evaluate_hand_discards, AnalysisContext, CandidateEvaluation};
use mahjong::explanation::Explainer;
use mahjong::hand::{Hand, Meld};
use mahjong::review::ReviewTracker;
use mahjong::shanten::calculate_shanten;
use mahjong::tile::TileName;
use mahjong::wall::Wall;
use rand::rngs::StdRng;
use rand::SeedableRng;

use mahjong::placement_ev::{
    calculate_orasu_conditions, evaluate_hand_discards_with_placement, MatchContext, RuleConfig,
};
use mahjong::yaku::{judge_yaku, WinContext};

const PLAYER_NAMES: [&str; 4] = [
    "あなた (東家)",
    "CPU下家 (南家)",
    "CPU対面 (西家)",
    "CPU上家 (北家)",
];

fn format_meld(meld: &Meld) -> String {
    match meld {
        Meld::Chii { called, consumed } => {
            let mut tiles = [*called, consumed[0], consumed[1]];
            tiles.sort();
            format!(
                "チー[{}-{}-{}]",
                tiles[0].as_str(),
                tiles[1].as_str(),
                tiles[2].as_str()
            )
        }
        Meld::Pon(t) => format!("ポン[{}-{}-{}]", t.as_str(), t.as_str(), t.as_str()),
        Meld::Daiminkan(t) | Meld::Ankan(t) | Meld::Kakan(t) => {
            format!("カン[{}]", t.as_str())
        }
    }
}

fn player_seat_wind(player_idx: usize) -> TileName {
    match player_idx {
        0 => TileName::East,
        1 => TileName::South,
        2 => TileName::West,
        _ => TileName::North,
    }
}

fn check_agari(
    hand: &Hand,
    win_tile: TileName,
    is_tsumo: bool,
    seat_wind: TileName,
    round_wind: TileName,
    is_riichi: bool,
) -> bool {
    let test_hand = if is_tsumo {
        hand.clone()
    } else {
        let mut h = hand.clone();
        h.push(win_tile);
        h
    };
    if calculate_shanten(&test_hand).min_shanten >= 0 {
        return false;
    }
    let win_ctx = WinContext {
        is_closed: hand.open_melds.is_empty(),
        is_tsumo,
        seat_wind: Some(seat_wind),
        round_wind: Some(round_wind),
        riichi: is_riichi,
        win_tile: Some(win_tile),
        ..Default::default()
    };
    let yaku_set = judge_yaku(&test_hand.counts, &hand.open_melds, win_ctx);
    !yaku_set.is_empty()
}

fn main() {
    println!("================================================================================");
    println!("🀄 リアルタイム期待値ヒント＆意思決定モデル 4人対局CPU学習モード (副露対応) 🀄");
    println!("================================================================================");
    println!("ルール:");
    println!("  ・4人対局 (25,000点持ち / 30,000点返し)");
    println!("  ・順位点: Mリーグ基準 (+50 / +10 / -10 / -30, オカ内包)");
    println!("  ・各手番でAIが打牌期待値（素点EV ＆ 順位Pt EV）・受入・打点・放銃リスクをリアルタイム算出");
    println!("  ・局進行に伴う点差・点況判断・オーラス逆転条件・押し引きを学習可能");
    println!("  ・終局時に最終成績・順位Pt精算・あなたの打牌精度スコアを表示\n");

    println!("対局モードを選択してください:");
    println!("  [1] 東風戦（東1局〜東4局オーラス、局進行・持ち点引き継ぎ・最終順位精算）[Enter]");
    println!(
        "  [2] オーラス逆転特訓（南4局 0本場、2位26,000点 vs 1位33,000点 トップ逆転条件に挑戦）"
    );
    print!("選択 (1または2, デフォルト: 1): ");
    io::stdout().flush().unwrap();
    let mut mode_in = String::new();
    let _ = io::stdin().read_line(&mut mode_in);
    let is_orasu_training = mode_in.trim() == "2";

    let mut match_ctx = if is_orasu_training {
        println!("\n⚡ 【オーラス逆転特訓モード】を開始します！");
        println!("  設定: 南4局 0本場、自家26,000点(2位)、対面33,000点(1位・点差7,000点)");
        MatchContext {
            scores: [26000, 22000, 33000, 19000],
            round_wind: TileName::South,
            round_number: 4,
            honba: 0,
            riichi_sticks: 0,
            dealer_idx: 3, // 上家が親
            rule: RuleConfig::mleague(),
        }
    } else {
        println!("\n🀄 【東風戦（東1局〜東4局オーラス）】を開始します！");
        MatchContext {
            scores: [25000, 25000, 25000, 25000],
            round_wind: TileName::East,
            round_number: 1,
            honba: 0,
            riichi_sticks: 0,
            dealer_idx: 0,
            rule: RuleConfig::mleague(),
        }
    };

    let mut tracker = ReviewTracker::new();
    let mut rng = StdRng::from_entropy();

    'match_loop: loop {
        let round_wind_str = match match_ctx.round_wind {
            TileName::East => "東",
            TileName::South => "南",
            _ => "西",
        };
        let is_orasu = match_ctx.round_number == 4
            && (match_ctx.round_wind == TileName::East || match_ctx.round_wind == TileName::South);
        let orasu_badge = if is_orasu {
            " 🔥【オーラス】"
        } else {
            ""
        };

        println!("\n{}", "#".repeat(80));
        println!(
            "🀄 【{}{}局 {}本場{}】 親: {} | 供託: {}本",
            round_wind_str,
            match_ctx.round_number,
            match_ctx.honba,
            orasu_badge,
            PLAYER_NAMES[match_ctx.dealer_idx],
            match_ctx.riichi_sticks
        );
        println!("{}", "#".repeat(80));

        let mut wall = Wall::new();
        wall.shuffle(&mut rng);

        let dora_indicator = wall.tiles()[135];
        let dora_tile = indicator_to_dora(dora_indicator);

        // 4人の手牌と河
        let mut hands: [Hand; 4] = [Hand::new(), Hand::new(), Hand::new(), Hand::new()];
        let mut rivers: [Vec<TileName>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut riichi_declared: [bool; 4] = [false, false, false, false];

        // 配牌 13枚ずつ
        for _ in 0..13 {
            for hand in &mut hands {
                if let Some(t) = wall.draw() {
                    hand.push(t);
                }
            }
        }

        let mut turn_count = 1;
        let mut current_turn = match_ctx.dealer_idx; // 親から手番スタート
        let mut skip_draw = false; // 副露直後の手番はツモをスキップ

        let renchan: bool = 'game: loop {
            let p_name = PLAYER_NAMES[current_turn];

            // 1. ツモ処理（副露直後でない場合のみ山から引く）
            let drawn_tile = if !skip_draw {
                let drawn = match wall.draw() {
                    Some(t) => t,
                    None => {
                        println!("\n【流局】山牌が尽きました。局終了です。");
                        // テンパイ料精算
                        let tenpai_status: Vec<bool> = (0..4)
                            .map(|p| calculate_shanten(&hands[p]).min_shanten == 0)
                            .collect();
                        let tenpai_count = tenpai_status.iter().filter(|&&is_t| is_t).count();
                        if tenpai_count > 0 && tenpai_count < 4 {
                            let gain = 3000 / tenpai_count as i32;
                            let pay = 3000 / (4 - tenpai_count) as i32;
                            for (p, &is_tenpai) in tenpai_status.iter().enumerate() {
                                if is_tenpai {
                                    match_ctx.scores[p] += gain;
                                } else {
                                    match_ctx.scores[p] -= pay;
                                }
                            }
                        }
                        break 'game tenpai_status[match_ctx.dealer_idx];
                    }
                };
                hands[current_turn].push(drawn);
                Some(drawn)
            } else {
                skip_draw = false;
                None
            };

            let discarded_tile: TileName;
            let discarder = current_turn;

            // --- A. 自家（プレイヤー）の手番 ---
            if current_turn == 0 {
                let remaining_wall = wall.remaining();
                let ranks = match_ctx.current_ranks();
                let dealer_label = if match_ctx.dealer_idx == 0 {
                    "親/自家"
                } else {
                    "子"
                };
                println!("\n{}", "=".repeat(80));
                println!(
                    "【{}{}局 {}本場{}】 巡目: {}巡目 ({}) | 残り山牌: {}枚 | ドラ表示牌: [{}] (ドラ: [{}])",
                    round_wind_str, match_ctx.round_number, match_ctx.honba, orasu_badge, turn_count, dealer_label, remaining_wall, dora_indicator.as_str(), dora_tile.as_str()
                );

                // 点況・スコア状況の表示
                println!("{}", "-".repeat(80));
                print!("📊 【点況・スコア状況 (Mリーグ順位点: +50/+10/-10/-30)】: ");
                for p in 0..4 {
                    let diff = match_ctx.scores[p] - match_ctx.scores[0];
                    let diff_str = if p == 0 {
                        "".to_string()
                    } else if diff >= 0 {
                        format!(" (+{}点)", diff)
                    } else {
                        format!(" ({}点)", diff)
                    };
                    print!(
                        "{}位: {} ({}点{})  ",
                        ranks[p], PLAYER_NAMES[p], match_ctx.scores[p], diff_str
                    );
                }
                println!();

                // オーラス／点況逆転ガイド
                let orasu_conds = calculate_orasu_conditions(&match_ctx, 0);
                if !orasu_conds.is_empty() {
                    println!("🎯 【点況ガイド】: {}", orasu_conds[0].summary);
                }

                // 他家の河とリーチ・副露状況を表示
                println!("{}", "-".repeat(80));
                for p in 1..4 {
                    let riichi_str = if riichi_declared[p] {
                        " 🚨[リーチ中]"
                    } else {
                        ""
                    };
                    let melds_str = if !hands[p].open_melds.is_empty() {
                        let meld_texts: Vec<String> =
                            hands[p].open_melds.iter().map(format_meld).collect();
                        format!(" [副露: {}]", meld_texts.join(" "))
                    } else {
                        String::new()
                    };
                    let river_tiles: Vec<&str> = rivers[p].iter().map(|t| t.as_str()).collect();
                    println!(
                        "{:<16}{}{}: {}",
                        PLAYER_NAMES[p],
                        riichi_str,
                        melds_str,
                        river_tiles.join(" ")
                    );
                }
                let my_river_tiles: Vec<&str> = rivers[0].iter().map(|t| t.as_str()).collect();
                println!("{:<16}  : {}", "あなたの河", my_river_tiles.join(" "));
                println!("{}", "-".repeat(80));

                // 手牌および副露の表示
                print!("あなたの手牌: ");
                let tiles_len = hands[0].tiles().len();
                for (i, &t) in hands[0].tiles().iter().enumerate() {
                    if drawn_tile.is_some() && i == tiles_len - 1 {
                        print!("  ツモ: [{}:{}]", i + 1, t.as_str());
                    } else {
                        print!("[{}:{}] ", i + 1, t.as_str());
                    }
                }
                if !hands[0].open_melds.is_empty() {
                    let meld_texts: Vec<String> =
                        hands[0].open_melds.iter().map(format_meld).collect();
                    print!("  副露: {}", meld_texts.join(" "));
                }
                println!();

                // 和了判定（ツモ和了）
                let is_agari = drawn_tile.is_some_and(|dt| {
                    check_agari(
                        &hands[0],
                        dt,
                        true,
                        TileName::East,
                        TileName::East,
                        riichi_declared[0],
                    )
                });
                if is_agari {
                    println!(
                        "\n🎉 【ツモ和了可能】完成形です！ ('tsumo' または 'ツモ' で和了可能)"
                    );
                }

                // 期待値計算 & リアルタイムヒント（順位EV統合）
                let ctx = AnalysisContext {
                    turn_number: turn_count,
                    remaining_wall_tiles: remaining_wall,
                    seat_wind: Some(TileName::East),
                    round_wind: Some(TileName::East),
                    dora_indicators: &[dora_indicator],
                    is_dealer: true,
                };

                let mut placement_evals =
                    evaluate_hand_discards_with_placement(&hands[0], None, &ctx, &match_ctx, 0);
                let any_riichi = riichi_declared[1] || riichi_declared[2] || riichi_declared[3];

                if any_riichi {
                    for pev in &mut placement_evals {
                        // リーチ者の河にある牌は現物（100%安全）
                        let is_genbutsu = (1..4)
                            .filter(|&p| riichi_declared[p])
                            .all(|p| rivers[p].contains(&pev.base.discard_tile));

                        if is_genbutsu {
                            pev.base.safety.risk_score = 0.0;
                            pev.base.safety.is_safe = true;
                        } else {
                            let d = pev.base.discard_tile;
                            let is_honor = d as usize >= 28;
                            let rank = (d as usize - 1) % 9 + 1;
                            let is_terminal = rank == 1 || rank == 9;

                            if is_honor || is_terminal {
                                pev.base.safety.risk_score = 0.15;
                            } else {
                                pev.base.safety.risk_score = 0.45; // 無筋中張牌
                            }
                            pev.base.safety.is_safe = false;
                        }
                        let win_prob = pev.base.speed.win_probability;
                        let value = pev.base.value.expected_score;
                        pev.base.ev = win_prob * value - pev.base.safety.risk_score * 8000.0;
                        // 放銃リスクに応じた順位EVペナルティ
                        pev.placement_ev -= pev.base.safety.risk_score * 30.0; // 満貫放銃で約30pt損失想定
                    }

                    // 順位EV降順で再ソート
                    placement_evals.sort_by(|a, b| {
                        b.placement_ev
                            .partial_cmp(&a.placement_ev)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }

                println!("{}", "-".repeat(80));
                println!("💡 【リアルタイム期待値＆順位EVヒント (AI Advisor)】");
                if any_riichi {
                    println!(
                        "  ⚠️ 他家リーチが入っています！放銃失点リスクを反映した押し引き評価です。"
                    );
                }
                if !placement_evals.is_empty() {
                    println!("  🎯 点況戦術: {}", placement_evals[0].situational_note);
                }

                for (rank, pev) in placement_evals.iter().take(4).enumerate() {
                    let shanten_text = match pev.base.shanten_after {
                        0 => "テンパイ".to_string(),
                        s => format!("{s}向聴"),
                    };
                    let safety_text = if pev.base.safety.risk_score == 0.0 {
                        "🛡️ 現物(安全)"
                    } else if pev.base.safety.risk_score < 0.25 {
                        "🟡 比較的安全"
                    } else {
                        "🔴 危険牌"
                    };

                    let mark = if rank == 0 { "★最善" } else { "  候補" };
                    println!(
                    "  {} [{:2}] 打[{:2}] -> {:>4} | Pt EV:{:>+5.1}pt | 素点EV:{:>+6.0}点 | 期待着順:{:.1}位 | 受入:{:>2}枚 | 打点:{:>5.0}点 | 守備: {}",
                    mark,
                    rank + 1,
                    pev.base.discard_tile.as_str(),
                    shanten_text,
                    pev.placement_ev,
                    pev.base.ev,
                    pev.expected_rank,
                    pev.base.speed.remaining_count,
                    pev.base.value.expected_score,
                    safety_text
                );
                }

                let raw_evals: Vec<CandidateEvaluation> =
                    placement_evals.iter().map(|p| p.base.clone()).collect();
                let rationale = Explainer::generate_rationale(&raw_evals);
                println!("\n📖 【なぜその打牌なのか (AIの判断根拠)】");
                println!("  {rationale}");
                println!("{}", "=".repeat(80));

                // 打牌入力
                let best_discard_tile = placement_evals[0].base.discard_tile;
                let chosen_index = loop {
                    print!(
                        "何を切りますか？ (1〜{}, 牌名, Enter/autoで最善[{}], q:中断): ",
                        hands[0].tiles().len(),
                        best_discard_tile.as_str()
                    );
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_err() {
                        continue;
                    }
                    let input = input.trim();

                    if input.eq_ignore_ascii_case("q") {
                        println!("対局を中断します。");
                        break 'match_loop;
                    }

                    if is_agari && (input.eq_ignore_ascii_case("tsumo") || input == "ツモ") {
                        if match_ctx.dealer_idx == 0 {
                            println!(
                            "\n🎉 【ツモ和了！】見事なアガリです！親満貫 12,000点 (各子 4,000点オール)"
                        );
                            match_ctx.scores[0] += 12000;
                            match_ctx.scores[1] -= 4000;
                            match_ctx.scores[2] -= 4000;
                            match_ctx.scores[3] -= 4000;
                        } else {
                            println!(
                            "\n🎉 【ツモ和了！】見事なアガリです！子満貫 8,000点 (親 4,000点 / 子 2,000点)"
                        );
                            match_ctx.scores[0] += 8000;
                            for p in 1..4 {
                                if p == match_ctx.dealer_idx {
                                    match_ctx.scores[p] -= 4000;
                                } else {
                                    match_ctx.scores[p] -= 2000;
                                }
                            }
                        }
                        break 'game match_ctx.dealer_idx == 0;
                    }

                    if input.is_empty() || input.eq_ignore_ascii_case("auto") {
                        if let Some(idx) = hands[0]
                            .tiles()
                            .iter()
                            .rposition(|&t| t == best_discard_tile)
                        {
                            println!(
                                "👉 AI推奨 [{}] を自動選択しました。",
                                best_discard_tile.as_str()
                            );
                            break idx;
                        }
                    }

                    if let Ok(num) = input.parse::<usize>() {
                        if num >= 1 && num <= hands[0].tiles().len() {
                            break num - 1;
                        }
                    }

                    if let Some(idx) = hands[0].tiles().iter().position(|&t| t.as_str() == input) {
                        break idx;
                    }

                    println!(
                        "⚠️ 無効な入力です。1〜{}の番号か牌名を入力してください。",
                        hands[0].tiles().len()
                    );
                };

                discarded_tile = match hands[0].discard(chosen_index) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                println!(">> あなたが [{}] を打牌しました。", discarded_tile.as_str());
                rivers[0].push(discarded_tile);
                tracker.record_decision(turn_count, discarded_tile, &raw_evals);
                turn_count += 1;
            } else {
                // --- B. CPUの手番 ---
                let ctx = AnalysisContext {
                    turn_number: turn_count,
                    remaining_wall_tiles: wall.remaining(),
                    seat_wind: Some(player_seat_wind(current_turn)),
                    round_wind: Some(TileName::East),
                    dora_indicators: &[dora_indicator],
                    is_dealer: false,
                };

                // CPUの和了チェック（ツモ和了）
                if let Some(dt) = drawn_tile {
                    if check_agari(
                        &hands[current_turn],
                        dt,
                        true,
                        player_seat_wind(current_turn),
                        TileName::East,
                        riichi_declared[current_turn],
                    ) {
                        if current_turn == match_ctx.dealer_idx {
                            println!("\n💥 【ツモ！】{} がツモアガリしました！親満貫 12,000点 (各子 4,000点オール)", p_name);
                            match_ctx.scores[current_turn] += 12000;
                            for p in 0..4 {
                                if p != current_turn {
                                    match_ctx.scores[p] -= 4000;
                                }
                            }
                        } else {
                            println!("\n💥 【ツモ！】{} がツモアガリしました！子満貫 8,000点 (親 4,000点 / 子 2,000点)", p_name);
                            match_ctx.scores[current_turn] += 8000;
                            for p in 0..4 {
                                if p != current_turn {
                                    if p == match_ctx.dealer_idx {
                                        match_ctx.scores[p] -= 4000;
                                    } else {
                                        match_ctx.scores[p] -= 2000;
                                    }
                                }
                            }
                        }
                        break 'game current_turn == match_ctx.dealer_idx;
                    }
                }

                // CPUの打牌決定（期待値最善牌）
                let cpu_evals = evaluate_hand_discards(&hands[current_turn], None, &ctx);
                let cpu_discard = if !cpu_evals.is_empty() {
                    cpu_evals[0].discard_tile
                } else {
                    hands[current_turn].tiles()[hands[current_turn].tiles().len() - 1]
                };

                let idx = hands[current_turn]
                    .tiles()
                    .iter()
                    .rposition(|&t| t == cpu_discard)
                    .unwrap_or(0);

                discarded_tile = hands[current_turn].discard(idx).unwrap();
                rivers[current_turn].push(discarded_tile);

                // リーチ宣言判定（門前かつテンパイで未宣言ならリーチ）
                if hands[current_turn].open_melds.is_empty()
                    && !riichi_declared[current_turn]
                    && !cpu_evals.is_empty()
                    && cpu_evals[0].shanten_after == 0
                {
                    riichi_declared[current_turn] = true;
                    println!(
                        "⚡ {} が リーチ を宣言しました！ 打牌: [{}]",
                        p_name,
                        discarded_tile.as_str()
                    );
                } else {
                    println!(
                        ">> {} が [{}] を打牌しました。",
                        p_name,
                        discarded_tile.as_str()
                    );
                }
            }

            // ====================================================================
            // 打牌後の割り込み判定（ロン和了判定 ＆ 副露チー・ポン判定）
            // ====================================================================

            // 1. ロン和了判定
            // 1-1. 自家（プレイヤー）へのロンチェック
            if discarder != 0 {
                let can_ron = check_agari(
                    &hands[0],
                    discarded_tile,
                    false,
                    TileName::East,
                    TileName::East,
                    riichi_declared[0],
                );
                if can_ron {
                    println!(
                        "\n🎉 【ロン和了！】{} の捨てた [{}] でロン和了可能です！",
                        PLAYER_NAMES[discarder],
                        discarded_tile.as_str()
                    );
                    print!("ロン和了しますか？ (Y/n): ");
                    io::stdout().flush().unwrap();
                    let mut ron_in = String::new();
                    let _ = io::stdin().read_line(&mut ron_in);
                    if !ron_in.trim().eq_ignore_ascii_case("n") {
                        let is_dealer_win = match_ctx.dealer_idx == 0;
                        let is_dealer_deal = discarder == match_ctx.dealer_idx;
                        let score = if is_dealer_win || is_dealer_deal {
                            12000
                        } else {
                            8000
                        };
                        let role = if is_dealer_win {
                            "親満貫 12,000点"
                        } else {
                            "子満貫 8,000点"
                        };
                        println!(
                            "\n🎊 【ロン和了成立！】お見事です！{}（放銃: {}）",
                            role, PLAYER_NAMES[discarder]
                        );
                        match_ctx.scores[0] += score;
                        match_ctx.scores[discarder] -= score;
                        break 'game is_dealer_win;
                    }
                }
            }

            // 1-2. 他家CPUへのロンチェック
            for p in 1..4 {
                if p == discarder {
                    continue;
                }
                if check_agari(
                    &hands[p],
                    discarded_tile,
                    false,
                    player_seat_wind(p),
                    TileName::East,
                    riichi_declared[p],
                ) {
                    let is_dealer_win = p == match_ctx.dealer_idx;
                    let is_dealer_deal = discarder == match_ctx.dealer_idx;
                    let score = if is_dealer_win || is_dealer_deal {
                        12000
                    } else {
                        8000
                    };
                    let role = if is_dealer_win {
                        "親満貫 12,000点"
                    } else {
                        "子満貫 8,000点"
                    };
                    println!(
                        "\n💥 【ロン！】{} が {} の捨て牌 [{}] でロン和了しました！{}",
                        PLAYER_NAMES[p],
                        PLAYER_NAMES[discarder],
                        discarded_tile.as_str(),
                        role
                    );
                    match_ctx.scores[p] += score;
                    match_ctx.scores[discarder] -= score;
                    break 'game is_dealer_win;
                }
            }

            // 2. 副露（鳴き：チー・ポン）割り込み判定
            // 2-1. 自家（プレイヤー）の鳴き判定
            if discarder != 0 && !riichi_declared[0] {
                let is_kamicha = discarder == 3; // 上家(北家)からの打牌ならチー可能
                let call_ctx = AnalysisContext {
                    turn_number: turn_count,
                    remaining_wall_tiles: wall.remaining(),
                    seat_wind: Some(TileName::East),
                    round_wind: Some(TileName::East),
                    dora_indicators: &[dora_indicator],
                    is_dealer: true,
                };

                if let Some(advice) =
                    CallAdvisor::advise_call(&hands[0], discarded_tile, is_kamicha, &call_ctx)
                {
                    let call_choices: Vec<_> = advice
                        .choices
                        .iter()
                        .filter(|c| c.action != CallAction::Pass)
                        .cloned()
                        .collect();

                    if !call_choices.is_empty() {
                        println!("\n{}", "-".repeat(80));
                        let rec_label = match advice.recommendation {
                            CallRecommendation::AggressiveCall => "🔥【積極推奨】",
                            CallRecommendation::LeanCall => "✨【微有利】",
                            CallRecommendation::LeanPass => "🛡️【微スルー】",
                            CallRecommendation::StrictPass => "⛔【積極スルー】",
                        };
                        println!(
                            "🔔 【鳴きアドバイザー (Call Advisor)】 {} から [{}] が切られました！",
                            PLAYER_NAMES[discarder],
                            discarded_tile.as_str()
                        );
                        println!("  AI判定: {} {}", rec_label, advice.rationale);
                        println!("  選択肢:");
                        for (idx, c) in call_choices.iter().enumerate() {
                            let shanten_text = match c.post_shanten {
                                0 => "テンパイ".to_string(),
                                s => format!("{s}向聴"),
                            };
                            println!(
                            "    [{}] {} (向聴: {} | 受入: {:>2}枚 | 打点: {:>5.0}点 | EV: {:>+6.0}点)",
                            idx + 1,
                            c.action.label_ja(),
                            shanten_text,
                            c.post_acceptance,
                            c.estimated_score,
                            c.ev
                        );
                        }
                        println!("    [Enter / s] スルー (門前維持)");

                        print!(
                            "アクションを選択してください (1〜{}, s:スルー): ",
                            call_choices.len()
                        );
                        io::stdout().flush().unwrap();
                        let mut call_in = String::new();
                        let _ = io::stdin().read_line(&mut call_in);
                        let call_in = call_in.trim();

                        let mut executed_meld = None;

                        if let Ok(num) = call_in.parse::<usize>() {
                            if num >= 1 && num <= call_choices.len() {
                                executed_meld = call_choices[num - 1].meld;
                            }
                        } else if call_in.eq_ignore_ascii_case("chi") || call_in == "チー" {
                            if let Some(c) = call_choices
                                .iter()
                                .find(|c| matches!(c.action, CallAction::Chii(..)))
                            {
                                executed_meld = c.meld;
                            }
                        } else if call_in.eq_ignore_ascii_case("pon") || call_in == "ポン" {
                            if let Some(c) =
                                call_choices.iter().find(|c| c.action == CallAction::Pon)
                            {
                                executed_meld = c.meld;
                            }
                        }

                        if let Some(meld) = executed_meld {
                            if hands[0].call_meld(meld).is_ok() {
                                println!("👉 あなたが [{}] を実行しました！", format_meld(&meld));
                                rivers[discarder].pop();
                                current_turn = 0; // 手番が自家へ
                                skip_draw = true; // ツモをスキップして打牌へ
                                continue 'game;
                            }
                        } else {
                            println!(">> スルーしました。");
                        }
                    }
                }
            }

            // 2-2. CPUたちの鳴き判定（プレイヤーが鳴かなかった場合、他CPUが鳴くか判定）
            let mut cpu_called = false;
            for p in 1..4 {
                if p == discarder || riichi_declared[p] {
                    continue;
                }
                let is_kamicha = (discarder + 1) % 4 == p;
                let cpu_call_ctx = AnalysisContext {
                    turn_number: turn_count,
                    remaining_wall_tiles: wall.remaining(),
                    seat_wind: Some(player_seat_wind(p)),
                    round_wind: Some(TileName::East),
                    dora_indicators: &[dora_indicator],
                    is_dealer: false,
                };

                if let Some(advice) =
                    CallAdvisor::advise_call(&hands[p], discarded_tile, is_kamicha, &cpu_call_ctx)
                {
                    if advice.recommendation == CallRecommendation::AggressiveCall
                        && advice.best_action != CallAction::Pass
                    {
                        if let Some(best_choice) = advice
                            .choices
                            .iter()
                            .find(|c| c.action == advice.best_action)
                        {
                            if let Some(meld) = best_choice.meld {
                                if hands[p].call_meld(meld).is_ok() {
                                    println!(
                                        "\n⚡ {} が [{}] を宣言しました！ 副露: {}",
                                        PLAYER_NAMES[p],
                                        best_choice.action.label_ja(),
                                        format_meld(&meld)
                                    );
                                    rivers[discarder].pop();
                                    current_turn = p;
                                    skip_draw = true;
                                    cpu_called = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if cpu_called {
                continue 'game;
            }

            // 3. 通常ターン進行（誰も鳴かなかった場合）
            current_turn = (discarder + 1) % 4;
        };

        // 局終了後のスコア表示＆次局更新
        let ranks = match_ctx.current_ranks();
        println!("\n{}", "-".repeat(80));
        print!("📊 【局終了時スコア速報】: ");
        for p in 0..4 {
            print!(
                "{}位: {} ({}点)  ",
                ranks[p], PLAYER_NAMES[p], match_ctx.scores[p]
            );
        }
        println!("\n{}", "-".repeat(80));

        // トビ終了チェック
        if match_ctx.scores.iter().any(|&s| s < 0) {
            println!("⚡ トビ（持ち点マイナス）が発生したため、半荘終了となります。");
            break 'match_loop;
        }

        // オーラス終了判定
        if is_orasu && (!renchan || is_orasu_training) {
            println!("🏁 オーラス終了！全対局が完了しました。");
            break 'match_loop;
        }

        // 連荘または親移動
        if renchan {
            println!("🔁 親が連荘しました（本場 +1）。");
            match_ctx.honba += 1;
        } else {
            match_ctx.dealer_idx = (match_ctx.dealer_idx + 1) % 4;
            match_ctx.round_number += 1;
            match_ctx.honba = 0;
            println!("➡️ 親が移動し、次局へ進みます。");
        }

        print!("\nEnterキーで次局へ進みます (q:終了): ");
        io::stdout().flush().unwrap();
        let mut next_in = String::new();
        let _ = io::stdin().read_line(&mut next_in);
        if next_in.trim().eq_ignore_ascii_case("q") {
            println!("対局を終了します。");
            break 'match_loop;
        }
    }

    // 終局時順位・ポイント精算発表
    println!("\n{}", "=".repeat(80));
    println!("🏆 【対局結果・最終成績 (Mリーグ順位点精算: +50/+10/-10/-30, オカ内包)】");
    println!("{}", "=".repeat(80));
    let final_ranks = match_ctx.current_ranks();
    let mut result_list: Vec<(usize, usize, i32, f64)> = (0..4)
        .map(|p| {
            let r = final_ranks[p];
            let s = match_ctx.scores[p];
            let pt = match_ctx.rule.calculate_point(r, s);
            (r, p, s, pt)
        })
        .collect();
    result_list.sort_by_key(|item| item.0);

    for (rank, p, score, pt) in result_list {
        let medal = match rank {
            1 => "🥇",
            2 => "🥈",
            3 => "🥉",
            _ => "  ",
        };
        println!(
            "  {} 第{}位: {:<16} | 素点: {:>6}点 | 最終Pt: {:>+6.1}pt",
            medal, rank, PLAYER_NAMES[p], score, pt
        );
    }
    println!("{}", "=".repeat(80));

    // 局後学習振り返りレポート表示
    let report = tracker.generate_report();
    println!("{}", tracker.format_report(&report));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_agari_open_meld_without_yaku() {
        // オタ風ポン（南家が北をポン）で牌姿は完成しているが役がない場合
        let mut hand = Hand::new();
        // 副露: 北ポン
        hand.open_melds.push(Meld::Pon(TileName::North));
        // 手牌: 1m, 2m, 3m, 4p, 5p, 6p, 9s, 9s, 1s, 2s (計10枚)
        // 3s でロンすると牌姿は完成するが役なし
        hand.push(TileName::OneM);
        hand.push(TileName::TwoM);
        hand.push(TileName::ThreeM);
        hand.push(TileName::FourP);
        hand.push(TileName::FiveP);
        hand.push(TileName::SixP);
        hand.push(TileName::NineS);
        hand.push(TileName::NineS);
        hand.push(TileName::OneS);
        hand.push(TileName::TwoS);

        let can_ron = check_agari(
            &hand,
            TileName::ThreeS,
            false,
            TileName::South, // 自家南風（北はオタ風）
            TileName::East,  // 場風東
            false,
        );
        assert!(!can_ron, "役なし副露手でロンが成立してはならない");
    }

    #[test]
    fn test_check_agari_open_meld_with_yaku() {
        // タンヤオ副露手でロンが成立する場合
        let mut hand = Hand::new();
        // 副露: 2m-3m-4m チー
        hand.open_melds.push(Meld::Chii {
            called: TileName::TwoM,
            consumed: [TileName::ThreeM, TileName::FourM],
        });
        // 手牌: 2p, 3p, 4p, 5s, 6s, 7s, 8s, 8s, 2s, 3s
        hand.push(TileName::TwoP);
        hand.push(TileName::ThreeP);
        hand.push(TileName::FourP);
        hand.push(TileName::FiveS);
        hand.push(TileName::SixS);
        hand.push(TileName::SevenS);
        hand.push(TileName::EightS);
        hand.push(TileName::EightS);
        hand.push(TileName::TwoS);
        hand.push(TileName::ThreeS);

        let can_ron = check_agari(
            &hand,
            TileName::FourS,
            false,
            TileName::South,
            TileName::East,
            false,
        );
        assert!(can_ron, "タンヤオ役がある副露手はロンが成立すべき");
    }
}
