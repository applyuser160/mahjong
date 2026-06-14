
#[allow(unused_macros)]
macro_rules! to_counts {
    ($tiles:expr) => {{
        let mut counts = [0u8; 35];
        for &t in $tiles {
            counts[t as usize] += 1;
        }
        counts
    }};
}
use mahjong::tile::TileName::*;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

#[test]
fn test_ryanpeiko_does_not_have_chitoitsu() {
    let tiles = vec![
        OneM, OneM, TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveM, FiveM, SixM, SixM, SevenM,
        SevenM,
    ];

    let result = judge_yaku(&to_counts!(&tiles), &[], WinContext::default());
    assert!(result.contains(&YakuId::Ryanpeiko));
    assert!(!result.contains(&YakuId::Chitoitsu));
}
