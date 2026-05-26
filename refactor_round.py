import re

with open("src/mahjong/round.rs", "r") as f:
    code = f.read()

translations = {
    "// For Kakan and Ankan, they are called on the player's own turn (drawn tile),\n        // not from the previous player's discard.": "// 加カンと暗カンは自身のツモ番で実行され、\n        // 他家の捨て牌からは鳴きません。",
    "// Determine the last discarded tile": "// 直前に捨てられた牌を取得します",
    "// Remove the tile from the previous player's river": "// 直前のプレイヤーの河から牌を取り除きます",
    "// For Ankan and Kakan, we do not depend on the previous player's discard.\n                    // We just call the meld directly.": "// 暗カンと加カンは他家の捨て牌に依存せず、\n                    // 直接鳴きを処理します。",
    "// Draw a replacement tile for Kan": "// カンの嶺上牌（リンシャンハイ）をツモります",
    "// Set turn to the player who called the meld. They will need to discard a tile next.\n        // For Daiminkan, Ankan, Kakan, they need to draw a replacement tile first, then discard.": "// 手番を鳴いたプレイヤーに設定します。\n        // 大明カン、暗カン、加カンの場合、嶺上牌をツモった後に打牌する必要があります。"
}

for en, ja in translations.items():
    code = code.replace(en, ja)

# Simplify logic where possible, such as `deal` function
deal_old = """    fn deal(&mut self) {
        for _ in 0..DEAL_BASE {
            for hand in &mut self.hands {
                if let Some(tile) = self.wall.draw() {
                    hand.push(tile);
                }
            }
        }
    }"""
deal_new = """    fn deal(&mut self) {
        for _ in 0..DEAL_BASE {
            for hand in &mut self.hands {
                let Some(tile) = self.wall.draw() else { continue; };
                hand.push(tile);
            }
        }
    }"""
code = code.replace(deal_old, deal_new)

with open("src/mahjong/round.rs", "w") as f:
    f.write(code)
