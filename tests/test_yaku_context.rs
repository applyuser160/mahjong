use mahjong::yaku::{judge_yaku, WinContext, YakuId};
use mahjong::tile::TileName;

fn create_valid_hand() -> Vec<TileName> {
    // A simple complete hand: 123m, 456m, 789m, 11z, 222z
    vec![
        TileName::OneM, TileName::TwoM, TileName::ThreeM,
        TileName::FourM, TileName::FiveM, TileName::SixM,
        TileName::SevenM, TileName::EightM, TileName::NineM,
        TileName::East, TileName::East,
        TileName::South, TileName::South, TileName::South,
    ]
}

#[test]
fn test_rinshan_kaihou() {
    let tiles = create_valid_hand();
    let mut ctx = WinContext::default();
    ctx.is_tsumo = true;
    ctx.is_rinshan = true;

    let yaku = judge_yaku(&tiles, &[], ctx);
    assert!(yaku.contains(&YakuId::RinshanKaihou));
}

#[test]
fn test_chankan() {
    let tiles = create_valid_hand();
    let mut ctx = WinContext::default();
    ctx.is_tsumo = false;
    ctx.is_chankan = true;

    let yaku = judge_yaku(&tiles, &[], ctx);
    assert!(yaku.contains(&YakuId::Chankan));
}

#[test]
fn test_haitei_raoyue() {
    let tiles = create_valid_hand();
    let mut ctx = WinContext::default();
    ctx.is_tsumo = true;
    ctx.is_haitei = true;

    let yaku = judge_yaku(&tiles, &[], ctx);
    assert!(yaku.contains(&YakuId::HaiteiRaoyue));
}

#[test]
fn test_houtei_raoyui() {
    let tiles = create_valid_hand();
    let mut ctx = WinContext::default();
    ctx.is_tsumo = false;
    ctx.is_houtei = true;

    let yaku = judge_yaku(&tiles, &[], ctx);
    assert!(yaku.contains(&YakuId::HouteiRaoyui));
}

#[test]
fn test_double_riichi_and_ippatsu() {
    let tiles = create_valid_hand();
    let mut ctx = WinContext::default();
    ctx.is_double_riichi = true;
    ctx.is_ippatsu = true;
    ctx.is_closed = true;

    let yaku = judge_yaku(&tiles, &[], ctx);
    assert!(yaku.contains(&YakuId::DoubleRiichi));
    assert!(yaku.contains(&YakuId::Ippatsu));
    assert!(!yaku.contains(&YakuId::Riichi)); // Double Riichi supersedes Riichi
}

#[test]
fn test_ippatsu_with_normal_riichi() {
    let tiles = create_valid_hand();
    let mut ctx = WinContext::default();
    ctx.riichi = true;
    ctx.is_ippatsu = true;
    ctx.is_closed = true;

    let yaku = judge_yaku(&tiles, &[], ctx);
    assert!(yaku.contains(&YakuId::Riichi));
    assert!(yaku.contains(&YakuId::Ippatsu));
    assert!(!yaku.contains(&YakuId::DoubleRiichi));
}
