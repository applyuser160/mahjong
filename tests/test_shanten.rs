use mahjong::hand::Hand;
use mahjong::shanten::calculate_shanten;
use mahjong::tile::TileName;

#[test]
fn test_complex_shanten_patterns() {
    // 1. 完全一向聴 (両面2組 + 刻子1組 + 雀頭 + 孤立牌なし)
    // 2m3m 4p5p 6s7s8s 9s9s9s 東東 (12枚) + 1s (孤立牌) -> 一向聴 (1)
    let mut hand = Hand::new();
    for &t in &[
        TileName::TwoM,
        TileName::ThreeM,
        TileName::FourP,
        TileName::FiveP,
        TileName::SixS,
        TileName::SevenS,
        TileName::EightS,
        TileName::NineS,
        TileName::NineS,
        TileName::NineS,
        TileName::East,
        TileName::East,
        TileName::OneS, // 孤立牌
    ] {
        hand.push(t);
    }
    let res = calculate_shanten(&hand);
    assert_eq!(res.normal, 1);
    assert_eq!(res.min_shanten, 1);
}

#[test]
fn test_chiitoitsu_with_quads() {
    // 4枚持ちがある七対子
    // 1m1m1m1m 2p2p 3p3p 4s4s 5s5s 6s (13枚, 6種類) -> 種類不足のため二向聴 (2)
    let mut hand = Hand::new();
    for &t in &[
        TileName::OneM,
        TileName::OneM,
        TileName::OneM,
        TileName::OneM,
        TileName::TwoP,
        TileName::TwoP,
        TileName::ThreeP,
        TileName::ThreeP,
        TileName::FourS,
        TileName::FourS,
        TileName::FiveS,
        TileName::FiveS,
        TileName::SixS,
    ] {
        hand.push(t);
    }
    let res = calculate_shanten(&hand);
    assert_eq!(res.chitoitsu, 2); // 5対子だが6種類しかないため二向聴

    // 7種類ある場合: 1m1m 2m2m 3p3p 4p4p 5s5s 6s 7s (13枚, 7種類, 5対子) -> 一向聴 (1)
    let mut hand7 = Hand::new();
    for &t in &[
        TileName::OneM,
        TileName::OneM,
        TileName::TwoM,
        TileName::TwoM,
        TileName::ThreeP,
        TileName::ThreeP,
        TileName::FourP,
        TileName::FourP,
        TileName::FiveS,
        TileName::FiveS,
        TileName::SixS,
        TileName::SevenS,
        TileName::EightS,
    ] {
        hand7.push(t);
    }
    let res7 = calculate_shanten(&hand7);
    assert_eq!(res7.chitoitsu, 1);
}

#[test]
fn test_kokushi_13_wait_tenpai() {
    // 国士無双 13面待ちテンパイ (13種各1枚) -> 0向聴
    let mut hand = Hand::new();
    for &t in &[
        TileName::OneM,
        TileName::NineM,
        TileName::OneP,
        TileName::NineP,
        TileName::OneS,
        TileName::NineS,
        TileName::East,
        TileName::South,
        TileName::West,
        TileName::North,
        TileName::White,
        TileName::Green,
        TileName::Red,
    ] {
        hand.push(t);
    }
    let res = calculate_shanten(&hand);
    assert_eq!(res.kokushi, 0);
    assert_eq!(res.min_shanten, 0);
}

#[test]
fn test_ryanpeiko_form() {
    // 二盃口形: 223344m 223344p 5s5s (和了形: -1)
    let mut hand = Hand::new();
    for &t in &[
        TileName::TwoM,
        TileName::TwoM,
        TileName::ThreeM,
        TileName::ThreeM,
        TileName::FourM,
        TileName::FourM,
        TileName::TwoP,
        TileName::TwoP,
        TileName::ThreeP,
        TileName::ThreeP,
        TileName::FourP,
        TileName::FourP,
        TileName::FiveS,
        TileName::FiveS,
    ] {
        hand.push(t);
    }
    let res = calculate_shanten(&hand);
    assert_eq!(res.normal, -1);
    assert_eq!(res.chitoitsu, -1); // 7対子でもある
    assert_eq!(res.min_shanten, -1);
}
