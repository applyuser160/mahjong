#[cfg(test)]

mod tests {
    use std::collections::HashMap;

    use mahjong::tile::{TileName, TILE_NAME_NUMBER, TILE_PER_KIND, TILE_WALL_CAPACITY};
    use mahjong::wall::Wall;

    #[test]
    fn wall_contains_four_each() {
        let wall = Wall::new();
        let mut counter: HashMap<TileName, usize> = HashMap::new();

        for tile in wall.tiles() {
            *counter.entry(*tile).or_default() += 1;
        }

        assert_eq!(wall.remaining(), TILE_WALL_CAPACITY);
        assert_eq!(counter.len(), TILE_NAME_NUMBER);
        for value in counter.values() {
            assert_eq!(*value, TILE_PER_KIND);
        }
    }
}
