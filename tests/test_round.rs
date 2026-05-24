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

            let discarded = r.play_turn(0).unwrap();

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

        let actual_discard = round.play_turn(0).unwrap();
        assert_eq!(actual_discard, discarded);
        assert_eq!(round.turn(), 2);

        let meld = Meld::Pon(discarded);
        let p2_discard = round.play_meld(3, meld, 0).unwrap();

        assert_eq!(round.turn(), 0);
        assert_eq!(round.river(1).tiles().len(), 0);
        assert_eq!(round.river(3).tiles().len(), 1);
        assert_eq!(round.river(3).tiles()[0], p2_discard);
    }

    #[test]
    fn test_play_meld_kakan_draw() {
        let seed = 1350;
        let discarded = TileName::SixS;

        let mut wall = Wall::new();
        wall.shuffle(&mut StdRng::seed_from_u64(seed));
        let mut round = Round::new(wall);

        // P1's turn
        let actual_discard = round.play_turn(0).unwrap();
        assert_eq!(actual_discard, discarded);

        // P3 calls Pon!
        let meld = Meld::Pon(discarded);
        let _ = round.play_meld(3, meld, 0).unwrap();
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
                let _ = round.play_turn(0);

                let p3_hand = round.hand(3);
                if p3_hand.contains(&discarded) {
                    let remaining_before = round.wall().remaining();

                    // P3 has drawn the 4th tile. P3 calls Kakan!
                    let kakan_meld = Meld::Kakan(discarded);
                    let kakan_discard = round.play_meld(3, kakan_meld, 0).unwrap();

                    // Verify wall decremented by 1 (replacement tile was drawn)
                    assert_eq!(round.wall().remaining(), remaining_before - 1);
                    assert_ne!(kakan_discard, TileName::None);
                    kakan_done = true;
                    break;
                }
            } else {
                if round.play_turn(0).is_none() {
                    break; // Wall empty
                }
            }
        }

        assert!(kakan_done, "Kakan should have been performed");
    }

    #[test]
    fn test_river_discards() {
        let wall = Wall::new();
        let mut round = Round::new(wall);

        let discarded = round.play_turn(0).unwrap();
        assert_eq!(round.river(1).tiles().len(), 1);
        assert_eq!(round.river(1).tiles()[0], discarded);

        let discarded2 = round.play_turn(0).unwrap();
        assert_eq!(round.river(2).tiles().len(), 1);
        assert_eq!(round.river(2).tiles()[0], discarded2);
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

        while let Some(tile) = round.play_turn(0) {
            draws += 1;
            assert_ne!(tile, TileName::None);
        }

        let total_tiles = PLAYER_NUMBER * 13 + draws;
        assert_eq!(total_tiles, TILE_WALL_CAPACITY);
    }
}
