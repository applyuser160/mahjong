import os
import glob
import re

def translate_comments():
    files = ["src/mahjong/hand.rs", "src/mahjong/river.rs", "src/mahjong/round.rs", "src/mahjong/tile.rs", "src/mahjong/wall.rs"]

    translations = {
        "pub struct Hand": "/// プレイヤーの手牌を表す構造体です。\npub struct Hand",
        "pub enum Meld": "/// 鳴き（副露）の種類を表す列挙型です。\npub enum Meld",
        "pub struct River": "/// 河（捨て牌の置き場）を表す構造体です。\npub struct River",
        "pub struct Round": "/// 1局（1回のゲーム）の進行状態を表す構造体です。\npub struct Round",
        "pub enum TileName": "/// 牌の種類（名前）を表す列挙型です。\npub enum TileName",
        "pub struct Wall": "/// 山（壁）を表す構造体です。\npub struct Wall",
        "pub enum MeldError": "/// 鳴きに関するエラーを表す列挙型です。\npub enum MeldError",
        "pub struct DiscardResult": "/// 打牌の結果を表す構造体です。\npub struct DiscardResult",
        "pub struct DrawResult": "/// ツモ（ドロー）の結果を表す構造体です。\npub struct DrawResult",
        "pub struct MeldResult": "/// 鳴き（副露）の結果を表す構造体です。\npub struct MeldResult",

        # specific file comments
        "// A simple wrapper around Vec": "/// Vecのシンプルなラッパーです",
        "// Kan draw reduces remaining tiles like a normal draw": "/// カンでのツモも通常のツモと同様に残り枚数を減らします",
        "// Tsumo checks": "/// ツモのチェック",
        "// Must be closed": "/// 門前（メンゼン）である必要があります",
        "// Check tenpai": "/// テンパイしているかチェックします",
        "// Requires exactly the requested tiles": "/// 必要な牌が正確に揃っている必要があります",
        "// Return the specific error type to be handled by Python": "/// Python側で処理される具体的なエラー型を返します",
        "// Kan variants require specifically 4 identical tiles (or 1 for Kakan)": "/// カンは4枚の同じ牌（加カンの場合は1枚）が必要です",
        "// Check if it's the player's turn to draw": "/// プレイヤーのツモ番かどうかをチェックします",
        "// Handle different types of melds": "/// 異なる種類の鳴き（副露）を処理します",
        "// Check if the meld is valid": "/// 鳴き（副露）が有効かどうかをチェックします",
        "// Ensure indices are within bounds": "/// インデックスが範囲内にあることを確認します",
    }

    for file in files:
        if not os.path.exists(file):
            continue
        with open(file, "r") as f:
            content = f.read()

        for en, ja in translations.items():
            content = content.replace(en, ja)

        with open(file, "w") as f:
            f.write(content)

if __name__ == "__main__":
    translate_comments()
