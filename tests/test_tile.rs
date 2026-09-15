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

    use std::iter::zip;

    use mahjong::tile::{Tile, TileCategory, TileName, TileType, TILE_NAME_NUMBER};

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
            assert_eq!(tile_name.as_str(), s);
        }
    }

    #[test]
    fn tile_metadata_matches_name() {
        let tile = Tile::new(TileName::Red);
        assert_eq!(tile.name(), TileName::Red);
        assert_eq!(tile.tile_type(), TileType::Dragons);
        assert_eq!(tile.category(), TileCategory::Honors);

        let tile = Tile::new(TileName::EightM);
        assert_eq!(tile.tile_type(), TileType::Characters);
        assert_eq!(tile.category(), TileCategory::Simples);
    }

    #[test]
    fn test_from_usize_out_of_bounds_exhaustion() {
        for n in 35..=255 {
            assert_eq!(TileName::from_usize(n), TileName::None);
        }
    }

    #[test]
    fn test_tile_type_and_category_exhaustive() {
        for n in 1..=34 {
            let t = TileName::from_usize(n);
            match n {
                1..=9 => {
                    assert_eq!(t.tile_type(), TileType::Characters);
                    assert_eq!(t.category(), TileCategory::Simples);
                }
                10..=18 => {
                    assert_eq!(t.tile_type(), TileType::Circles);
                    assert_eq!(t.category(), TileCategory::Simples);
                }
                19..=27 => {
                    assert_eq!(t.tile_type(), TileType::Bamboos);
                    assert_eq!(t.category(), TileCategory::Simples);
                }
                28..=31 => {
                    assert_eq!(t.tile_type(), TileType::Winds);
                    assert_eq!(t.category(), TileCategory::Honors);
                }
                32..=34 => {
                    assert_eq!(t.tile_type(), TileType::Dragons);
                    assert_eq!(t.category(), TileCategory::Honors);
                }
                _ => unreachable!(),
            }
        }
        assert_eq!(TileName::None.tile_type(), TileType::None);
        assert_eq!(TileName::None.category(), TileCategory::None);
    }

    #[test]
    fn test_indicator_to_dora_exhaustive() {
        use mahjong::dora::indicator_to_dora;

        // 萬子 (1m->2m...9m->1m)
        assert_eq!(indicator_to_dora(TileName::OneM), TileName::TwoM);
        assert_eq!(indicator_to_dora(TileName::NineM), TileName::OneM);

        // 筒子 (1p->2p...9p->1p)
        assert_eq!(indicator_to_dora(TileName::OneP), TileName::TwoP);
        assert_eq!(indicator_to_dora(TileName::NineP), TileName::OneP);

        // 索子 (1s->2s...9s->1s)
        assert_eq!(indicator_to_dora(TileName::OneS), TileName::TwoS);
        assert_eq!(indicator_to_dora(TileName::NineS), TileName::OneS);

        // 風牌 (東->南->西->北->東)
        assert_eq!(indicator_to_dora(TileName::East), TileName::South);
        assert_eq!(indicator_to_dora(TileName::South), TileName::West);
        assert_eq!(indicator_to_dora(TileName::West), TileName::North);
        assert_eq!(indicator_to_dora(TileName::North), TileName::East);

        // 三元牌 (白->發->中->白)
        assert_eq!(indicator_to_dora(TileName::White), TileName::Green);
        assert_eq!(indicator_to_dora(TileName::Green), TileName::Red);
        assert_eq!(indicator_to_dora(TileName::Red), TileName::White);

        // None
        assert_eq!(indicator_to_dora(TileName::None), TileName::None);
    }
}
