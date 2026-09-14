use mahjong::expectation::AnalysisContext;
use mahjong::hand::Hand;
use mahjong::placement_ev::{
    calculate_orasu_conditions, evaluate_hand_discards_with_placement, MatchContext, RuleConfig,
};
use mahjong::tile::TileName;

#[test]
fn test_rule_config_points() {
    let rule = RuleConfig::mleague();

    // 1位: 45,000点 -> (45-30) + 50(ウマ・オカ内包) = 65.0 pt
    let pt1 = rule.calculate_point(1, 45000);
    assert!((pt1 - 65.0).abs() < 1e-6);

    // 2位: 28,000点 -> (28-30) + 10(ウマ) = 8.0 pt
    let pt2 = rule.calculate_point(2, 28000);
    assert!((pt2 - 8.0).abs() < 1e-6);

    // 3位: 17,000点 -> (17-30) - 10(ウマ) = -23.0 pt
    let pt3 = rule.calculate_point(3, 17000);
    assert!((pt3 - (-23.0)).abs() < 1e-6);

    // 4位: 10,000点 -> (10-30) - 30(ウマ) = -50.0 pt
    let pt4 = rule.calculate_point(4, 10000);
    assert!((pt4 - (-50.0)).abs() < 1e-6);

    // 4名の合計ポイントが 0.0 pt になることの検証
    let total_pt = pt1 + pt2 + pt3 + pt4;
    assert!(
        total_pt.abs() < 1e-6,
        "4名のポイント合計は 0 になるべき: {}",
        total_pt
    );
}

#[test]
fn test_match_context_ranks() {
    let ctx = MatchContext {
        scores: [32000, 25000, 18000, 25000],
        round_wind: TileName::East,
        round_number: 1,
        honba: 0,
        riichi_sticks: 0,
        dealer_idx: 0,
        rule: RuleConfig::default(),
    };

    let ranks = ctx.current_ranks();
    // 0: 32000点 (1位)
    // 1: 25000点 (同点だが 1番席 > 3番席 なので 2位)
    // 3: 25000点 (3位)
    // 2: 18000点 (4位)
    assert_eq!(ranks[0], 1);
    assert_eq!(ranks[1], 2);
    assert_eq!(ranks[3], 3);
    assert_eq!(ranks[2], 4);
}

#[test]
fn test_orasu_win_conditions() {
    // 南4局（オーラス）、自家（0: 東家/親 28,000点）、対面（2: 西家 33,000点、差 5,000点）
    let ctx = MatchContext {
        scores: [28000, 20000, 33000, 19000],
        round_wind: TileName::South,
        round_number: 4,
        honba: 0,
        riichi_sticks: 0,
        dealer_idx: 0,
        rule: RuleConfig::default(),
    };

    assert!(ctx.is_orasu());
    let conds = calculate_orasu_conditions(&ctx, 0);
    assert_eq!(conds.len(), 1); // 2位なので1位（対面）への逆転条件のみ

    let cond1 = &conds[0];
    assert_eq!(cond1.target_rank, 1);
    assert_eq!(cond1.target_player, 2);
    assert_eq!(cond1.diff, 5000);

    // 直撃条件: 親の直撃で5000点差を縮める -> 2900点直撃で自家+2900、相手-2900で5800差詰まり逆転
    assert!(cond1.ron_direct_req.is_some());
    let direct = cond1.ron_direct_req.unwrap();
    assert!(2 * direct >= 5000);

    // 脇出和了条件: 相手動かず自分だけ加点 -> 5800点以上の和了が必要
    assert!(cond1.ron_other_req.is_some());
    let other = cond1.ron_other_req.unwrap();
    assert!(other >= 5000);
}

#[test]
fn test_top_player_orasu_summary() {
    // 南4局、自家（0: 38,000点 トップ目）
    let ctx = MatchContext {
        scores: [38000, 24000, 20000, 18000],
        round_wind: TileName::South,
        round_number: 4,
        honba: 0,
        riichi_sticks: 0,
        dealer_idx: 3,
        rule: RuleConfig::default(),
    };

    let conds = calculate_orasu_conditions(&ctx, 0);
    assert_eq!(conds.len(), 1);
    assert_eq!(conds[0].target_rank, 1);
    assert!(conds[0].summary.contains("トップ目"));
}

#[test]
fn test_evaluate_hand_discards_with_placement() {
    let mut hand = Hand::new();
    // テンパイ形: 1m, 2m, 3m, 4p, 5p, 6p, 7s, 8s, 9s, 1s, 1s, 2s, 3s, 5z(白)
    hand.push(TileName::OneM);
    hand.push(TileName::TwoM);
    hand.push(TileName::ThreeM);
    hand.push(TileName::FourP);
    hand.push(TileName::FiveP);
    hand.push(TileName::SixP);
    hand.push(TileName::SevenS);
    hand.push(TileName::EightS);
    hand.push(TileName::NineS);
    hand.push(TileName::OneS);
    hand.push(TileName::OneS);
    hand.push(TileName::TwoS);
    hand.push(TileName::ThreeS);
    hand.push(TileName::White);

    let analysis_ctx = AnalysisContext {
        turn_number: 5,
        remaining_wall_tiles: 50,
        seat_wind: Some(TileName::East),
        round_wind: Some(TileName::East),
        dora_indicators: &[TileName::NineM],
        is_dealer: true,
        ..Default::default()
    };

    let match_ctx = MatchContext {
        scores: [25000, 25000, 25000, 25000],
        round_wind: TileName::East,
        round_number: 1,
        honba: 0,
        riichi_sticks: 0,
        dealer_idx: 0,
        rule: RuleConfig::default(),
    };

    let evals = evaluate_hand_discards_with_placement(&hand, None, &analysis_ctx, &match_ctx, 0);
    assert!(!evals.is_empty());

    let best = &evals[0];
    assert_eq!(best.base.discard_tile, TileName::White);
    assert!(best.placement_ev > 0.0);
    assert!(best.rank_probabilities[0] > 0.3);
}

#[test]
fn test_placement_ev_dealer_riichi_defense() {
    // Issue #79 検証: オーラスで親リーチが入っている局面での Placement EV
    let mut hand = Hand::new();
    // 手牌: 現物 4m と 無筋 5m を含む手
    for &t in &[
        TileName::FourM, // 親の現物
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

    let river_dealer = vec![TileName::FourM, TileName::East];
    let rivers: [&[TileName]; 4] = [&[], &river_dealer, &[], &[]];

    // 南4局（オーラス）、下家（1番）が親でリーチ
    // 自家（0番）は 2位 27,000点、親は 3位 23,000点、ラス目は 18,000点
    // 親に満貫（12,000点）放銃すると一撃でラス落ち！
    let analysis_ctx = AnalysisContext {
        turn_number: 10,
        remaining_wall_tiles: 35,
        seat_wind: Some(TileName::North),
        round_wind: Some(TileName::South),
        dora_indicators: &[TileName::NineM],
        is_dealer: false,
        riichi_status: [false, true, false, false], // 下家（親）リーチ
        player_rivers: &rivers,
        player_is_dealer: [false, true, false, false],
        ..Default::default()
    };

    let match_ctx = MatchContext {
        scores: [27000, 23000, 32000, 18000],
        round_wind: TileName::South,
        round_number: 4,
        honba: 0,
        riichi_sticks: 1,
        dealer_idx: 1,
        rule: RuleConfig::default(),
    };

    let evals = evaluate_hand_discards_with_placement(&hand, None, &analysis_ctx, &match_ctx, 0);

    let eval_4m = evals
        .iter()
        .find(|e| e.base.discard_tile == TileName::FourM)
        .unwrap();
    let eval_5m = evals
        .iter()
        .find(|e| e.base.discard_tile == TileName::FiveM)
        .unwrap();

    // 親満貫放銃のラス落ちリスクが反映され、現物4mのPlacement EVが無筋5mを圧倒すること！
    assert!(
        eval_4m.placement_ev > eval_5m.placement_ev,
        "Genbutsu 4m Placement EV ({}) must be significantly higher than dangerous 5m ({})",
        eval_4m.placement_ev,
        eval_5m.placement_ev
    );
    // 4位率（ラス率）も無筋5mの方が圧倒的に高くなること
    assert!(
        eval_5m.rank_probabilities[3] > eval_4m.rank_probabilities[3],
        "4th place probability of 5m ({}) must be higher than 4m ({})",
        eval_5m.rank_probabilities[3],
        eval_4m.rank_probabilities[3]
    );
}
