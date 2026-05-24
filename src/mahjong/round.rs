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
        let previous_player = (self.turn + PLAYER_NUMBER - 1) % PLAYER_NUMBER;

        if let Meld::Chii { .. } = meld {
            if player_index != self.turn {
                return Err("Chii can only be called from the Kamicha (previous player)");
            }
        }

        let called_tile = match meld {
            Meld::Chii { called, .. } => called,
            Meld::Pon(called) => called,
            Meld::Daiminkan(called) => called,
            Meld::Ankan(called) => called,
            Meld::Kakan(called) => called,
        };

        // For Kakan and Ankan, they are called on the player's own turn (drawn tile),
        // not from the previous player's discard.
        let is_self_meld = matches!(meld, Meld::Ankan(_) | Meld::Kakan(_));

        if !is_self_meld {
            // Determine the last discarded tile
            let last_discard = self.rivers[previous_player]
                .tiles()
                .last()
                .copied()
                .ok_or("No tile in river to call")?;

            if last_discard != called_tile {
                return Err("Called tile does not match the last discarded tile");
            }
        }

        // Apply meld to hand (this will fail if hand doesn't have the consumed tiles)
        // Check condition before modifying the state
        match meld {
            Meld::Chii { called, .. } | Meld::Pon(called) | Meld::Daiminkan(called) => {
                // Determine the last discarded tile
                let last_discard = self.rivers[previous_player]
                    .tiles()
                    .last()
                    .copied()
                    .ok_or("No tile in river to call")?;

                if last_discard != called {
                    return Err("Called tile does not match the last discarded tile");
                }

                self.hands[player_index].call_meld(meld)?;

                // Remove the tile from the previous player's river
                self.rivers[previous_player].pop();
            }
            Meld::Ankan(_) | Meld::Kakan(_) => {
                // For Ankan and Kakan, we do not depend on the previous player's discard.
                // We just call the meld directly.
                self.hands[player_index].call_meld(meld)?;
            }
        }

        let hand = &mut self.hands[player_index];

        // For Kan, we draw a replacement tile.
        if let Meld::Daiminkan(_) | Meld::Ankan(_) | Meld::Kakan(_) = meld {
            if let Some(drawn) = self.wall.draw() {
                hand.push(drawn);
            } else {
                return Err("Wall is empty, cannot draw replacement tile for Kan");
            }
        }

        let discarded = hand.discard(discard_index);
        if let Err(_e) = discarded {
            return Err("Discard Error");
        }
        self.rivers[player_index].push(discarded.unwrap());

        // Update turn: the next turn belongs to the player after the one who called the meld
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
