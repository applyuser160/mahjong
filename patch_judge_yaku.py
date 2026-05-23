with open("src/mahjong/yaku.rs", "r") as f:
    content = f.read()

# Instead of blindly setting win_tile = tiles.last() in judge_yaku,
# I will let it be None if the caller doesn't provide it.
# Wait, if detect_pinfu requires win_tile (since I used ctx.win_tile?),
# any test that expects Pinfu but doesn't set win_tile will fail Pinfu.
# BUT none of the tests calling WinContext::default() are testing Pinfu.
# Let's check `tests/test_yaku.rs`.
# - detect_chitoitsu
# - detect_kokushi
# - detect_daisangen
# - detect_sanshoku_doujun
# - detect_ipeiko
# - detect_ryanpeiko
# - detect_yakuhai_with_seat_and_round_wind
# - detect_honitsu
# - detect_sanshoku_doukou
# - detect_chinroutou

# Wait, `detect_ipeiko` tests Ipeiko, which implies a sequence. Does it get Pinfu?
# In `detect_ipeiko`:
#         let result = judge_yaku(&tiles, &[], WinContext::default());
#         assert!(result.contains(&YakuId::Ipeiko));
# If it also got Pinfu, it would just not get Pinfu anymore because win_tile is None.
# But it only asserts Ipeiko.
