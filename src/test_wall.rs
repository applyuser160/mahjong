#[cfg(test)]

mod tests {
    use std::iter::zip;

    use crate::{
        tile::TILE_NAME_NUMBER,
        wall::{Wall, RED_TILE},
    };

    #[test]
    fn wall_case01() {
        let wall = Wall::new();
        for (tile, n) in zip(
            wall.tiles,
            (1..=TILE_NAME_NUMBER)
                .flat_map(|i| vec![i; 4])
                .collect::<Vec<_>>(),
        ) {
            assert_eq!(tile.to_u8(), n as u8);
        }
    }

    #[test]
    fn wall_case02() {
        let wall = Wall::from(true);
        let mut used_reds: Vec<usize> = Vec::new();
        for (tile, n) in zip(
            wall.tiles,
            (1..=TILE_NAME_NUMBER)
                .flat_map(|i| vec![i; 4])
                .collect::<Vec<_>>(),
        ) {
            let mut assert_n = n;
            let is_contain = RED_TILE.contains(&(n as u8));
            let is_zero = used_reds.contains(&n);
            if is_contain && !is_zero {
                assert_n += 128;
                used_reds.push(n);
            }
            assert_eq!(tile.to_u8(), assert_n as u8);
        }
    }

    #[test]
    fn wall_case03() {
        let tile_numbers: [u8; TILE_NAME_NUMBER] = [
            1, 2, 3, 4, 3, 2, 1, 0, 0, 3, 1, 2, 3, 4, 3, 2, 1, 0, 0, 2, 1, 2, 3, 4, 3, 2, 1, 0, 0,
            3, 4, 3, 2, 1,
        ];
        let wall = Wall::from_custom(tile_numbers, true);
        assert_eq!(wall.tiles.len(), 66);
    }
}
