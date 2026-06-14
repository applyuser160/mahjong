#[allow(unused_macros)]
macro_rules! to_counts {
    ($tiles:expr) => {{
        let mut counts = [0u8; 35];
        for &t in $tiles {
            counts[t as usize] += 1;
        }
        counts
    }};
}

#[cfg(test)]
mod tests {

    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use mahjong::hand::Meld;
    use mahjong::round::{Round, PLAYER_NUMBER};
    use mahjong::tile::{TileName, TILE_WALL_CAPACITY};
    use mahjong::wall::Wall;

    #[test]
    fn test_play_meld_turn_update() {
        let mut found_seed = None;
        for seed in 0..1000 {
            let mut w = Wall::new();
            w.shuffle(&mut StdRng::seed_from_u64(seed));
            let mut r = Round::new(w);

            let discarded = {
                r.draw_tile();
                r.discard_tile(0).unwrap()
            };

            let p2_hand = r.hand(3);
            let count = p2_hand.iter().filter(|&&t| t == discarded).count();
            if count >= 2 {
                found_seed = Some((seed, discarded));
                break;
            }
        }

        let (seed, discarded) = found_seed.unwrap();
        let mut wall = Wall::new();
        wall.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut round = Round::new(wall);

        let actual_discard = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        assert_eq!(actual_discard, discarded);
        assert_eq!(round.turn(), 1);

        let meld = Meld::Pon(discarded);
        round.play_meld(3, meld).unwrap();
        let p2_discard = round.discard_tile(0).unwrap();

        assert_eq!(round.turn(), 0);
        assert_eq!(round.river(0).tiles().len(), 0);
        assert_eq!(round.river(3).tiles().len(), 1);
        assert_eq!(round.river(3).tiles()[0], p2_discard);
    }

    #[test]
    fn test_play_meld_kakan_draw() {
        let seed = 128;
        let discarded = TileName::FourM;

        let mut wall = Wall::new();
        wall.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut round = Round::new(wall);

        // P0's turn
        let actual_discard = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        assert_eq!(actual_discard, discarded);

        // P3 calls Pon!
        let meld = Meld::Pon(discarded);
        round.play_meld(3, meld).unwrap();
        let _ = round.discard_tile(0).unwrap();
        assert_eq!(round.turn(), 0);

        // For testing Kakan with the current play_meld implementation, we just need to
        // play turns until the 4th tile is drawn into P3's hand.
        // To prevent P3 from immediately discarding the 4th tile upon drawing it,
        // we discard index 0. Since the drawn tile is pushed to the end of the hand,
        // discarding index 0 ensures the drawn tile stays in the hand.
        // Note: Realistically, Kakan is declared *instead* of discarding.
        // In the current `Round` struct logic, `play_meld` is called independently,
        // but it doesn't support an "interrupt" of one's own turn cleanly before discarding.
        // However, `play_meld` does successfully process a self-meld if the tile is in hand.

        let mut kakan_done = false;
        loop {
            let turn = round.turn();
            if turn == 3 {
                round.draw_tile();

                let p3_hand = round.hand(3);
                if p3_hand.contains(&discarded) {
                    let remaining_before = round.wall().remaining();

                    // P3 has drawn the 4th tile. P3 calls Kakan!
                    let kakan_meld = Meld::Kakan(discarded);
                    round.play_meld(3, kakan_meld).unwrap();
                    let kakan_discard = round.discard_tile(0).unwrap();

                    // Verify wall decremented by 1 (replacement tile was drawn)
                    assert_eq!(round.wall().remaining(), remaining_before - 1);
                    assert_ne!(kakan_discard, TileName::None);
                    kakan_done = true;
                    break;
                } else {
                    let _ = round.discard_tile(0).unwrap();
                }
            } else {
                if round.draw_tile().is_none() {
                    break; // Wall empty
                }
                let _ = round.discard_tile(0).unwrap();
            }
        }

        assert!(kakan_done, "Kakan should have been performed");
    }

    #[test]
    fn test_river_discards() {
        let wall = Wall::new();
        let mut round = Round::new(wall);

        let discarded = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        assert_eq!(round.river(0).tiles().len(), 1);
        assert_eq!(round.river(0).tiles()[0], discarded);

        let discarded2 = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        assert_eq!(round.river(1).tiles().len(), 1);
        assert_eq!(round.river(1).tiles()[0], discarded2);
    }

    #[test]
    fn play_full_round() {
        let mut wall = Wall::new();
        let mut rng = StdRng::seed_from_u64(7);
        wall.shuffle(&mut rng);

        let mut round = Round::new(wall);
        let mut draws = 0;

        assert_eq!(round.hand(0).len(), 13);
        for index in 1..PLAYER_NUMBER {
            assert_eq!(round.hand(index).len(), 13);
        }

        while let Some(_drawn) = round.draw_tile() {
            let tile = round.discard_tile(0).unwrap();
            draws += 1;
            assert_ne!(tile, TileName::None);
        }

        let total_tiles = PLAYER_NUMBER * 13 + draws;
        assert_eq!(total_tiles, TILE_WALL_CAPACITY - 14);
    }

    #[test]
    fn test_kan_draw_reduces_remaining() {
        // Since we want to test Kan replacement draw logic, let's just create a deterministic setup
        // Player 1 will Ankan, which means they need 4 identical tiles.
        // The first 52 tiles are dealt to players.
        // P0: 0, 4, 8...
        // P1: 1, 5, 9...
        // By default, Wall is ordered: 1m, 1m, 1m, 1m, 2m, 2m, 2m, 2m, ...
        // Let's just find a seed where P1 gets an Ankan initially to simplify this test.
        let mut found_ankan = None;
        for seed in 0..1000 {
            let mut w = Wall::new();
            w.shuffle(&mut rand::rngs::StdRng::seed_from_u64(seed));
            let r = Round::new(w);
            let p0_hand = r.hand(0);
            let mut counts = [0; 40];
            for &t in p0_hand {
                counts[t as usize] += 1;
            }
            if counts.contains(&4) {
                let tile_idx = counts.iter().position(|&c| c == 4).unwrap();
                found_ankan = Some((seed, TileName::from_usize(tile_idx)));
                break;
            }
        }

        let (seed, tile) = found_ankan.expect("Should find a seed with an Ankan in initial hand");
        let mut wall = Wall::new();
        wall.shuffle(&mut rand::rngs::StdRng::seed_from_u64(seed));
        let mut round = Round::new(wall);

        let remaining_before = round.wall().remaining();

        let meld = Meld::Ankan(tile);
        let res = round.play_meld(0, meld);

        assert!(res.is_ok(), "Ankan should succeed");

        let remaining_after = round.wall().remaining();
        assert_eq!(
            remaining_after,
            remaining_before - 1,
            "Drawing a replacement tile for a Kan should reduce the remaining drawable tiles by 1"
        );
    }

    #[test]
    fn test_play_meld_ankan() {
        let mut found_ankan = None;
        for seed in 0..100000 {
            let mut w = Wall::new();
            w.shuffle(&mut StdRng::seed_from_u64(seed));
            let r = Round::new(w);
            let p0_hand = r.hand(0);
            let mut counts = [0; 40];
            for &t in p0_hand {
                counts[t as usize] += 1;
            }
            if counts.iter().any(|&c| c >= 4) {
                let tile_idx = counts.iter().position(|&c| c >= 4).unwrap();
                found_ankan = Some((seed, TileName::from_usize(tile_idx)));
                break;
            }
        }

        let (seed, tile) = found_ankan.unwrap();
        let mut wall = Wall::new();
        wall.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut round = Round::new(wall);

        let r0_len_before = round.river(0).tiles().len();
        let r1_len_before = round.river(1).tiles().len();
        let r2_len_before = round.river(2).tiles().len();
        let r3_len_before = round.river(3).tiles().len();

        let meld = Meld::Ankan(tile);
        let res = round.play_meld(0, meld);

        assert!(res.is_ok(), "Ankan should succeed");

        assert_eq!(
            round.river(0).tiles().len(),
            r0_len_before,
            "River should not be popped"
        );
        assert_eq!(
            round.river(1).tiles().len(),
            r1_len_before,
            "River should not be popped"
        );
        assert_eq!(
            round.river(2).tiles().len(),
            r2_len_before,
            "River should not be popped"
        );
        assert_eq!(
            round.river(3).tiles().len(),
            r3_len_before,
            "River should not be popped"
        );
    }

    #[test]
    fn test_play_meld_kakan() {
        let mut found_kakan = None;
        for seed in 0..100000 {
            let mut w = Wall::new();
            w.shuffle(&mut StdRng::seed_from_u64(seed));
            let r = Round::new(w);
            let p0_hand = r.hand(0);
            let p3_hand = r.hand(3);

            let mut p0_counts = [0; 40];
            for &t in p0_hand {
                p0_counts[t as usize] += 1;
            }

            if p0_counts.iter().any(|&c| c >= 3) {
                let tile_idx = p0_counts.iter().position(|&c| c >= 3).unwrap();
                let target_tile = TileName::from_usize(tile_idx);
                // P0 needs the tile to discard on their turn (which is the first turn)
                if p3_hand[0] == target_tile {
                    found_kakan = Some((seed, target_tile));
                    break;
                }
            }
        }

        let (seed, tile) = found_kakan.unwrap();
        let mut wall = Wall::new();
        wall.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut round = Round::new(wall);

        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        }; // P0 plays, turn -> 1
        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        }; // P1 plays, turn -> 2
        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        }; // P2 plays, turn -> 3

        let p0_discard = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        assert_eq!(p0_discard, tile);

        let pon_meld = Meld::Pon(tile);
        let res = round.play_meld(0, pon_meld);
        assert!(res.is_ok());
        round.discard_tile(0).unwrap(); // P0 discards after Pon, turn -> 1

        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        }; // P1 plays, turn -> 2
        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        }; // P2 plays, turn -> 3
        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        }; // P3 plays, turn -> 0

        let kakan_meld = Meld::Kakan(tile);
        let r0_len = round.river(0).tiles().len();
        let r1_len = round.river(1).tiles().len();
        let r2_len = round.river(2).tiles().len();
        let r3_len = round.river(3).tiles().len();

        let res_kakan = round.play_meld(0, kakan_meld);
        assert!(res_kakan.is_ok(), "Kakan should succeed");

        assert_eq!(
            round.river(0).tiles().len(),
            r0_len,
            "River should not be popped"
        );
        assert_eq!(
            round.river(1).tiles().len(),
            r1_len,
            "River should not be popped"
        );
        assert_eq!(
            round.river(2).tiles().len(),
            r2_len,
            "River should not be popped"
        );
        assert_eq!(
            round.river(3).tiles().len(),
            r3_len,
            "River should not be popped"
        );
    }

    #[test]
    fn test_chii_validation_kamicha() {
        let wall = Wall::new(); // Deterministic wall (unshuffled)
        let mut round = Round::new(wall);

        // Player 1 plays their turn.
        // With an unshuffled wall, Player 1 discards OneM.
        // Turn updates to Player 2.
        let discarded = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        assert_eq!(discarded, TileName::OneM);
        assert_eq!(round.turn(), 1);

        // A valid Chii using OneM (called) and TwoM, ThreeM (consumed).
        // Since Player 1 is the next turn (which is Player 0's Shimodate/Kamicha relationship),
        // Player 1 is allowed to call Chii.
        let chii_meld = Meld::Chii {
            called: TileName::OneM,
            consumed: [TileName::TwoM, TileName::ThreeM],
        };

        // Player 2 (index 2) attempts to call Chii.
        // Player 2 is NOT the immediate next player in order (not self.turn()), so it must fail.
        let res2 = round.play_meld(2, chii_meld);
        assert!(res2.is_err());
        assert_eq!(
            res2.unwrap_err(),
            "Chii can only be called from the Kamicha (previous player)"
        );

        // Player 3 (index 3) attempts to call Chii.
        // Player 3 is NOT the immediate next player in order (not self.turn()), so it must fail.
        let res3 = round.play_meld(3, chii_meld);
        assert!(res3.is_err());
        assert_eq!(
            res3.unwrap_err(),
            "Chii can only be called from the Kamicha (previous player)"
        );

        // Player 0 (index 0) attempts to call Chii.
        // Player 0 is NOT the immediate next player in order (not self.turn()), so it must fail.
        let res0 = round.play_meld(0, chii_meld);
        assert!(res0.is_err());
        assert_eq!(
            res0.unwrap_err(),
            "Chii can only be called from the Kamicha (previous player)"
        );

        // Player 1 (index 1) calls Chii. This is valid.
        let res1 = round.play_meld(1, chii_meld);
        assert!(res1.is_ok());
    }

    #[test]
    fn test_play_meld_multiple_discards_in_river() {
        let mut found_seed = None;
        for seed in 0..10000 {
            let mut w = Wall::new();
            w.shuffle(&mut StdRng::seed_from_u64(seed));
            let mut r = Round::new(w);

            // Let's play 4 turns so everyone has discarded 1 tile.
            let _d1 = {
                r.draw_tile();
                r.discard_tile(0).unwrap()
            }; // P1
            let _d2 = {
                r.draw_tile();
                r.discard_tile(0).unwrap()
            }; // P2
            let _d3 = {
                r.draw_tile();
                r.discard_tile(0).unwrap()
            }; // P3
            let _d0 = {
                r.draw_tile();
                r.discard_tile(0).unwrap()
            }; // P0

            // P1 plays again and discards their second tile.
            let second_discard = {
                r.draw_tile();
                r.discard_tile(0).unwrap()
            }; // P1

            // P2 has hand. Let's see if P2 has at least 2 of second_discard.
            let p2_hand = r.hand(2);
            let count = p2_hand.iter().filter(|&&t| t == second_discard).count();
            if count >= 2 {
                found_seed = Some((seed, second_discard));
                break;
            }
        }

        let (seed, discarded) =
            found_seed.expect("Should find a seed where P2 can Pon P0's second discard");
        let mut wall = Wall::new();
        wall.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut round = Round::new(wall);

        // Play 5 turns to get P1's river to have 2 tiles, and the last discard is `discarded`
        let first_discard = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        let second_discard = {
            round.draw_tile();
            round.discard_tile(0).unwrap()
        };
        assert_eq!(second_discard, discarded);

        // Check P0 (index 0) river has 2 tiles
        assert_eq!(round.river(0).tiles().len(), 2);
        assert_eq!(round.river(0).tiles()[0], first_discard);
        assert_eq!(round.river(0).tiles()[1], second_discard);

        // P2 (index 2) calls Pon on P0's discard
        let meld = Meld::Pon(discarded);
        round.play_meld(2, meld).unwrap();
        let p2_discard = round.discard_tile(0).unwrap();

        // Verify that ONLY the last discard was removed from P0's river.
        // First discard should still be there!
        assert_eq!(
            round.river(0).tiles().len(),
            1,
            "Only 1 tile should be removed from P0's river"
        );
        assert_eq!(
            round.river(0).tiles()[0],
            first_discard,
            "P0's first discard should remain in the river"
        );

        // Verify P2's river has their discard
        assert_eq!(round.river(2).tiles().len(), 2);
        assert_eq!(round.river(2).tiles()[1], p2_discard);
    }
}
