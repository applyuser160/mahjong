use mahjong::river::River;
use mahjong::tile::TileName;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_river_push() {
        let mut river = River::new();
        assert_eq!(river.tiles().len(), 0);

        river.push(TileName::OneM);
        assert_eq!(river.tiles().len(), 1);
        assert_eq!(river.tiles()[0], TileName::OneM);

        river.push(TileName::TwoP);
        assert_eq!(river.tiles().len(), 2);
        assert_eq!(river.tiles()[1], TileName::TwoP);
    }

    #[test]
    fn test_river_pop() {
        let mut river = River::new();
        assert_eq!(river.pop(), None);

        river.push(TileName::OneM);
        river.push(TileName::TwoP);

        assert_eq!(river.pop(), Some(TileName::TwoP));
        assert_eq!(river.tiles().len(), 1);
        assert_eq!(river.tiles()[0], TileName::OneM);

        assert_eq!(river.pop(), Some(TileName::OneM));
        assert_eq!(river.tiles().len(), 0);

        assert_eq!(river.pop(), None);
    }
}
