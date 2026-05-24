use mahjong::tile::TileName::*;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

#[test]
fn detect_pinfu_overlapping_kanchan_is_not_misidentified() {
    // 13-tile hand:
    // 1m, 3m (kanchan 2m)
    // 2m, 3m, 4m (sequence)
    // 4p, 5p, 6p
    // 3s, 4s, 5s
    // 2p, 2p (pair)
    // Win on 2m
    // Hand: 1m, 2m, 2m, 3m, 3m, 4m, 4p, 5p, 6p, 3s, 4s, 5s, 2p, 2p
    // This parses as 123m and 234m.
    // The previous logic considered this ryamen because 2m is the start of 234m.
    // The new logic correctly verifies that removing 2m from 234m leaves 34m,
    // which IS a valid ryamen taatsu.
    // So this is STILL correctly identified as Pinfu, confirming the logic rewrite
    // behaves equivalently while mathematically satisfying the explicit 13-tile extraction.
    let tiles = vec![
        OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, // 122334m
        FourP, FiveP, SixP, // 456p
        ThreeS, FourS, FiveS, // 345s
        TwoP, TwoP, // pair
    ];

    let ctx = WinContext {
        is_closed: true,
        is_tsumo: false,
        win_tile: Some(TwoM), // won on 2m
        ..Default::default()
    };
    let result = judge_yaku(&tiles, &[], ctx);
    assert!(result.contains(&YakuId::Pinfu), "Should be Pinfu under standard high-score rules");
}

#[test]
fn detect_pinfu_nobetan_is_correctly_rejected() {
    // 2345m with win on 5m
    let tiles = vec![
        TwoM, ThreeM, FourM, FiveM, FiveM, // 2345m + 5m = 23455m
        FourP, FiveP, SixP, // 456p
        ThreeS, FourS, FiveS, // 345s
        SixS, SevenS, EightS, // 678s
    ];

    let ctx = WinContext {
        is_closed: true,
        is_tsumo: false,
        win_tile: Some(FiveM),
        ..Default::default()
    };
    let result = judge_yaku(&tiles, &[], ctx);
    assert!(!result.contains(&YakuId::Pinfu), "Nobetan should not be Pinfu");
}

#[test]
fn detect_pinfu_overlapping_pair_tanki_rejected() {
    // 23444m with win on 4m
    let tiles = vec![
        TwoM, ThreeM, FourM, FourM, FourM, // 23444m
        FourP, FiveP, SixP, // 456p
        ThreeS, FourS, FiveS, // 345s
        SixS, SevenS, EightS, // 678s
    ];

    let ctx = WinContext {
        is_closed: true,
        is_tsumo: false,
        win_tile: Some(FourM),
        ..Default::default()
    };
    let result = judge_yaku(&tiles, &[], ctx);
    assert!(!result.contains(&YakuId::Pinfu), "Aryamen/tanki should not be Pinfu");
}
