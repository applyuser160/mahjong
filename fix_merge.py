with open("src/mahjong/yaku.rs", "r") as f:
    text = f.read()

conflict = """<<<<<<< HEAD
    // そうしないと、副露した牌を再度パースしようとしてしまいます。
    let patterns = generate_patterns(&closed_counts_usize, &open_melds, &closed_melds);
=======
    let mut max_han = -1;
    let mut best_yaku = HashSet::new();
>>>>>>> origin/main"""

resolution = """    // そうしないと、副露した牌を再度パースしようとしてしまいます。
    let mut max_han = -1;
    let mut best_yaku = HashSet::new();"""
text = text.replace(conflict, resolution)

with open("src/mahjong/yaku.rs", "w") as f:
    f.write(text)
