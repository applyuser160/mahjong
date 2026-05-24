import pytest
import mahjong

def test_tile_bindings():
    # Test PyTileName Enum and its methods
    t_name = mahjong.PyTileName.East
    assert t_name.as_str() == "東"
    assert t_name.tile_type() == mahjong.PyTileType.Winds
    assert t_name.category() == mahjong.PyTileCategory.Honors

    # Test PyTile Wrapper
    tile = mahjong.PyTile(t_name)
    assert tile.name == mahjong.PyTileName.East
    assert tile.tile_type == mahjong.PyTileType.Winds
    assert tile.category == mahjong.PyTileCategory.Honors

def test_hand_melds():
    hand = mahjong.PyHand()

    # Push tiles
    hand.push(mahjong.PyTileName.OneM)
    hand.push(mahjong.PyTileName.TwoM)
    hand.push(mahjong.PyTileName.ThreeM)
    hand.push(mahjong.PyTileName.East)
    hand.push(mahjong.PyTileName.East)

    tiles = hand.tiles
    assert len(tiles) == 5
    assert tiles[0] == mahjong.PyTileName.OneM

    # Call meld (Pon)
    # To call a Pon on East, we should have two Easts in hand. We do.
    # Oh wait, hand.call_meld consumes from hand if the tiles exist. Let's see.
    pon_meld = mahjong.PyMeld.pon(mahjong.PyTileName.East)
    hand.call_meld(pon_meld)

    assert len(hand.tiles) == 3
    assert len(hand.open_melds) == 1
    assert hand.open_melds[0].kind == "pon"

def test_round_wall():
    wall = mahjong.PyWall()
    wall.shuffle(42)

    assert wall.remaining() == 136 - 14

    round_obj = mahjong.PyRound(wall)
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
    ctx = mahjong.PyWinContext(
        is_closed=True,
        is_tsumo=True,
        seat_wind=mahjong.PyTileName.East,
        round_wind=mahjong.PyTileName.East, win_tile=mahjong.PyTileName.FourM,
    )

    # Chinitsu + Tanyao (1s 1s 1s 2s 3s 4s 5s 6s 7s 8s 9s 9s 9s) -> Chinitsu, Chuuren Poutou...
    # Let's just do a simple Riichi Menzen Tsumo Tanyao
    # 2m 3m 4m, 3p 4p 5p, 4s 5s 6s, 6s 7s 8s, 2p 2p
    tiles = [
        mahjong.PyTileName.TwoM, mahjong.PyTileName.ThreeM, mahjong.PyTileName.FourM,
        mahjong.PyTileName.ThreeP, mahjong.PyTileName.FourP, mahjong.PyTileName.FiveP,
        mahjong.PyTileName.FourS, mahjong.PyTileName.FiveS, mahjong.PyTileName.SixS,
        mahjong.PyTileName.SixS, mahjong.PyTileName.SevenS, mahjong.PyTileName.EightS,
        mahjong.PyTileName.TwoP, mahjong.PyTileName.TwoP,
    ]
    melds = []

    ctx.riichi = True

    yaku_ids = mahjong.py_judge_yaku(tiles, melds, ctx)
    assert mahjong.PyYakuId.Riichi in yaku_ids
    assert mahjong.PyYakuId.MenzenTsumo in yaku_ids
    assert mahjong.PyYakuId.Tanyao in yaku_ids
    assert mahjong.PyYakuId.Pinfu in yaku_ids # All sequences + non-yakuhai pair + 2-sided wait
