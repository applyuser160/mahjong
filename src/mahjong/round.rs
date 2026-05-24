use crate::hand::{Hand, Meld};
use crate::river::River;
use crate::tile::TileName;
use crate::wall::Wall;

pub const PLAYER_NUMBER: usize = 4;
const DEAL_BASE: usize = 13;

#[derive(Debug)]
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
            turn: 1,
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

    pub fn play_turn(&mut self, discard_index: usize) -> Option<TileName> {
        let drawn = self.wall.draw()?;
        let hand = &mut self.hands[self.turn];
        hand.push(drawn);
        let discarded = hand.discard(discard_index).ok()?;
        self.rivers[self.turn].push(discarded);
        self.turn = (self.turn + 1) % PLAYER_NUMBER;
        Some(discarded)
    }

    pub fn play_meld(
        &mut self,
        player_index: usize,
        meld: Meld,
        discard_index: usize,
    ) -> Result<TileName, &'static str> {
        // 1. 他家の捨て牌を必要とするアクション（ポン・チー・大明槓）かどうかを判定
        let is_open_call = match meld {
            Meld::Chii { .. } | Meld::Pon(_) | Meld::Daiminkan(_) => true,
            Meld::Ankan(_) | Meld::Kakan(_) => false,
        };

        if is_open_call {
            let previous_player = (self.turn + PLAYER_NUMBER - 1) % PLAYER_NUMBER;
            let called_tile = match meld {
                Meld::Chii { called, .. } => called,
                Meld::Pon(called) => called,
                Meld::Daiminkan(called) => called,
                _ => unreachable!(),
            };

            // 直前のプレイヤーの河から牌を確認
            let last_discard = self.rivers[previous_player]
                .tiles()
                .last()
                .copied()
                .ok_or("No tile in river to call")?;

            if last_discard != called_tile {
                return Err("Called tile does not match the last discarded tile");
            }

            // 手牌に副露を適用
            self.hands[player_index].call_meld(meld)?;

            // 直前のプレイヤーの河から捨て牌を奪う
            self.rivers[previous_player].pop();
        } else {
            // 暗槓・加槓の場合は、他家の河を参照せず、自分の手牌のみでカンを適用
            self.hands[player_index].call_meld(meld)?;
        }

        let hand = &mut self.hands[player_index];

        // 2. カンの場合の補充牌（嶺上ツモ）処理
        // ※ 以前のバグ修正：ここに Meld::Kakan(_) を追加して加槓でもツモるようにする
        if let Meld::Daiminkan(_) | Meld::Ankan(_) | Meld::Kakan(_) = meld {
            if let Some(drawn) = self.wall.draw() {
                hand.push(drawn);
            } else {
                return Err("Wall is empty, cannot draw replacement tile for Kan");
            }
        }

        // 打牌処理
        let discarded = hand.discard(discard_index);
        if let Err(_e) = discarded {
            return Err("Discard Error");
        }
        self.rivers[player_index].push(discarded.unwrap());

        // 順番の更新
        self.turn = (player_index + 1) % PLAYER_NUMBER;

        Ok(discarded.unwrap())
    }

    fn deal(&mut self) {
        for _ in 0..DEAL_BASE {
            for hand in &mut self.hands {
                if let Some(tile) = self.wall.draw() {
                    hand.push(tile);
                }
            }
        }
    }
}
