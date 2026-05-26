import re

with open("src/mahjong/wall.rs", "r") as f:
    code = f.read()

# Add a few Japanese doc comments
code = code.replace(
    "pub fn shuffle<R: Rng + ?Sized>",
    "/// 牌山をシャッフルし、ツモの位置を初期化します。\n    pub fn shuffle<R: Rng + ?Sized>"
)
code = code.replace(
    "pub fn draw(&mut self)",
    "/// 通常のツモを行います。\n    /// 王牌（ワンパイ）の14枚を除いた残りの山から牌を引きます。\n    pub fn draw(&mut self)"
)
code = code.replace(
    "pub fn draw_replacement(&mut self)",
    "/// カンが発生した際に、嶺上牌（リンシャンハイ）を引きます。\n    pub fn draw_replacement(&mut self)"
)
code = code.replace(
    "pub fn remaining(&self)",
    "/// 山の残り枚数を返します。\n    pub fn remaining(&self)"
)

with open("src/mahjong/wall.rs", "w") as f:
    f.write(code)
