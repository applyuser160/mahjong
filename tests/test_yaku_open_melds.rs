use mahjong::hand::Meld;
use mahjong::tile::TileName;
use mahjong::yaku::{judge_yaku, WinContext, YakuId};

#[test]
fn test_yaku_open_melds() {
    // A hand that relies on an open meld for YakuhaiHaku
    // Tiles: 123m 123p 123s 22s
    let tiles = vec![
        TileName::OneM,
        TileName::TwoM,
        TileName::ThreeM,
        TileName::OneP,
        TileName::TwoP,
        TileName::ThreeP,
        TileName::OneS,
        TileName::TwoS,
        TileName::ThreeS,
        TileName::TwoS,
        TileName::TwoS,
    ];
    // Meld: Pon of White Dragon
    let melds = vec![Meld::Pon(TileName::White)];

    let ctx = WinContext {
        is_closed: false,
        ..WinContext::default()
    };

    let mut counts = [0u8; 35];
    for &t in &tiles {
        counts[t as usize] += 1;
    }
    let result = judge_yaku(&tiles, &counts, &melds, ctx);
    assert!(result.contains(&YakuId::YakuhaiHaku));
}
