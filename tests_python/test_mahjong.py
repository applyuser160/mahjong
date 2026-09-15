import mahjong


def test_tile_bindings():
    # Test PyTileName Enum and its methods
    t_name = mahjong.TileName.East
    assert t_name.as_str() == "東"
    assert t_name.tile_type() == mahjong.TileType.Winds
    assert t_name.category() == mahjong.TileCategory.Honors

    # Test PyTile Wrapper
    tile = mahjong.Tile(t_name)
    assert tile.name == mahjong.TileName.East
    assert tile.tile_type == mahjong.TileType.Winds
    assert tile.category == mahjong.TileCategory.Honors


def test_hand_melds():
    hand = mahjong.Hand()

    # Push tiles
    hand.push(mahjong.TileName.OneM)
    hand.push(mahjong.TileName.TwoM)
    hand.push(mahjong.TileName.ThreeM)
    hand.push(mahjong.TileName.East)
    hand.push(mahjong.TileName.East)

    tiles = hand.tiles
    assert len(tiles) == 5
    assert tiles[0] == mahjong.TileName.OneM

    # Call meld (Pon)
    # To call a Pon on East, we should have two Easts in hand. We do.
    # Oh wait, hand.call_meld consumes from hand if the tiles exist. Let's see.
    pon_meld = mahjong.Meld.pon(mahjong.TileName.East)
    hand.call_meld(pon_meld)

    assert len(hand.tiles) == 3
    assert len(hand.open_melds) == 1
    assert hand.open_melds[0].kind == "pon"


def test_round_wall():
    wall = mahjong.Wall()
    wall.shuffle(42)

    assert wall.remaining() == 136 - 14

    round_obj = mahjong.Round(wall)
    assert round_obj.turn() == 0
    assert len(round_obj.hand(0)) == 13

    # Draw tile
    drawn = round_obj.draw_tile()
    assert drawn is not None
    assert len(round_obj.hand(0)) == 14

    # Discard tile
    discarded = round_obj.discard_tile(0)
    assert discarded is not None
    assert len(round_obj.hand(0)) == 13
    assert round_obj.turn() == 1


def test_judge_yaku():
    # Tsumo, Closed, Seat East, Round East
    ctx = mahjong.WinContext(
        is_closed=True,
        is_tsumo=True,
        seat_wind=mahjong.TileName.East,
        round_wind=mahjong.TileName.East,
        win_tile=mahjong.TileName.FourM,
    )

    # Chinitsu + Tanyao (1s 1s 1s 2s 3s 4s 5s 6s 7s 8s 9s 9s 9s) -> Chinitsu, Chuuren Poutou...
    # Let's just do a simple Riichi Menzen Tsumo Tanyao
    # 2m 3m 4m, 3p 4p 5p, 4s 5s 6s, 6s 7s 8s, 2p 2p
    tiles = [
        mahjong.TileName.TwoM,
        mahjong.TileName.ThreeM,
        mahjong.TileName.FourM,
        mahjong.TileName.ThreeP,
        mahjong.TileName.FourP,
        mahjong.TileName.FiveP,
        mahjong.TileName.FourS,
        mahjong.TileName.FiveS,
        mahjong.TileName.SixS,
        mahjong.TileName.SixS,
        mahjong.TileName.SevenS,
        mahjong.TileName.EightS,
        mahjong.TileName.TwoP,
        mahjong.TileName.TwoP,
    ]
    melds = []

    ctx.riichi = True

    yaku_ids = mahjong.judge_yaku(tiles, melds, ctx)
    assert mahjong.YakuId.Riichi in yaku_ids
    assert mahjong.YakuId.MenzenTsumo in yaku_ids
    assert mahjong.YakuId.Tanyao in yaku_ids
    assert (
        mahjong.YakuId.Pinfu in yaku_ids
    )  # All sequences + non-yakuhai pair + 2-sided wait


def test_shanten_bindings():
    # 1m 2m 3m 4p 5p 6p 7s 8s 9s East East East White White -> 和了形 (-1向聴)
    tiles = [
        mahjong.TileName.OneM,
        mahjong.TileName.TwoM,
        mahjong.TileName.ThreeM,
        mahjong.TileName.FourP,
        mahjong.TileName.FiveP,
        mahjong.TileName.SixP,
        mahjong.TileName.SevenS,
        mahjong.TileName.EightS,
        mahjong.TileName.NineS,
        mahjong.TileName.East,
        mahjong.TileName.East,
        mahjong.TileName.East,
        mahjong.TileName.White,
        mahjong.TileName.White,
    ]
    res = mahjong.calculate_shanten(tiles)
    assert res.min_shanten == -1
    assert res.normal == -1

    # Hand instance method test
    hand = mahjong.Hand()
    for t in tiles[:13]:  # 白単騎テンパイ (0向聴)
        hand.push(t)
    hand_res = hand.shanten()
    assert hand_res.min_shanten == 0
    assert hand_res.normal == 0

    # 5枚以上の同一牌で ValueError が発生することの検証
    import pytest

    with pytest.raises(ValueError):
        mahjong.calculate_shanten([mahjong.TileName.NineM] * 5)


def test_expectation_and_review_tracker():
    tiles = [
        mahjong.TileName.OneM,
        mahjong.TileName.TwoM,
        mahjong.TileName.ThreeM,
        mahjong.TileName.FourP,
        mahjong.TileName.FiveP,
        mahjong.TileName.SixP,
        mahjong.TileName.SevenS,
        mahjong.TileName.EightS,
        mahjong.TileName.NineS,
        mahjong.TileName.East,
        mahjong.TileName.East,
        mahjong.TileName.TwoS,
        mahjong.TileName.ThreeS,
        mahjong.TileName.NineS,
    ]
    evals = mahjong.evaluate_hand_discards(tiles)
    assert len(evals) > 0
    best = evals[0]
    assert best.discard_tile == mahjong.TileName.NineS
    assert best.shanten_after == 0
    assert best.ev > 2000.0

    tracker = mahjong.ReviewTracker()
    tracker.record_decision(1, best.discard_tile, evals)
    assert tracker.get_accuracy_rate() == 1.0
    assert tracker.get_total_ev_loss() == 0.0
    report_text = tracker.format_report()
    assert "局後学習振り返りレポート" in report_text


def test_call_advisor():
    # 手牌: 123m 45p 789s EE (11枚)
    tiles = [
        mahjong.TileName.OneM,
        mahjong.TileName.TwoM,
        mahjong.TileName.ThreeM,
        mahjong.TileName.FourP,
        mahjong.TileName.FiveP,
        mahjong.TileName.SevenS,
        mahjong.TileName.EightS,
        mahjong.TileName.NineS,
        mahjong.TileName.East,
        mahjong.TileName.East,
    ]
    # 上家から 6p (チー可能)
    advice = mahjong.advise_call(tiles, mahjong.TileName.SixP, is_kamicha=True)
    assert advice is not None
    assert advice.target_tile == mahjong.TileName.SixP
    assert len(advice.choices) >= 2  # Chii and Pass
    assert len(advice.recommendation) > 0


def test_drill_problem():
    problem = mahjong.generate_drill_problem(target_shanten=0)
    assert problem is not None
    assert len(problem.tiles) == 14
    assert len(problem.candidates) > 0
    assert problem.best_tile is not None
    assert len(problem.rationale) > 0
