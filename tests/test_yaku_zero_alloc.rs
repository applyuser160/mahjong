use mahjong::hand::{Hand, Meld};
use mahjong::tile::TileName::*;
use mahjong::yaku::{judge_yaku, judge_yaku_set, WinContext, YakuId};

#[test]
fn test_sanshoku_doujun_and_doukou_zero_alloc() {
    // 1. 三色同順 (123m, 123p, 123s)
    let mut hand = Hand::new();
    for &t in &[
        OneM, TwoM, ThreeM, OneP, TwoP, ThreeP, OneS, TwoS, ThreeS, East, East,
    ] {
        hand.push(t);
    }
    // 副露チーで 123m を持っている場合でも成立
    let open_melds = vec![Meld::Chii {
        called: OneM,
        consumed: [TwoM, ThreeM],
    }];
    let ctx = WinContext {
        is_closed: false,
        is_tsumo: true,
        win_tile: Some(OneP),
        ..Default::default()
    };
    let set = judge_yaku_set(&hand.counts, &open_melds, ctx);
    assert!(
        set.contains(&YakuId::SanshokuDoujun),
        "Must contain SanshokuDoujun"
    );

    // 2. 2色のみ (123m, 123p, 456s) -> 三色同順不成立
    let mut hand2 = Hand::new();
    for &t in &[
        OneM, TwoM, ThreeM, OneP, TwoP, ThreeP, FourS, FiveS, SixS, East, East, FourP, FiveP, SixP,
    ] {
        hand2.push(t);
    }
    let ctx2 = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(SixP),
        ..Default::default()
    };
    let set2 = judge_yaku_set(&hand2.counts, &[], ctx2);
    assert!(
        !set2.contains(&YakuId::SanshokuDoujun),
        "Must NOT contain SanshokuDoujun"
    );

    // 3. 三色同刻 (222m, 222p, 222s)
    let mut hand3 = Hand::new();
    for &t in &[
        TwoM, TwoM, TwoM, TwoP, TwoP, TwoP, TwoS, TwoS, TwoS, West, West,
    ] {
        hand3.push(t);
    }
    let open_melds3 = vec![Meld::Pon(TwoM)];
    let ctx3 = WinContext {
        is_closed: false,
        is_tsumo: true,
        win_tile: Some(TwoS),
        ..Default::default()
    };
    let set3 = judge_yaku_set(&hand3.counts, &open_melds3, ctx3);
    assert!(
        set3.contains(&YakuId::SanshokuDoukou),
        "Must contain SanshokuDoukou"
    );
}

#[test]
fn test_ipeiko_and_ryanpeiko_zero_alloc() {
    // 一盃口 (223344m)
    let mut hand = Hand::new();
    for &t in &[
        TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveP, SixP, SevenP, EightS, EightS, EightS,
        White, White,
    ] {
        hand.push(t);
    }
    let ctx = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(White),
        ..Default::default()
    };
    let set = judge_yaku_set(&hand.counts, &[], ctx);
    assert!(set.contains(&YakuId::Ipeiko));
    assert!(!set.contains(&YakuId::Ryanpeiko));

    // 二盃口 (223344m + 556677p)
    let mut hand2 = Hand::new();
    for &t in &[
        TwoM, TwoM, ThreeM, ThreeM, FourM, FourM, FiveP, FiveP, SixP, SixP, SevenP, SevenP, East,
        East,
    ] {
        hand2.push(t);
    }
    let ctx2 = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(East),
        ..Default::default()
    };
    let set2 = judge_yaku_set(&hand2.counts, &[], ctx2);
    assert!(set2.contains(&YakuId::Ryanpeiko));
    // 二盃口成立時は一盃口は除外される（上位役）
    assert!(!set2.contains(&YakuId::Ipeiko));
}

#[test]
fn test_chuuren_many_patterns_smallvec_safety() {
    // 純正九蓮宝燈 (1112345678999m + 1m)
    let mut chuuren = Hand::new();
    for &t in &[
        OneM, OneM, OneM, TwoM, ThreeM, FourM, FiveM, SixM, SevenM, EightM, NineM, NineM, NineM,
        OneM,
    ] {
        chuuren.push(t);
    }
    let ctx = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(OneM),
        ..Default::default()
    };
    // パターン数が多くても panic せず正常に役満判定されること
    let set = judge_yaku_set(&chuuren.counts, &[], ctx);
    assert!(set.contains(&YakuId::ChuurenPoutou));
}

#[test]
fn test_judge_yaku_wrapper_compatibility() {
    // judge_yaku (HashSet返却) と judge_yaku_set (YakuSet返却) の同値性を確認
    let mut hand = Hand::new();
    for &t in &[
        OneM, TwoM, ThreeM, FourP, FiveP, SixP, SevenS, EightS, NineS, East, East, TwoS, ThreeS,
        OneS,
    ] {
        hand.push(t);
    }
    let ctx = WinContext {
        is_closed: true,
        is_tsumo: true,
        win_tile: Some(OneS),
        ..Default::default()
    };

    let set = judge_yaku_set(&hand.counts, &[], ctx);
    let hashset = judge_yaku(&hand.counts, &[], ctx);

    assert_eq!(set.len(), hashset.len());
    for id in set {
        assert!(
            hashset.contains(&id),
            "HashSet must contain YakuId {:?}",
            id
        );
    }
}
