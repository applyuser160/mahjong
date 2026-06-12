use mahjong::tile::TileName::*;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

#[test]
fn test_chitoitsu_honitsu() {
    let tiles = vec![
        OneM, OneM, TwoM, TwoM, FourM, FourM, FiveM, FiveM, SevenM, SevenM, NineM, NineM, Red, Red,
    ];

    let mut counts = [0u8; 35];
    for &t in &tiles {
        counts[t as usize] += 1;
    }

    let result = judge_yaku(
        &tiles,
        &counts,
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
        OneM, OneM, TwoM, TwoM, FourM, FourM, FiveM, FiveM, SevenM, SevenM, EightM, EightM, NineM,
        NineM,
    ];

    let mut counts = [0u8; 35];
    for &t in &tiles {
        counts[t as usize] += 1;
    }

    let result = judge_yaku(
        &tiles,
        &counts,
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

    let mut counts = [0u8; 35];
    for &t in &tiles {
        counts[t as usize] += 1;
    }

    let result = judge_yaku(
        &tiles,
        &counts,
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

    let mut counts = [0u8; 35];
    for &t in &tiles {
        counts[t as usize] += 1;
    }

    let result = judge_yaku(
        &tiles,
        &counts,
        &[],
        WinContext {
            is_closed: true,
            ..Default::default()
        },
    );
    assert!(!result.contains(&YakuId::Chitoitsu));
    assert!(result.contains(&YakuId::Tsuuiisou));
}
