import pytest
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
        round_wind=mahjong.TileName.East, win_tile=mahjong.TileName.FourM,
    )

    # Chinitsu + Tanyao (1s 1s 1s 2s 3s 4s 5s 6s 7s 8s 9s 9s 9s) -> Chinitsu, Chuuren Poutou...
    # Let's just do a simple Riichi Menzen Tsumo Tanyao
    # 2m 3m 4m, 3p 4p 5p, 4s 5s 6s, 6s 7s 8s, 2p 2p
    tiles = [
        mahjong.TileName.TwoM, mahjong.TileName.ThreeM, mahjong.TileName.FourM,
        mahjong.TileName.ThreeP, mahjong.TileName.FourP, mahjong.TileName.FiveP,
        mahjong.TileName.FourS, mahjong.TileName.FiveS, mahjong.TileName.SixS,
        mahjong.TileName.SixS, mahjong.TileName.SevenS, mahjong.TileName.EightS,
        mahjong.TileName.TwoP, mahjong.TileName.TwoP,
    ]
    melds = []

    ctx.riichi = True

    yaku_ids = mahjong.judge_yaku(tiles, melds, ctx)
    assert mahjong.YakuId.Riichi in yaku_ids
    assert mahjong.YakuId.MenzenTsumo in yaku_ids
    assert mahjong.YakuId.Tanyao in yaku_ids
    assert mahjong.YakuId.Pinfu in yaku_ids # All sequences + non-yakuhai pair + 2-sided wait
