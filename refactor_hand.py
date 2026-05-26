import re

with open("src/mahjong/hand.rs", "r") as f:
    code = f.read()

# Add Japanese translations to `call_meld`
code = code.replace(
    "// Verify that we have the consumed tiles, accounting for duplicates",
    "// 手牌に必要な牌が含まれているか、重複を考慮して検証します"
)
code = code.replace(
    "// Remove the consumed tiles",
    "// 消費された牌を手牌から取り除きます"
)
code = code.replace(
    "// Must have a corresponding Pon",
    "// 対応するポンがすでに存在している必要があります"
)

# Convert iterator to a more declarative style where appropriate (if possible),
# but the existing loop is already quite simple and requires mutating a vec to check duplicates.

with open("src/mahjong/hand.rs", "w") as f:
    f.write(code)
