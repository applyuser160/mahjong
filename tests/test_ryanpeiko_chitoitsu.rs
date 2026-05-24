use mahjong::tile::TileName::*;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

#[test]
fn test_ryanpeiko_does_not_have_chitoitsu() {
    let tiles = vec![
        OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveM, FiveM, SixM, SixM, SevenM,
        SevenM,
    ];

    let result = judge_yaku(&tiles, &[], WinContext::default());
    assert!(result.contains(&YakuId::Ryanpeiko));
    assert!(!result.contains(&YakuId::Chitoitsu));
}
