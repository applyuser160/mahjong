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
        // Find a seed where Player 2 can pon Player 0's discard.
        let mut found_seed = None;
        for seed in 0..1000 {
            let mut w = Wall::new();
            w.shuffle(&mut StdRng::seed_from_u64(seed));
            let mut r = Round::new(w);

            // turn is 1, so P1 plays first! Wait, the prompt says "turnの初期値は1であるべき" (initial value of turn should be 1).
            // P1 draws and discards the first tile (index 0)
            let discarded = r.play_turn(0).unwrap();

            // Does P3 have at least two of `discarded`?
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

        // P1's turn
        let actual_discard = round.play_turn(0).unwrap();
        assert_eq!(actual_discard, discarded);
        assert_eq!(round.turn(), 2);

        // P3 calls Pon!
        // Since P3 has 2 copies, they can call Pon. We discard index 0 from P3's hand after Pon.
        let meld = Meld::Pon(discarded);

        let p2_discard = round.play_meld(3, meld, 0).unwrap();

        // Now turn should be 0 (P3's turn is over, next is P0)
        assert_eq!(round.turn(), 0);

        // P1's river should be missing the discarded tile (it was popped)
        assert_eq!(round.river(1).tiles().len(), 0);

        // P3's river should have the discarded tile
        assert_eq!(round.river(3).tiles().len(), 1);
        assert_eq!(round.river(3).tiles()[0], p2_discard);
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
