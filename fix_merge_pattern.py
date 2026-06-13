import re
with open("src/mahjong/yaku.rs", "r") as f:
    text = f.read()

text = text.replace("generate_patterns(&closed_counts, &open_melds, &closed_melds, |pattern| {", "generate_patterns(&closed_counts_usize, &open_melds, &closed_melds, |pattern| {")

with open("src/mahjong/yaku.rs", "w") as f:
    f.write(text)
