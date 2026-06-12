use mahjong::tile::TileName::*;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

#[test]
fn test_ryanpeiko_does_not_have_chitoitsu() {
    let tiles = vec![
        OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveM, FiveM, SixM, SixM, SevenM,
        SevenM,
    ];

    let mut counts = [0u8; 35];
    for &t in &tiles {
        counts[t as usize] += 1;
    }
    let result = judge_yaku(&tiles, &counts, &[], WinContext::default());
    assert!(result.contains(&YakuId::Ryanpeiko));
    assert!(!result.contains(&YakuId::Chitoitsu));
}
