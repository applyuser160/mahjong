use std::io::{self, Write};

use mahjong::drill::{DrillEngine, DrillSession};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() {
    println!("================================================================================");
    println!("🎯 麻雀 期待値判断・何切るドリル＆クイズ (What-to-Discard Drill) 🎯");
    println!("================================================================================");
    println!("説明:");
    println!(
        "  ・14枚の手牌とドラ・局面が表示されます。「何を切るのが最善か」を選択してください。"
    );
    println!("  ・手牌番号 (1〜14) または 牌名 (例: 9p, 1m, 東) を入力します。");
    println!("  ・回答直後に正誤・EV差・なぜその打牌が最善なのかの解説がフィードバックされます。");
    println!("  ・'q': ドリルを終了し、総合成績レポートを表示します。\n");

    let mut rng = StdRng::from_entropy();
    let mut session = DrillSession::new();
    let mut problem_count = 1;

    loop {
        println!("{}", "=".repeat(80));
        println!("【第 {} 問】", problem_count);

        // テンパイまたは1向聴・2向聴からランダム出題
        let target_shanten = match problem_count % 3 {
            1 => Some(0), // テンパイ問題
            2 => Some(1), // 1向聴問題
            _ => None,    // ランダム問題
        };

        let problem = match DrillEngine::generate_problem(target_shanten, 50, &mut rng) {
            Some(p) => p,
            None => {
                println!("問題の生成に失敗しました。再試行します...");
                continue;
            }
        };

        println!(
            "想定巡目: {}巡目 | ドラ表示牌: [{}]",
            problem.turn_number,
            problem.dora_indicator.as_str()
        );
        println!("{}", "-".repeat(80));

        // 手牌の表示
        print!("手牌: ");
        for (i, &t) in problem.hand.tiles().iter().enumerate() {
            if i == 13 {
                print!("  ツモ: [{}:{}]", i + 1, t.as_str());
            } else {
                print!("[{}:{}] ", i + 1, t.as_str());
            }
        }
        println!("\n{}", "-".repeat(80));

        // 解答受付
        let chosen_tile = loop {
            print!("何を切りますか？ (1〜14, 牌名, q:終了): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                continue;
            }
            let input = input.trim();

            if input.eq_ignore_ascii_case("q") {
                println!("\nドリルを終了します。");
                let report = session.generate_report();
                println!("{}", session.format_report(&report));
                return;
            }

            // 数字入力 (1..=14)
            if let Ok(num) = input.parse::<usize>() {
                if num >= 1 && num <= problem.hand.tiles().len() {
                    break problem.hand.tiles()[num - 1];
                }
            }

            // 牌名入力 (例: "9p", "東")
            if let Some(&t) = problem.hand.tiles().iter().find(|&&t| t.as_str() == input) {
                break t;
            }

            println!("⚠️ 1〜14の番号か、手牌にある牌名を入力してください。");
        };

        // 採点とフィードバック
        let answer_result = DrillEngine::evaluate_answer(&problem, chosen_tile);
        println!("\n{}", answer_result.feedback);

        // 候補ランキング上位の表示
        println!("\n📊 【上位打牌候補の比較】");
        for (rank, cand) in problem.candidates.iter().take(3).enumerate() {
            let shanten_text = match cand.shanten_after {
                0 => "テンパイ".to_string(),
                s => format!("{s}向聴"),
            };
            let mark = if rank == 0 { "★最善" } else { "  次善" };
            println!(
                "  {} 打[{:2}] -> {:>4} | EV: {:>5.0}点 | 受入:{:>2}枚 | 想定:{:>5.0}点",
                mark,
                cand.discard_tile.as_str(),
                shanten_text,
                cand.ev,
                cand.speed.remaining_count,
                cand.value.expected_score
            );
        }

        session.record_result(answer_result);
        problem_count += 1;

        print!("\n次の問題へ進むには Enter を押してください (q:終了): ");
        io::stdout().flush().unwrap();
        let mut next_in = String::new();
        let _ = io::stdin().read_line(&mut next_in);
        if next_in.trim().eq_ignore_ascii_case("q") {
            println!("\nドリルを終了します。");
            let report = session.generate_report();
            println!("{}", session.format_report(&report));
            return;
        }
    }
}
