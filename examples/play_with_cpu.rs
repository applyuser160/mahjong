use std::io::{self, Write};

use mahjong::call_advisor::{CallAction, CallAdvisor, CallRecommendation};
use mahjong::dora::indicator_to_dora;
use mahjong::expectation::{evaluate_hand_discards, AnalysisContext};
use mahjong::explanation::Explainer;
use mahjong::hand::{Hand, Meld};
use mahjong::review::ReviewTracker;
use mahjong::shanten::calculate_shanten;
use mahjong::tile::TileName;
use mahjong::wall::Wall;
use rand::rngs::StdRng;
use rand::SeedableRng;

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

fn main() {
    println!("================================================================================");
    println!("🀄 リアルタイム期待値ヒント＆意思決定モデル 4人対局CPU学習モード (副露対応) 🀄");
    println!("================================================================================");
    println!("ルール:");
    println!("  ・東1局 0本場、あなた（東家/起家）vs CPU 3名");
    println!("  ・他家の打牌に対してリアルタイムにチー・ポンの期待値（EV）を判定・提示");
    println!("  ・各手番でAIが打牌期待値・受入・打点・放銃リスクをリアルタイム算出");
    println!("  ・他家のリーチや仕掛けに対する守備・押し引き判断も学習可能");
    println!("  ・終局時にあなたの打牌精度スコア・総EV損失・悪手ワースト診断を表示\n");

    let mut wall = Wall::new();
    let mut rng = StdRng::from_entropy();
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

    let mut tracker = ReviewTracker::new();
    let mut turn_count = 1;
    let mut current_turn = 0; // 0: 自家(東), 1: 南, 2: 西, 3: 北
    let mut skip_draw = false; // 副露直後の手番はツモをスキップ

    'game: loop {
        let p_name = PLAYER_NAMES[current_turn];

        // 1. ツモ処理（副露直後でない場合のみ山から引く）
        let drawn_tile = if !skip_draw {
            let drawn = match wall.draw() {
                Some(t) => t,
                None => {
                    println!("\n【流局】山牌が尽きました。局終了です。");
                    break 'game;
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
            println!("\n{}", "=".repeat(80));
            println!(
                "【東1局 0本場】 巡目: {}巡目 (親/自家) | 残り山牌: {}枚 | ドラ表示牌: [{}] (ドラ: [{}])",
                turn_count, remaining_wall, dora_indicator.as_str(), dora_tile.as_str()
            );

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
                let meld_texts: Vec<String> = hands[0].open_melds.iter().map(format_meld).collect();
                print!("  副露: {}", meld_texts.join(" "));
            }
            println!();

            // 和了判定（ツモ和了）
            let current_shanten = calculate_shanten(&hands[0]);
            let is_agari = drawn_tile.is_some() && current_shanten.min_shanten < 0;
            if is_agari {
                println!("\n🎉 【ツモ和了可能】完成形です！ ('tsumo' または 'ツモ' で和了可能)");
            }

            // 期待値計算 & リアルタイムヒント
            let ctx = AnalysisContext {
                turn_number: turn_count,
                remaining_wall_tiles: remaining_wall,
                seat_wind: Some(TileName::East),
                round_wind: Some(TileName::East),
                dora_indicators: &[dora_indicator],
                is_dealer: true,
            };

            // 他家の現物（安全牌）を考慮して危険度を補正
            let mut evaluations = evaluate_hand_discards(&hands[0], None, &ctx);
            let any_riichi = riichi_declared[1] || riichi_declared[2] || riichi_declared[3];

            if any_riichi {
                for ev in &mut evaluations {
                    // リーチ者の河にある牌は現物（100%安全）
                    let is_genbutsu = (1..4)
                        .filter(|&p| riichi_declared[p])
                        .all(|p| rivers[p].contains(&ev.discard_tile));

                    if is_genbutsu {
                        ev.safety.risk_score = 0.0;
                        ev.safety.is_safe = true;
                    } else {
                        // 筋・字牌・端牌などに応じた危険度
                        let d = ev.discard_tile;
                        let is_honor = d as usize >= 28;
                        let rank = (d as usize - 1) % 9 + 1;
                        let is_terminal = rank == 1 || rank == 9;

                        if is_honor || is_terminal {
                            ev.safety.risk_score = 0.15;
                        } else {
                            ev.safety.risk_score = 0.45; // 無筋中張牌は危険
                        }
                        ev.safety.is_safe = false;
                    }
                    // EVを再計算: EV = 和了確率 * 想定打点 - 放銃危険度 * 8000点
                    let win_prob = ev.speed.win_probability;
                    let value = ev.value.expected_score;
                    ev.ev = win_prob * value - ev.safety.risk_score * 8000.0;
                }

                // EV降順で再ソート
                evaluations
                    .sort_by(|a, b| b.ev.partial_cmp(&a.ev).unwrap_or(std::cmp::Ordering::Equal));
            }

            println!("{}", "-".repeat(80));
            println!("💡 【リアルタイム期待値ヒント (AI Advisor)】");
            if any_riichi {
                println!(
                    "  ⚠️ 他家リーチが入っています！放銃失点リスクを反映した押し引き評価です。"
                );
            }

            for (rank, ev) in evaluations.iter().take(4).enumerate() {
                let shanten_text = match ev.shanten_after {
                    0 => "テンパイ".to_string(),
                    s => format!("{s}向聴"),
                };
                let safety_text = if ev.safety.risk_score == 0.0 {
                    "🛡️ 現物(安全)"
                } else if ev.safety.risk_score < 0.25 {
                    "🟡 比較的安全"
                } else {
                    "🔴 危険牌"
                };

                let mark = if rank == 0 { "★最善" } else { "  候補" };
                println!(
                    "  {} [{:2}] 打[{:2}] -> {:>4} | EV: {:>+6.0}点 | 受入:{:>2}枚 | 打点:{:>5.0}点 | 守備: {}",
                    mark,
                    rank + 1,
                    ev.discard_tile.as_str(),
                    shanten_text,
                    ev.ev,
                    ev.speed.remaining_count,
                    ev.value.expected_score,
                    safety_text
                );
            }

            let rationale = Explainer::generate_rationale(&evaluations);
            println!("\n📖 【なぜその打牌なのか (AIの判断根拠)】");
            println!("  {rationale}");
            println!("{}", "=".repeat(80));

            // 打牌入力
            let best_discard_tile = evaluations[0].discard_tile;
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
                    break 'game;
                }

                if is_agari && (input.eq_ignore_ascii_case("tsumo") || input == "ツモ") {
                    println!("\n🎉 【ツモ和了！】見事なアガリです！対局終了！");
                    break 'game;
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
            tracker.record_decision(turn_count, discarded_tile, &evaluations);
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
            if drawn_tile.is_some() {
                let cpu_shanten = calculate_shanten(&hands[current_turn]);
                if cpu_shanten.min_shanten < 0 {
                    println!("\n💥 【ツモ！】{} がツモアガリしました！", p_name);
                    break 'game;
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
            let mut test_hand = hands[0].clone();
            test_hand.push(discarded_tile);
            let ron_shanten = calculate_shanten(&test_hand);
            if ron_shanten.min_shanten < 0 {
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
                    println!("\n🎊 【ロン和了成立！】お見事です！局終了となります。");
                    break 'game;
                }
            }
        }

        // 1-2. 他家CPUへのロンチェック
        for p in 1..4 {
            if p == discarder {
                continue;
            }
            let mut test_hand = hands[p].clone();
            test_hand.push(discarded_tile);
            if calculate_shanten(&test_hand).min_shanten < 0 {
                println!(
                    "\n💥 【ロン！】{} が {} の捨て牌 [{}] でロン和了しました！",
                    PLAYER_NAMES[p],
                    PLAYER_NAMES[discarder],
                    discarded_tile.as_str()
                );
                break 'game;
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
                        if let Some(c) = call_choices.iter().find(|c| c.action == CallAction::Pon) {
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
    }

    // 局後学習振り返りレポート表示
    let report = tracker.generate_report();
    println!("{}", tracker.format_report(&report));
}
