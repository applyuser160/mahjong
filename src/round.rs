use crate::hand::Hand;
use crate::tile::TileName;
use crate::wall::Wall;

pub const PLAYER_NUMBER: usize = 4;
const DEAL_BASE: usize = 13;

#[derive(Debug)]
pub struct Round {
    wall: Wall,
    hands: [Hand; PLAYER_NUMBER],
    turn: usize,
}

impl Round {
    pub fn new(mut wall: Wall) -> Self {
        let mut round = Self {
            wall,
            hands: [Hand::new(), Hand::new(), Hand::new(), Hand::new()],
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

    pub fn play_turn(&mut self, discard_index: usize) -> Option<TileName> {
        let drawn = self.wall.draw()?;
        let hand = &mut self.hands[self.turn];
        hand.push(drawn);
        let discarded = hand.discard(discard_index);
        self.turn = (self.turn + 1) % PLAYER_NUMBER;
        Some(discarded)
    }

    fn deal(&mut self) {
        for _ in 0..DEAL_BASE {
            for hand in &mut self.hands {
                if let Some(tile) = self.wall.draw() {
                    hand.push(tile);
                }
            }
        }

        if let Some(tile) = self.wall.draw() {
            self.hands[0].push(tile);
        }
    }
}
