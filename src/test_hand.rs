#[cfg(test)]

mod tests {
    use crate::{
        hand::Hand,
        tile::{Tile, TileName},
    };

    #[test]
    fn get_chows_case01() {
        let mut hand = Hand::new();
        hand.push_tile(Tile::from_name(TileName::OneM, false));
        hand.push_tile(Tile::from_name(TileName::East, false));
        hand.push_tile(Tile::from_name(TileName::NineP, false));
        hand.push_tile(Tile::from_name(TileName::East, false));
        hand.push_tile(Tile::from_name(TileName::NineP, false));
        hand.push_tile(Tile::from_name(TileName::OneM, false));
        hand.push_tile(Tile::from_name(TileName::White, false));
        hand.push_tile(Tile::from_name(TileName::TwoM, false));
        hand.push_tile(Tile::from_name(TileName::EightP, false));
        hand.push_tile(Tile::from_name(TileName::NineP, false));
        hand.push_tile(Tile::from_name(TileName::OneM, false));
        hand.push_tile(Tile::from_name(TileName::SevenP, false));
        hand.push_tile(Tile::from_name(TileName::ThreeM, false));
        hand.push_tile(Tile::from_name(TileName::OneM, false));

        hand.sort();

        /* sorted tiles */
        /* 1m,1m,1m,1m,2m,3m,7p,8p,9p,9p,9p,東,東,白 */
        /*  0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13 */

        let chows = hand.get_chows();
        assert_eq!(chows, vec![3, 6]);

        let pungs = hand.get_pungs();
        assert_eq!(pungs, vec![0, 1, 8]);

        let kongs = hand.get_kongs();
        assert_eq!(kongs, vec![0]);

        let pairs = hand.get_pairs();
        assert_eq!(pairs, vec![0, 1, 2, 8, 9, 11]);
    }
}
