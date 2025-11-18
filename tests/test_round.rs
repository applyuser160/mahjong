#[cfg(test)]

mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use mahjong::round::{Round, PLAYER_NUMBER};
    use mahjong::tile::{TileName, TILE_WALL_CAPACITY};
    use mahjong::wall::Wall;

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
