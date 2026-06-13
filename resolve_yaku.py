with open("yaku_main.rs", "r") as f:
    main_code = f.read()

# We need to port the `closed_counts` parameter over to `yaku_main.rs`.
# 1. Modify the signature
old_sig = """pub fn judge_yaku(
    tiles: &[TileName],
    open_melds_input: &[crate::hand::Meld],
    mut ctx: WinContext,
) -> HashSet<YakuId> {"""
new_sig = """pub fn judge_yaku(
    tiles: &[TileName],
    closed_counts: &[u8; 35],
    open_melds_input: &[crate::hand::Meld],
    mut ctx: WinContext,
) -> HashSet<YakuId> {"""
main_code = main_code.replace(old_sig, new_sig)

# 2. Modify `counts` creation
old_counts = """    let mut counts = [0u8; 35];
    for tile in tiles.iter().copied() {
        let idx = tile as usize;
        if idx < counts.len() {
            counts[idx] += 1;
        }
    }"""
new_counts = """    let mut counts = [0u8; 35];
    counts.copy_from_slice(closed_counts);"""
main_code = main_code.replace(old_counts, new_counts)

# 3. Modify `closed_counts` creation in main
# Wait, `closed_counts` is now passed as parameter, so we don't need to create it!
# Main has:
old_closed_counts = """    let mut closed_counts = [0u8; 35];
    for tile in tiles.iter().copied() {
        let idx = tile as usize;
        if idx < closed_counts.len() {
            closed_counts[idx] += 1;
        }
    }"""
# Wait, `closed_counts` in main is named `closed_counts`.
# We pass `closed_counts` as a parameter.
# So we can just DELETE this part!
main_code = main_code.replace(old_closed_counts, "")

with open("src/mahjong/yaku.rs", "w") as f:
    f.write(main_code)
