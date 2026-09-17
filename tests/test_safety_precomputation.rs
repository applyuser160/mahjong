use mahjong::expectation::{
    evaluate_tile_safety, evaluate_tile_safety_with_features, AnalysisContext, SafetyFeatures,
};
use mahjong::tile::TileName;

#[test]
fn test_safety_equivalence_no_riichi() {
    let ctx = AnalysisContext {
        turn_number: 6,
        target_player: 0,
        riichi_status: [false; 4],
        ..Default::default()
    };
    let visible_counts = [1u8; 35];
    let features = SafetyFeatures::from_context(&ctx);

    for i in 1..=34 {
        let tile = TileName::from_usize(i);
        let expected = evaluate_tile_safety(tile, &ctx, &visible_counts);
        let actual = evaluate_tile_safety_with_features(tile, &ctx, &visible_counts, &features);
        assert_eq!(
            expected, actual,
            "Tile {:?} safety mismatch in no_riichi case",
            tile
        );
    }
}

#[test]
fn test_safety_equivalence_single_riichi_with_river() {
    use TileName::*;
    let river1 = vec![OneM, TwoM, ThreeM, FourM, East, South, FiveP, NineS];
    let player_rivers: Vec<&[TileName]> = vec![&[], &river1, &[], &[]];

    let ctx = AnalysisContext {
        turn_number: 10,
        target_player: 0,
        riichi_status: [false, true, false, false],
        player_rivers: &player_rivers,
        ..Default::default()
    };
    let mut visible_counts = [0u8; 35];
    for &t in &river1 {
        visible_counts[t as usize] += 1;
    }
    let features = SafetyFeatures::from_context(&ctx);

    for i in 1..=34 {
        let tile = TileName::from_usize(i);
        let expected = evaluate_tile_safety(tile, &ctx, &visible_counts);
        let actual = evaluate_tile_safety_with_features(tile, &ctx, &visible_counts, &features);
        assert_eq!(
            expected, actual,
            "Tile {:?} safety mismatch in single_riichi case",
            tile
        );
    }
}

#[test]
fn test_safety_equivalence_multiple_riichi() {
    use TileName::*;
    let river1 = vec![OneM, FourM, SevenM, East, White];
    let river2 = vec![TwoP, FiveP, EightP, East, Green];
    let river3 = vec![OneM, TwoP, NineS, West];
    let player_rivers: Vec<&[TileName]> = vec![&[], &river1, &river2, &river3];

    let ctx = AnalysisContext {
        turn_number: 14,
        target_player: 0,
        riichi_status: [false, true, true, true],
        player_rivers: &player_rivers,
        ..Default::default()
    };
    let mut visible_counts = [0u8; 35];
    for river in [&river1, &river2, &river3] {
        for &t in river {
            visible_counts[t as usize] = (visible_counts[t as usize] + 1).min(4);
        }
    }
    let features = SafetyFeatures::from_context(&ctx);

    for i in 1..=34 {
        let tile = TileName::from_usize(i);
        let expected = evaluate_tile_safety(tile, &ctx, &visible_counts);
        let actual = evaluate_tile_safety_with_features(tile, &ctx, &visible_counts, &features);
        assert_eq!(
            expected, actual,
            "Tile {:?} safety mismatch in multiple_riichi case",
            tile
        );
    }
}

#[test]
fn test_safety_equivalence_missing_river_slice() {
    use TileName::*;
    let river1 = vec![OneM, FourM, SevenM];
    // player_rivers の長さが 2 しかない（プレイヤー2, 3 の河スライスが欠損）
    let player_rivers: Vec<&[TileName]> = vec![&[], &river1];

    let ctx = AnalysisContext {
        turn_number: 12,
        target_player: 0,
        riichi_status: [false, true, true, false], // player 2 もリーチだが河欠損
        player_rivers: &player_rivers,
        ..Default::default()
    };
    let visible_counts = [1u8; 35];
    let features = SafetyFeatures::from_context(&ctx);

    for i in 1..=34 {
        let tile = TileName::from_usize(i);
        let expected = evaluate_tile_safety(tile, &ctx, &visible_counts);
        let actual = evaluate_tile_safety_with_features(tile, &ctx, &visible_counts, &features);
        assert_eq!(
            expected, actual,
            "Tile {:?} safety mismatch in missing_river case",
            tile
        );
    }
}
