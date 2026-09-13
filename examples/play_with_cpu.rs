use std::io::{self, Write};

use mahjong::dora::indicator_to_dora;
use mahjong::expectation::{evaluate_hand_discards, AnalysisContext};
use mahjong::explanation::Explainer;
use mahjong::hand::Hand;
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

fn main() {
    println!("================================================================================");
    println!("🀄 リアルタイム期待値ヒント＆意思決定モデル 4人対局CPU学習モード 🀄");
    println!("================================================================================");
    println!("ルール:");
    println!("  ・東1局 0本場、あなた（東家/起家）vs CPU 3名");
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

    'game: loop {
        let p_name = PLAYER_NAMES[current_turn];

        // 1. ツモ
        let drawn = match wall.draw() {
            Some(t) => t,
            None => {
                println!("\n【流局】山牌が尽きました。局終了です。");
                break 'game;
            }
        };
        hands[current_turn].push(drawn);

        // --- A. 自家（プレイヤー）の手番 ---
        if current_turn == 0 {
            let remaining_wall = wall.remaining();
            println!("\n{}", "=".repeat(80));
            println!(
                "【東1局 0本場】 巡目: {}巡目 (親/自家) | 残り山牌: {}枚 | ドラ表示牌: [{}] (ドラ: [{}])",
                turn_count, remaining_wall, dora_indicator.as_str(), dora_tile.as_str()
            );

            // 他家の河とリーチ状況を表示
            println!("{}", "-".repeat(80));
            for p in 1..4 {
                let riichi_str = if riichi_declared[p] {
                    " 🚨[リーチ中]"
                } else {
                    ""
                };
                let river_tiles: Vec<&str> = rivers[p].iter().map(|t| t.as_str()).collect();
                println!(
                    "{:<16}{}: {}",
                    PLAYER_NAMES[p],
                    riichi_str,
                    river_tiles.join(" ")
                );
            }
            let my_river_tiles: Vec<&str> = rivers[0].iter().map(|t| t.as_str()).collect();
            println!("{:<16}  : {}", "あなたの河", my_river_tiles.join(" "));
            println!("{}", "-".repeat(80));

            // 手牌表示
            print!("あなたの手牌: ");
            for (i, &t) in hands[0].tiles().iter().enumerate() {
                if i == hands[0].tiles().len() - 1 {
                    print!("  ツモ: [{}:{}]", i + 1, t.as_str());
                } else {
                    print!("[{}:{}] ", i + 1, t.as_str());
                }
            }
            println!();

            // 和了判定
            let current_shanten = calculate_shanten(&hands[0]);
            let is_agari = current_shanten.min_shanten < 0;
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
                    "何を切りますか？ (1〜14, 牌名, Enter/autoで最善[{}], q:中断): ",
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

                println!("⚠️ 無効な入力です。1〜14の番号か牌名を入力してください。");
            };

            if let Ok(discarded) = hands[0].discard(chosen_index) {
                println!(">> あなたが [{}] を打牌しました。", discarded.as_str());
                rivers[0].push(discarded);
                tracker.record_decision(turn_count, discarded, &evaluations);
            }

            turn_count += 1;
        } else {
            // --- B. CPUの手番 ---
            let ctx = AnalysisContext {
                turn_number: turn_count,
                remaining_wall_tiles: wall.remaining(),
                seat_wind: Some(TileName::East),
                round_wind: Some(TileName::East),
                dora_indicators: &[dora_indicator],
                is_dealer: false,
            };

            // CPUの和了チェック
            let cpu_shanten = calculate_shanten(&hands[current_turn]);
            if cpu_shanten.min_shanten < 0 {
                println!("\n💥 【ツモ！】{} がツモアガリしました！", p_name);
                break 'game;
            }

            // CPUの打牌決定（期待値最善牌）
            let cpu_evals = evaluate_hand_discards(&hands[current_turn], None, &ctx);
            let cpu_discard = if !cpu_evals.is_empty() {
                cpu_evals[0].discard_tile
            } else {
                hands[current_turn].tiles()[hands[current_turn].tiles().len() - 1]
            };

            if let Some(idx) = hands[current_turn]
                .tiles()
                .iter()
                .rposition(|&t| t == cpu_discard)
            {
                if let Ok(discarded) = hands[current_turn].discard(idx) {
                    rivers[current_turn].push(discarded);

                    // リーチ宣言判定（テンパイかつ門前かつまだリーチしていない場合、確率でリーチ）
                    if !riichi_declared[current_turn]
                        && !cpu_evals.is_empty()
                        && cpu_evals[0].shanten_after == 0
                    {
                        riichi_declared[current_turn] = true;
                        println!(
                            "⚡ {} が リーチ を宣言しました！ 打牌: [{}]",
                            p_name,
                            discarded.as_str()
                        );
                    }

                    // プレイヤーへのロン和了チェック
                    let mut test_hand = hands[0].clone();
                    test_hand.push(discarded);
                    let ron_shanten = calculate_shanten(&test_hand);
                    if ron_shanten.min_shanten < 0 {
                        println!(
                            "\n🎉 【ロン和了！】{} の捨てた [{}] でロン和了可能です！",
                            p_name,
                            discarded.as_str()
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
            }
        }

        // 次のプレイヤーへ
        current_turn = (current_turn + 1) % 4;
    }

    // 局後学習振り返りレポート表示
    let report = tracker.generate_report();
    println!("{}", tracker.format_report(&report));
}
