#[cfg(test)]

mod tests {
    use std::iter::zip;

    use crate::tile::{Tile, TileCategory, TileName, TileType, TILE_NAME_NUMBER};

    #[test]
    fn tile_name_case01() {
        let tile_numbers = 0..=TILE_NAME_NUMBER + 1;
        let tile_strings = vec![
            " ",  // TileName::None
            "1m", // TileName::OneM
            "2m", // TileName::TwoM
            "3m", // TileName::ThreeM
            "4m", // TileName::FourM
            "5m", // TileName::FiveM
            "6m", // TileName::SixM
            "7m", // TileName::SevenM
            "8m", // TileName::EightM
            "9m", // TileName::NineM
            "1p", // TileName::OneP
            "2p", // TileName::TwoP
            "3p", // TileName::ThreeP
            "4p", // TileName::FourP
            "5p", // TileName::FiveP
            "6p", // TileName::SixP
            "7p", // TileName::SevenP
            "8p", // TileName::EightP
            "9p", // TileName::NineP
            "1s", // TileName::OneS
            "2s", // TileName::TwoS
            "3s", // TileName::ThreeS
            "4s", // TileName::FourS
            "5s", // TileName::FiveS
            "6s", // TileName::SixS
            "7s", // TileName::SevenS
            "8s", // TileName::EightS
            "9s", // TileName::NineS
            "東", // TileName::East
            "南", // TileName::South
            "西", // TileName::West
            "北", // TileName::North
            "中", // TileName::Red
            "発", // TileName::Green
            "白", // TileName::White
            " ",  // TileName::None
        ];

        for (n, s) in zip(tile_numbers, tile_strings) {
            let tile_name = TileName::from_usize(n);

            let assert_number = if n > TILE_NAME_NUMBER { 0 } else { n };
            assert_eq!(tile_name as usize, assert_number);
            assert_eq!(tile_name.to_string(), s);
        }
    }

    #[test]
    fn tile_case01() {
        let tile_numbers = 0..=TILE_NAME_NUMBER;
        for n in tile_numbers {
            let name = TileName::from_usize(n);

            let mut tile = Tile::from(
                name,
                TileType::from_tile_name(name),
                TileCategory::from_tile_name(name),
                false,
            );
            assert_eq!(tile.to_u8(), n as u8);
            assert_eq!(tile.tile_type, TileType::from_tile_name(name));
            assert_eq!(tile.category, TileCategory::from_tile_name(name));

            tile = Tile::from(
                name,
                TileType::from_tile_name(name),
                TileCategory::from_tile_name(name),
                true,
            );
            assert_eq!(tile.to_u8(), (n + 128) as u8);
            assert_eq!(tile.tile_type, TileType::from_tile_name(name));
            assert_eq!(tile.category, TileCategory::from_tile_name(name));
        }
    }

    #[test]
    fn tile_case02() {
        let tile_numbers = 0..=TILE_NAME_NUMBER;
        for n in tile_numbers {
            let name = TileName::from_usize(n);

            let mut tile = Tile::from_name(name, false);
            assert_eq!(tile.to_u8(), n as u8);
            assert_eq!(tile.tile_type, TileType::from_tile_name(name));
            assert_eq!(tile.category, TileCategory::from_tile_name(name));

            tile = Tile::from_name(name, true);
            assert_eq!(tile.to_u8(), (n + 128) as u8);
            assert_eq!(tile.tile_type, TileType::from_tile_name(name));
            assert_eq!(tile.category, TileCategory::from_tile_name(name));
        }
    }
}
