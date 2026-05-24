use mahjong::tile::TileName::*;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

#[test]
fn test_chitoitsu_honitsu() {
    let tiles = vec![
        OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveM, FiveM, SixM, SixM, Red, Red,
    ];

    let result = judge_yaku(
        &tiles,
        &[],
        WinContext {
            is_closed: true,
            ..Default::default()
        },
    );
    assert!(result.contains(&YakuId::Chitoitsu));
    assert!(result.contains(&YakuId::Honitsu));
}

#[test]
fn test_chitoitsu_chinitsu() {
    let tiles = vec![
        OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveM, FiveM, SixM, SixM, SevenM,
        SevenM,
    ];

    let result = judge_yaku(
        &tiles,
        &[],
        WinContext {
            is_closed: true,
            ..Default::default()
        },
    );
    assert!(result.contains(&YakuId::Chitoitsu));
    assert!(result.contains(&YakuId::Chinitsu));
}

#[test]
fn test_chitoitsu_honroutou() {
    let tiles = vec![
        OneM, OneM, NineM, NineM, OneP, OneP, NineP, NineP, OneS, OneS, NineS, NineS, East, East,
    ];

    let result = judge_yaku(
        &tiles,
        &[],
        WinContext {
            is_closed: true,
            ..Default::default()
        },
    );
    assert!(result.contains(&YakuId::Chitoitsu));
    assert!(result.contains(&YakuId::Honroutou));
}

#[test]
fn test_chitoitsu_tsuuiisou() {
    let tiles = vec![
        East, East, South, South, West, West, North, North, White, White, Green, Green, Red, Red,
    ];

    let result = judge_yaku(
        &tiles,
        &[],
        WinContext {
            is_closed: true,
            ..Default::default()
        },
    );
    assert!(!result.contains(&YakuId::Chitoitsu));
    assert!(result.contains(&YakuId::Tsuuiisou));
}
