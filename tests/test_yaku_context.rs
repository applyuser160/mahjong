
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
use mahjong::tile::TileName;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

fn create_valid_hand() -> Vec<TileName> {
    // A simple complete hand: 123m, 456m, 789m, 11z, 222z
    vec![
        TileName::OneM,
        TileName::TwoM,
        TileName::ThreeM,
        TileName::FourM,
        TileName::FiveM,
        TileName::SixM,
        TileName::SevenM,
        TileName::EightM,
        TileName::NineM,
        TileName::East,
        TileName::East,
        TileName::South,
        TileName::South,
        TileName::South,
    ]
}

#[test]
fn test_rinshan_kaihou() {
    let tiles = create_valid_hand();
    let ctx = WinContext {
        is_tsumo: true,
        is_rinshan: true,
        ..WinContext::default()
    };

    let yaku = judge_yaku(&to_counts!(&tiles), &[], ctx);
    assert!(yaku.contains(&YakuId::RinshanKaihou));
}

#[test]
fn test_chankan() {
    let tiles = create_valid_hand();
    let ctx = WinContext {
        is_tsumo: false,
        is_chankan: true,
        ..WinContext::default()
    };

    let yaku = judge_yaku(&to_counts!(&tiles), &[], ctx);
    assert!(yaku.contains(&YakuId::Chankan));
}

#[test]
fn test_haitei_raoyue() {
    let tiles = create_valid_hand();
    let ctx = WinContext {
        is_tsumo: true,
        is_haitei: true,
        ..WinContext::default()
    };

    let yaku = judge_yaku(&to_counts!(&tiles), &[], ctx);
    assert!(yaku.contains(&YakuId::HaiteiRaoyue));
}

#[test]
fn test_houtei_raoyui() {
    let tiles = create_valid_hand();
    let ctx = WinContext {
        is_tsumo: false,
        is_houtei: true,
        ..WinContext::default()
    };

    let yaku = judge_yaku(&to_counts!(&tiles), &[], ctx);
    assert!(yaku.contains(&YakuId::HouteiRaoyui));
}

#[test]
fn test_double_riichi_and_ippatsu() {
    let tiles = create_valid_hand();
    let ctx = WinContext {
        is_double_riichi: true,
        is_ippatsu: true,
        is_closed: true,
        ..WinContext::default()
    };

    let yaku = judge_yaku(&to_counts!(&tiles), &[], ctx);
    assert!(yaku.contains(&YakuId::DoubleRiichi));
    assert!(yaku.contains(&YakuId::Ippatsu));
    assert!(!yaku.contains(&YakuId::Riichi)); // Double Riichi supersedes Riichi
}

#[test]
fn test_ippatsu_with_normal_riichi() {
    let tiles = create_valid_hand();
    let ctx = WinContext {
        riichi: true,
        is_ippatsu: true,
        is_closed: true,
        ..WinContext::default()
    };

    let yaku = judge_yaku(&to_counts!(&tiles), &[], ctx);
    assert!(yaku.contains(&YakuId::Riichi));
    assert!(yaku.contains(&YakuId::Ippatsu));
    assert!(!yaku.contains(&YakuId::DoubleRiichi));
}
