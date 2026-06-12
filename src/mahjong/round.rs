use crate::hand::{Hand, Meld};
use crate::river::River;
use crate::tile::TileName;
use crate::wall::Wall;

pub const PLAYER_NUMBER: usize = 4;
const DEAL_BASE: usize = 13;

#[derive(Debug)]
/// 1局（1回のゲーム）の進行状態を表す構造体です。
pub struct Round {
    wall: Wall,
    hands: [Hand; PLAYER_NUMBER],
    rivers: [River; PLAYER_NUMBER],
    turn: usize,
}

impl Round {
    pub fn new(wall: Wall) -> Self {
        let mut round = Self {
            wall,
            hands: [Hand::new(), Hand::new(), Hand::new(), Hand::new()],
            rivers: [River::new(), River::new(), River::new(), River::new()],
            turn: 0,
        };

        round.deal();
        round
    }

    pub fn hand(&self, index: usize) -> &[TileName] {
        self.hands[index].tiles()
    }

    pub fn wall(&self) -> &Wall {
        &self.wall
    }

    pub fn river(&self, index: usize) -> &River {
        &self.rivers[index]
    }

    pub fn turn(&self) -> usize {
        self.turn
    }

    pub fn draw_tile(&mut self) -> Option<TileName> {
        let drawn = self.wall.draw()?;
        self.hands[self.turn].push(drawn);
        Some(drawn)
    }

    pub fn discard_tile(&mut self, discard_index: usize) -> Result<TileName, &'static str> {
        let hand = &mut self.hands[self.turn];
        let discarded = hand.discard(discard_index).map_err(|_| "Discard Error")?;
        self.rivers[self.turn].push(discarded);
        self.turn = (self.turn + 1) % PLAYER_NUMBER;
        Ok(discarded)
    }

    pub fn play_meld(&mut self, player_index: usize, meld: Meld) -> Result<(), &'static str> {
        if matches!(meld, Meld::Ankan(_) | Meld::Kakan(_)) && player_index != self.turn {
            return Err("Ankan and Kakan can only be called on your own turn");
        }

        let previous_player = (self.turn + PLAYER_NUMBER - 1) % PLAYER_NUMBER;

        if let Meld::Chii { .. } = meld {
            if player_index != self.turn {
                return Err("Chii can only be called from the Kamicha (previous player)");
            }
        }

        if let Meld::Ankan(_) | Meld::Kakan(_) = meld {
            if player_index != self.turn {
                return Err("Self meld (Ankan/Kakan) can only be called on the player's own turn");
            }
        }

        let called_tile = match meld {
            Meld::Chii { called, .. } => called,
            Meld::Pon(called) => called,
            Meld::Daiminkan(called) => called,
            Meld::Ankan(called) => called,
            Meld::Kakan(called) => called,
        };

        // 加カンと暗カンは自身のツモ番で実行され、
        // 他家の捨て牌からは鳴きません。
        let is_self_meld = matches!(meld, Meld::Ankan(_) | Meld::Kakan(_));

        if !is_self_meld {
            // 直前に捨てられた牌を取得します
            let last_discard = self.rivers[previous_player]
                .tiles()
                .last()
                .copied()
                .ok_or("No tile in river to call")?;

            if last_discard != called_tile {
                return Err("Called tile does not match the last discarded tile");
            }
        }

        let is_kan = matches!(meld, Meld::Daiminkan(_) | Meld::Ankan(_) | Meld::Kakan(_));

        if is_kan && !self.wall.can_draw_replacement() {
            return Err("No replacement tile available in wall");
        }

        // `call_meld` は失敗時に内部状態を変更しないよう実装されているため、そのまま呼び出せます。
        self.hands[player_index].call_meld(meld)?;

        // 自身のツモ番ではない（他家の捨て牌からの鳴き）場合、
        // 捨て牌を河から取り除きます。
        if !is_self_meld {
            self.rivers[previous_player].pop();
        }

        // カンの場合は嶺上牌を引きます。
        if is_kan {
            if let Some(replacement) = self.wall.draw_replacement() {
                self.hands[player_index].push(replacement);
            }
        }

        // 手番を鳴いたプレイヤーに設定します。
        // 大明カン、暗カン、加カンの場合、嶺上牌をツモった後に打牌する必要があります。
        self.turn = player_index;

        Ok(())
    }

    fn deal(&mut self) {
        for _ in 0..DEAL_BASE {
            for hand in &mut self.hands {
                let Some(tile) = self.wall.draw() else {
                    continue;
                };
                hand.push(tile);
            }
        }
    }
}
