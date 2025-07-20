#[cfg(test)]

mod tests {
    use crate::{
        hand::{Hand, Honor},
        tile::{Tile, TileName},
    };

    #[test]
    fn get_chows_case01() {
        let mut tiles: Vec<Tile> = Vec::new();
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::NineP, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::NineP, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::White, false));
        tiles.push(Tile::from_name(TileName::TwoM, false));
        tiles.push(Tile::from_name(TileName::EightP, false));
        tiles.push(Tile::from_name(TileName::NineP, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::SevenP, false));
        tiles.push(Tile::from_name(TileName::ThreeM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));

        tiles.sort();

        /* sorted tiles */
        /* 1m,1m,1m,1m,2m,3m,7p,8p,9p,9p,9p,��,東,白 */
        /*  0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13 */

        let chows = Hand::get_chows(&tiles);
        assert_eq!(chows, vec![3, 6]);

        let pungs = Hand::get_pungs(&tiles);
        assert_eq!(pungs, vec![0, 1, 8]);

        let kongs = Hand::get_kongs(&tiles);
        assert_eq!(kongs, vec![0]);

        let pairs = Hand::get_pairs(&tiles);
        assert_eq!(pairs, vec![0, 1, 2, 8, 9, 11]);
    }

    #[test]
    fn get_standard_case01() {
        let mut tiles: Vec<Tile> = Vec::new();
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::Green, false));
        tiles.push(Tile::from_name(TileName::Green, false));

        Hand::sort(&mut tiles);

        /* sorted tiles */
        /* 1m,1m,1m,1p,1p,1p,1s,1s,1s,東,東,東,緑, 緑 */
        /*  0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13 */

        let standard_indexes = Hand::get_standard(&mut tiles);

        assert_eq!(standard_indexes[0], vec![0, 3, 6, 9, 12]);
    }

    #[test]
    fn find_honors_case01() {
        let mut tiles: Vec<Tile> = Vec::new();
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::Green, false));
        tiles.push(Tile::from_name(TileName::Green, false));

        Hand::sort(&mut tiles);

        /* sorted tiles */
        /* 1m,1m,1m,1p,1p,1p,1s,1s,1s,東,東,東,緑, 緑 */
        /*  0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13 */

        let honor = Hand::find_honors(&mut tiles, false, true, true, true);

        let honor_assert = Honor::from(Vec::new(), vec![0, 3, 6, 9], Vec::new(), vec![12]);
        assert_eq!(honor[0], honor_assert);
    }

    #[test]
    fn find_honors_case02() {
        let mut tiles: Vec<Tile> = Vec::new();
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneM, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::OneP, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::OneS, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::East, false));
        tiles.push(Tile::from_name(TileName::Green, false));
        tiles.push(Tile::from_name(TileName::Green, false));

        Hand::sort(&mut tiles);

        /* sorted tiles */
        /* 1m,1m,1m,1m,1p,1p,1p,1s,1s,東,東,東,緑, 緑 */
        /*  0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13 */

        let honor = Hand::find_honors(&mut tiles, true, true, true, true);

        assert_eq!(
            honor[0],
            Honor::from(Vec::new(), vec![4, 9], vec![0], vec![7, 12],),
        );
        assert_eq!(
            honor[1],
            Honor::from(Vec::new(), vec![4, 9], Vec::new(), vec![0, 2, 7, 12],),
        );
    }

    #[test]
    fn is_all_simple_case01() {
        let mut tiles: Vec<Tile> = Vec::new();
        tiles.push(Tile::from_name(TileName::TwoM, false));
        tiles.push(Tile::from_name(TileName::TwoM, false));
        tiles.push(Tile::from_name(TileName::TwoM, false));
        tiles.push(Tile::from_name(TileName::ThreeM, false));
        tiles.push(Tile::from_name(TileName::FourM, false));
        tiles.push(Tile::from_name(TileName::FiveM, false));
        tiles.push(Tile::from_name(TileName::TwoS, false));
        tiles.push(Tile::from_name(TileName::ThreeS, false));
        tiles.push(Tile::from_name(TileName::FourS, false));
        tiles.push(Tile::from_name(TileName::ThreeP, false));
        tiles.push(Tile::from_name(TileName::ThreeP, false));
        tiles.push(Tile::from_name(TileName::ThreeP, false));
        tiles.push(Tile::from_name(TileName::FiveP, false));

        Hand::sort(&mut tiles);

        let draw = Tile::from_name(TileName::FiveP, false);

        let hand = Hand::from(
            tiles,
            Some(draw),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        /* sorted tiles */
        /* 2m,2m,2m,3m,4m,5m,6m,2s,3s,4s,3p,3p,3p,5p */
        /*  0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13 */

        let result = hand.is_all_simples();

        assert!(result);
    }

    #[test]
    fn test_is_winning() {
        let mut hand = Hand::new();
        hand.tiles = vec![
            Tile::from_name(TileName::OneM, false),
            Tile::from_name(TileName::OneM, false),
            Tile::from_name(TileName::OneM, false),
            Tile::from_name(TileName::TwoM, false),
            Tile::from_name(TileName::ThreeM, false),
            Tile::from_name(TileName::FourM, false),
            Tile::from_name(TileName::FiveM, false),
            Tile::from_name(TileName::SixM, false),
            Tile::from_name(TileName::SevenM, false),
            Tile::from_name(TileName::SevenM, false),
            Tile::from_name(TileName::SevenM, false),
            Tile::from_name(TileName::East, false),
            Tile::from_name(TileName::East, false),
        ];
        hand.draw = Some(Tile::from_name(TileName::East, false));
        assert!(hand.is_winning());

        let mut hand = Hand::new();
        hand.tiles = vec![
            Tile::from_name(TileName::OneP, false),
            Tile::from_name(TileName::OneP, false),
        ];
        hand.draw = Some(Tile::from_name(TileName::OneP, false));
        assert!(!hand.is_winning());
    }
}
