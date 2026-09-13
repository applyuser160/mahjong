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

fn main() {
    println!("================================================================================");
    println!("🀄 リアルタイム期待値ヒント＆意思決定モデル 麻雀学習対局 (一人麻雀モード) 🀄");
    println!("================================================================================");
    println!("操作方法:");
    println!("  ・手牌番号 (1〜14) または 牌名 (例: 9p, 1m, 東) を入力して打牌");
    println!("  ・'auto' または Enter のみ: AI推奨の最善打牌（1位）を自動選択");
    println!("  ・'q': 終了（局後学習レポートを表示して終了）\n");

    let mut wall = Wall::new();
    let mut rng = StdRng::from_entropy();
    wall.shuffle(&mut rng);

    // ドラ表示牌 (王牌の最初の1枚)
    let dora_indicator = wall.tiles()[135];
    let dora_tile = indicator_to_dora(dora_indicator);

    let mut user_hand = Hand::new();
    // 配牌 13枚
    for _ in 0..13 {
        if let Some(t) = wall.draw() {
            user_hand.push(t);
        }
    }

    let mut tracker = ReviewTracker::new();
    let mut turn_count = 1;

    'game: loop {
        // 1. ツモ
        let drawn = match wall.draw() {
            Some(t) => t,
            None => {
                println!("\n【流局】山牌がなくなりました。対局終了です。");
                break 'game;
            }
        };
        user_hand.push(drawn);

        // 2. 局面状況
        let remaining_wall = wall.remaining();
        println!("\n{}", "=".repeat(80));
        println!(
            "【東1局 0本場】 巡目: {}巡目 (親/東家)   残りツモ: {}枚",
            turn_count, remaining_wall
        );
        println!(
            "ドラ表示牌: [{}]  (ドラ: [{}])",
            dora_indicator.as_str(),
            dora_tile.as_str()
        );
        println!("{}", "-".repeat(80));

        // 3. 手牌の表示
        print!("手牌: ");
        for (i, &t) in user_hand.tiles().iter().enumerate() {
            if i == 13 {
                print!("  ツモ: [{}:{}]", i + 1, t.as_str());
            } else {
                print!("[{}:{}] ", i + 1, t.as_str());
            }
        }
        println!();

        // 4. 現在の向聴数判定
        let current_shanten = calculate_shanten(&user_hand);
        let is_agari = current_shanten.min_shanten < 0;
        if is_agari {
            println!("\n🎉 【和了可能】ツモ和了の形が完成しています！ ('tsumo' または 'ツモ' で和了終了できます)");
        }

        // 5. 期待値計算 & 要因モデル
        let ctx = AnalysisContext {
            turn_number: turn_count,
            remaining_wall_tiles: remaining_wall,
            seat_wind: Some(TileName::East),
            round_wind: Some(TileName::East),
            dora_indicators: &[dora_indicator],
            is_dealer: true,
        };

        let evaluations = evaluate_hand_discards(&user_hand, None, &ctx);

        println!("{}", "-".repeat(80));
        println!("💡 【リアルタイム期待値ヒント (AI Advisor)】");

        for (rank, ev) in evaluations.iter().take(3).enumerate() {
            let shanten_text = match ev.shanten_after {
                0 => "テンパイ".to_string(),
                s => format!("{s}向聴"),
            };
            let yaku_str = if !ev.value.primary_yaku.is_empty() {
                format!("  役: {}", ev.value.primary_yaku.join(", "))
            } else {
                String::new()
            };

            let waits_str = if !ev.speed.accepted_tiles.is_empty() {
                let wait_names: Vec<&str> =
                    ev.speed.accepted_tiles.iter().map(|t| t.as_str()).collect();
                format!("  受入: [{}]", wait_names.join(","))
            } else {
                String::new()
            };

            let mark = if rank == 0 { "★最善" } else { "  候補" };
            println!(
                "  {} [{:2}] 打[{:2}] -> {:>4} | EV: {:>5.0}点 | 残り:{:>2}枚 | 想定:{:>5.0}点 ({:.1}翻){}{}",
                mark,
                rank + 1,
                ev.discard_tile.as_str(),
                shanten_text,
                ev.ev,
                ev.speed.remaining_count,
                ev.value.expected_score,
                ev.value.expected_han,
                waits_str,
                yaku_str,
            );
        }

        // 6. 意思決定理由 (Rationale)
        let rationale = Explainer::generate_rationale(&evaluations);
        println!("\n📖 【なぜその打牌なのか (AIの判断根拠)】");
        println!("  {rationale}");
        println!("{}", "=".repeat(80));

        // 7. ユーザーの打牌選択
        let best_discard_tile = evaluations[0].discard_tile;
        let chosen_index = loop {
            print!(
                "何を切りますか？ (1〜14, 牌名, Enter/autoでAI最善[{}], q:終了): ",
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
                println!("\n🎉 【ツモ和了】見事なアガリです！おめでとうございます！");
                break 'game;
            }

            if input.is_empty() || input.eq_ignore_ascii_case("auto") {
                // 推奨牌のインデックスを特定
                if let Some(idx) = user_hand
                    .tiles()
                    .iter()
                    .rposition(|&t| t == best_discard_tile)
                {
                    println!(
                        "👉 AI推奨 [{}] ({}番) を自動選択しました。",
                        best_discard_tile.as_str(),
                        idx + 1
                    );
                    break idx;
                }
            }

            // 数字入力 (1..=14)
            if let Ok(num) = input.parse::<usize>() {
                if num >= 1 && num <= user_hand.tiles().len() {
                    break num - 1;
                }
            }

            // 牌名入力 (例: "9p", "東")
            if let Some(idx) = user_hand.tiles().iter().position(|&t| t.as_str() == input) {
                break idx;
            }

            println!("⚠️ 無効な入力です。1〜14の番号か牌名を入力してください。");
        };

        // 打牌実行
        if let Ok(discarded) = user_hand.discard(chosen_index) {
            println!(">> [{}] を打牌しました。", discarded.as_str());
            tracker.record_decision(turn_count, discarded, &evaluations);
        }

        turn_count += 1;
        // 他家3人の簡易ツモ切りシミュレーション（山牌を3枚消費）
        for _ in 0..3 {
            wall.draw();
        }
    }

    // 局後学習振り返りレポート表示
    let report = tracker.generate_report();
    println!("{}", tracker.format_report(&report));
}
