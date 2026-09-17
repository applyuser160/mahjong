import pytest
from mahjong import (
    TileName,
    Tile,
    Hand,
    Meld,
    River,
    RuleConfig,
    MatchContext,
    TableState,
    evaluate_placement_discards,
    calculate_orasu_conditions,
    get_ai_hud_data,
    generate_drill_problem,
    advise_call,
    evaluate_hand_discards,
    ReviewTracker,
)


def test_tile_mpsz_and_dict():
    tile = Tile(TileName.OneM)
    assert tile.as_str() == "1m"
    assert tile.mpsz() == "1m"
    d = tile.to_dict()
    assert d["name"] == "1m"
    assert d["mpsz"] == "1m"

    east = TileName.East
    assert east.as_str() == "東"
    assert east.mpsz() == "1z"

    white = TileName.White
    assert white.as_str() == "白"
    assert white.mpsz() == "5z"


def test_hand_and_river_to_dict():
    hand = Hand()
    hand.push(TileName.OneM)
    hand.push(TileName.TwoM)
    hand.push(TileName.ThreeM)
    hand.push(TileName.SevenS)
    hand.push(TileName.SevenS)

    meld = Meld.pon(TileName.SevenS)
    assert meld.kind == "pon"
    meld_dict = meld.to_dict()
    assert meld_dict["kind"] == "pon"
    assert meld_dict["mpsz"] == ["7s", "7s", "7s"]

    hand.call_meld(meld)
    hand_dict = hand.to_dict()
    assert hand_dict["tiles"] == ["1m", "2m", "3m"]
    assert hand_dict["mpsz"] == ["1m", "2m", "3m"]
    assert len(hand_dict["open_melds"]) == 1
    assert "shanten" in hand_dict

    river = River()
    river_dict = river.to_dict()
    assert river_dict["tiles"] == []
    assert river_dict["mpsz"] == []


def test_rule_config_and_match_context():
    rule = RuleConfig.mleague()
    assert rule.origin_score == 25000
    assert rule.return_score == 30000
    assert rule.uma == [50, 10, -10, -30]
    rule_dict = rule.to_dict()
    assert rule_dict["origin_score"] == 25000

    ctx = MatchContext(
        scores=[35000, 28000, 20000, 17000],
        round_wind=TileName.South,
        round_number=4,
        honba=1,
        riichi_sticks=1,
        dealer_idx=0,
        rule=rule,
    )
    assert ctx.is_orasu() is True
    ranks = ctx.current_ranks()
    assert ranks == [1, 2, 3, 4]
    assert ctx.score_diff(1, 0) == 7000  # player 1 vs player 0

    ctx_dict = ctx.to_dict()
    assert ctx_dict["is_orasu"] is True
    assert ctx_dict["scores"] == [35000, 28000, 20000, 17000]
    assert ctx_dict["ranks"] == [1, 2, 3, 4]
    assert ctx_dict["round_wind"] == "南"
    assert ctx_dict["round_wind_mpsz"] == "2z"


def test_placement_evaluation_and_hud_data():
    tiles = [
        TileName.OneM,
        TileName.TwoM,
        TileName.ThreeM,
        TileName.FourP,
        TileName.FiveP,
        TileName.SixP,
        TileName.SevenS,
        TileName.EightS,
        TileName.NineS,
        TileName.East,
        TileName.East,
        TileName.White,
        TileName.White,
        TileName.NineM,
    ]
    ctx = MatchContext(
        scores=[32000, 28000, 22000, 18000],
        round_wind=TileName.East,
        round_number=1,
    )

    evals = evaluate_placement_discards(tiles, ctx, player_idx=0)
    assert len(evals) > 0
    top = evals[0]
    top_dict = top.to_dict()

    assert "discard_tile" in top_dict
    assert "mpsz" in top_dict
    assert "placement_ev" in top_dict
    assert "rank_probabilities" in top_dict
    assert len(top_dict["rank_probabilities"]) == 4
    assert "situational_note" in top_dict

    # HUD data unified dictionary
    hud = get_ai_hud_data(tiles, ctx, player_idx=0)
    assert hud["current_rank"] == 1
    assert hud["current_score"] == 32000
    assert hud["is_orasu"] is False
    assert len(hud["candidates"]) > 0
    assert "best_tile" in hud
    assert "best_mpsz" in hud
    assert "best_placement_ev" in hud
    assert "best_note" in hud


def test_orasu_win_conditions():
    # Player 1 is 2nd place with 25000 pts, trailing Player 0 with 31000 pts (6000 diff) in South 4
    ctx = MatchContext(
        scores=[31000, 25000, 24000, 20000],
        round_wind=TileName.South,
        round_number=4,
        dealer_idx=0,
    )
    assert ctx.is_orasu() is True
    conds = calculate_orasu_conditions(ctx, player_idx=1)
    assert len(conds) > 0
    c_dict = conds[0].to_dict()
    assert c_dict["target_rank"] == 1
    assert c_dict["target_player"] == 0
    assert c_dict["diff"] == 6000
    assert "summary" in c_dict


def test_table_state_masking():
    hands = [
        [TileName.OneM, TileName.TwoM, TileName.ThreeM],
        [TileName.FourP, TileName.FiveP, TileName.SixP],
        [TileName.SevenS, TileName.EightS, TileName.NineS],
        [TileName.East, TileName.South, TileName.West],
    ]
    melds = [[], [], [], []]
    rivers = [[], [], [], []]

    table = TableState(
        round_wind=TileName.East,
        round_number=1,
        honba=0,
        riichi_sticks=0,
        dealer_idx=0,
        current_turn=0,
        dora_indicators=[TileName.FiveM],
        remaining_wall_tiles=70,
        scores=[25000, 25000, 25000, 25000],
        is_riichi=[False, False, False, False],
        hands=hands,
        melds=melds,
        rivers=rivers,
    )

    # By default, opponent hands are masked with "?"
    masked_dict = table.to_dict(reveal_all=False)
    players = masked_dict["players"]
    assert players[0]["hand"] == ["1m", "2m", "3m"]  # Seat 0 visible
    assert players[1]["hand"] == ["?", "?", "?"]  # Seat 1 masked
    assert players[2]["hand"] == ["?", "?", "?"]  # Seat 2 masked
    assert players[3]["hand"] == ["?", "?", "?"]  # Seat 3 masked

    # Reveal all (e.g. spectator / replay mode)
    revealed_dict = table.to_dict(reveal_all=True)
    players_rev = revealed_dict["players"]
    assert players_rev[1]["hand"] == ["4p", "5p", "6p"]
    assert players_rev[1]["hand_mpsz"] == ["4p", "5p", "6p"]


def test_drill_and_call_advice_to_dict():
    prob = generate_drill_problem()
    if prob is not None:
        p_dict = prob.to_dict()
        assert "tiles" in p_dict
        assert "mpsz" in p_dict
        assert "best_tile" in p_dict
        assert "best_mpsz" in p_dict
        assert "candidates" in p_dict

    tiles = [
        TileName.OneM,
        TileName.TwoM,
        TileName.FourM,
        TileName.FiveP,
        TileName.FiveP,
        TileName.SevenS,
        TileName.EightS,
        TileName.NineS,
        TileName.White,
        TileName.White,
        TileName.White,
        TileName.East,
        TileName.East,
    ]
    advice = advise_call(tiles, TileName.ThreeM, is_kamicha=True)
    if advice is not None:
        a_dict = advice.to_dict()
        assert a_dict["target_tile"] == "3m"
        assert a_dict["target_mpsz"] == "3m"
        assert "best_action" in a_dict
        assert "choices" in a_dict


def test_review_tracker_to_dict():
    tracker = ReviewTracker()
    tiles = [
        TileName.OneM,
        TileName.TwoM,
        TileName.ThreeM,
        TileName.FourP,
        TileName.FiveP,
        TileName.SixP,
        TileName.SevenS,
        TileName.EightS,
        TileName.NineS,
        TileName.East,
        TileName.East,
        TileName.White,
        TileName.White,
        TileName.NineM,
    ]
    cands = evaluate_hand_discards(tiles)
    tracker.record_decision(1, TileName.NineM, cands)

    r_dict = tracker.to_dict()
    assert r_dict["total_turns"] == 1
    assert "accuracy_rate" in r_dict
    assert "blunders" in r_dict

    blunders = tracker.get_blunders_dict()
    assert isinstance(blunders, list)


def test_invalid_player_and_dealer_indices():
    tiles = [TileName.OneM] * 14
    ctx = MatchContext(dealer_idx=0)

    # 1. Reject out-of-range player_idx in public evaluation functions
    with pytest.raises(ValueError, match="player_idx"):
        evaluate_placement_discards(tiles, ctx, player_idx=4)
    with pytest.raises(ValueError, match="player_idx"):
        calculate_orasu_conditions(ctx, player_idx=4)
    with pytest.raises(ValueError, match="player_idx"):
        get_ai_hud_data(tiles, ctx, player_idx=4)

    # 2. Reject out-of-range dealer_idx in MatchContext construction and mutation
    with pytest.raises(ValueError, match="dealer_idx"):
        MatchContext(dealer_idx=4)
    with pytest.raises(ValueError, match="dealer_idx"):
        ctx.dealer_idx = 4

    # 3. Reject out-of-range player index in score_diff
    with pytest.raises(ValueError, match="player index"):
        ctx.score_diff(4, 0)
    with pytest.raises(ValueError, match="player index"):
        ctx.score_diff(0, 4)

    # 4. Reject out-of-range dealer_idx and current_turn in TableState
    hands = [[TileName.OneM]] * 4
    melds = [[]] * 4
    rivers = [[]] * 4
    with pytest.raises(ValueError, match="dealer_idx"):
        TableState(
            round_wind=TileName.East,
            round_number=1,
            honba=0,
            riichi_sticks=0,
            dealer_idx=4,
            current_turn=0,
            dora_indicators=[TileName.FiveM],
            remaining_wall_tiles=70,
            scores=[25000] * 4,
            is_riichi=[False] * 4,
            hands=hands,
            melds=melds,
            rivers=rivers,
        )

    with pytest.raises(ValueError, match="current_turn"):
        TableState(
            round_wind=TileName.East,
            round_number=1,
            honba=0,
            riichi_sticks=0,
            dealer_idx=0,
            current_turn=4,
            dora_indicators=[TileName.FiveM],
            remaining_wall_tiles=70,
            scores=[25000] * 4,
            is_riichi=[False] * 4,
            hands=hands,
            melds=melds,
            rivers=rivers,
        )

    table = TableState(
        round_wind=TileName.East,
        round_number=1,
        honba=0,
        riichi_sticks=0,
        dealer_idx=0,
        current_turn=0,
        dora_indicators=[TileName.FiveM],
        remaining_wall_tiles=70,
        scores=[25000] * 4,
        is_riichi=[False] * 4,
        hands=hands,
        melds=melds,
        rivers=rivers,
    )
    with pytest.raises(ValueError, match="dealer_idx"):
        table.dealer_idx = 10
    with pytest.raises(ValueError, match="current_turn"):
        table.current_turn = 5


def test_evaluate_hand_discards_with_context():
    # 1m-9m, 1p,2p,3p, 4p,4p (14 tiles)
    hand = [
        TileName.OneM,
        TileName.TwoM,
        TileName.ThreeM,
        TileName.FourM,
        TileName.FiveM,
        TileName.SixM,
        TileName.SevenM,
        TileName.EightM,
        TileName.NineM,
        TileName.OneP,
        TileName.TwoP,
        TileName.ThreeP,
        TileName.FourP,
        TileName.FourP,
    ]
    # Default call (backward compatibility)
    evs_default = evaluate_hand_discards(hand)
    assert len(evs_default) > 0

    # Call with full context: late turn (17), small wall (4), visible tiles
    # If 4p is all seen elsewhere, discarding 4p will affect evaluation
    visible_tiles = [
        TileName.FourP,
        TileName.FourP,
    ]  # 2 more 4p seen elsewhere -> total 4 of 4p seen
    evs_context = evaluate_hand_discards(
        hand,
        is_dealer=False,
        dora_indicators=[TileName.East],
        turn_number=17,
        remaining_wall_tiles=4,
        seat_wind=TileName.South,
        round_wind=TileName.East,
        visible_tiles=visible_tiles,
    )
    assert len(evs_context) > 0


def test_get_ai_hud_data_with_context():
    hand = [
        TileName.OneM,
        TileName.TwoM,
        TileName.ThreeM,
        TileName.FourM,
        TileName.FiveM,
        TileName.SixM,
        TileName.SevenM,
        TileName.EightM,
        TileName.NineM,
        TileName.OneP,
        TileName.TwoP,
        TileName.ThreeP,
        TileName.FourP,
        TileName.FourP,
    ]
    ctx = MatchContext(dealer_idx=0)
    hud = get_ai_hud_data(
        hand,
        ctx,
        player_idx=1,
        turn_number=10,
        remaining_wall_tiles=30,
        seat_wind=TileName.South,
        visible_tiles=[TileName.OneM] * 3,
    )
    assert "candidates" in hud
    assert "best_tile" in hud


def test_hud_data_exact_equivalence():
    hand = [
        TileName.OneM,
        TileName.TwoM,
        TileName.ThreeM,
        TileName.FourP,
        TileName.FiveP,
        TileName.SixP,
        TileName.SevenS,
        TileName.EightS,
        TileName.NineS,
        TileName.East,
        TileName.East,
        TileName.White,
        TileName.White,
        TileName.NineM,
    ]
    # オーラス局面 (南4局)
    ctx = MatchContext(
        scores=[31000, 25000, 24000, 20000],
        round_wind=TileName.South,
        round_number=4,
        dealer_idx=0,
    )

    evs = evaluate_placement_discards(hand, ctx, player_idx=1)
    hud = get_ai_hud_data(hand, ctx, player_idx=1)

    # 1. 基本ステータスの一致
    ranks = ctx.current_ranks()
    assert hud["current_rank"] == ranks[1]
    assert hud["current_score"] == 25000
    assert hud["is_orasu"] is True

    # 2. 候補リストの完全同値検証
    assert len(hud["candidates"]) == len(evs)
    expected_keys = {
        "discard_tile",
        "mpsz",
        "raw_ev",
        "placement_ev",
        "expected_rank",
        "rank_probabilities",
        "situational_note",
        "shanten_after",
        "remaining_count",
        "expected_score",
        "risk_score",
        "is_safe",
    }
    for cand_dict, ev in zip(hud["candidates"], evs):
        ev_dict = ev.to_dict()
        assert set(cand_dict.keys()) == expected_keys
        assert set(cand_dict.keys()) == set(ev_dict.keys())
        assert cand_dict["discard_tile"] == ev_dict["discard_tile"]
        assert cand_dict["mpsz"] == ev_dict["mpsz"]
        assert cand_dict["raw_ev"] == pytest.approx(ev_dict["raw_ev"])
        assert cand_dict["placement_ev"] == pytest.approx(ev_dict["placement_ev"])
        assert cand_dict["expected_rank"] == pytest.approx(ev_dict["expected_rank"])
        assert len(cand_dict["rank_probabilities"]) == 4
        for p_hud, p_ev in zip(
            cand_dict["rank_probabilities"], ev_dict["rank_probabilities"]
        ):
            assert p_hud == pytest.approx(p_ev)
        assert cand_dict["situational_note"] == ev_dict["situational_note"]
        assert cand_dict["shanten_after"] == ev_dict["shanten_after"]
        assert cand_dict["remaining_count"] == ev_dict["remaining_count"]
        assert cand_dict["expected_score"] == pytest.approx(ev_dict["expected_score"])
        assert cand_dict["risk_score"] == pytest.approx(ev_dict["risk_score"])
        assert cand_dict["is_safe"] == ev_dict["is_safe"]

    # 3. best_* フィールドの検証
    assert len(evs) > 0
    best_ev = evs[0]
    assert hud["best_tile"] == best_ev.discard_tile.as_str()
    assert hud["best_mpsz"] == best_ev.discard_tile.mpsz()
    assert hud["best_placement_ev"] == pytest.approx(best_ev.placement_ev)
    assert hud["best_raw_ev"] == pytest.approx(best_ev.raw_ev)
    assert hud["best_note"] == best_ev.situational_note

    # 4. orasu_conditions の完全同値検証
    conds = calculate_orasu_conditions(ctx, player_idx=1)
    assert "orasu_conditions" in hud
    assert len(hud["orasu_conditions"]) == len(conds)
    cond_keys = {
        "target_rank",
        "target_player",
        "diff",
        "ron_direct_req",
        "tsumo_req",
        "ron_other_req",
        "summary",
    }
    for c_hud, c in zip(hud["orasu_conditions"], conds):
        c_dict = c.to_dict()
        assert set(c_hud.keys()) == cond_keys
        assert set(c_hud.keys()) == set(c_dict.keys())
        assert c_hud["target_rank"] == c_dict["target_rank"]
        assert c_hud["target_player"] == c_dict["target_player"]
        assert c_hud["diff"] == c_dict["diff"]
        assert c_hud["ron_direct_req"] == c_dict["ron_direct_req"]
        assert c_hud["tsumo_req"] == c_dict["tsumo_req"]
        assert c_hud["ron_other_req"] == c_dict["ron_other_req"]
        assert c_hud["summary"] == c_dict["summary"]


def test_hud_data_exception_parity():
    hand = [TileName.OneM] * 14
    valid_ctx = MatchContext(dealer_idx=0)

    # 1. player_idx out of range (>= 4)
    with pytest.raises(ValueError, match="player_idx must be in range 0..4"):
        evaluate_placement_discards(hand, valid_ctx, player_idx=4)

    with pytest.raises(ValueError, match="player_idx must be in range 0..4"):
        get_ai_hud_data(hand, valid_ctx, player_idx=4)

    with pytest.raises(ValueError, match="player_idx must be in range 0..4"):
        calculate_orasu_conditions(valid_ctx, player_idx=4)

    # 2. dealer_idx out of range (>= 4) in MatchContext constructor & setters
    with pytest.raises(ValueError, match="dealer_idx must be in range 0..4"):
        MatchContext(dealer_idx=4)

    with pytest.raises(ValueError, match="dealer_idx must be in range 0..4"):
        valid_ctx.dealer_idx = 4
