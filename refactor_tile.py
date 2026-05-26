import re

with open("src/mahjong/tile.rs", "r") as f:
    code = f.read()

translations = {
    "pub enum TileType": "/// 牌の種類（数牌の各スーツ、風牌、三元牌）を表す列挙型です。\npub enum TileType",
    "pub enum TileCategory": "/// 牌のカテゴリ（数牌か字牌か）を表す列挙型です。\npub enum TileCategory",
    "pub struct Tile": "/// 牌を表す構造体です。\npub struct Tile"
}

for en, ja in translations.items():
    code = code.replace(en, ja)

with open("src/mahjong/tile.rs", "w") as f:
    f.write(code)
