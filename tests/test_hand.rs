#[cfg(test)]
mod tests {
    use mahjong::hand::{Hand, Meld};
    use mahjong::tile::TileName::*;

    #[test]
    fn test_call_meld_chii() {
        let mut hand = Hand::new();
        hand.push(TwoM);
        hand.push(ThreeM);
        hand.push(FourM);
        hand.push(FiveM);

        // Valid Chii (2m, 3m, 4m)
        assert!(hand.call_meld(Meld::Chii(TwoM), &[ThreeM, FourM]).is_ok());
        assert_eq!(hand.open_melds.len(), 1);
        assert_eq!(hand.open_melds[0], Meld::Chii(TwoM));
    }

    #[test]
    fn test_call_meld_invalid_chii_cross_suit() {
        let mut hand = Hand::new();
        hand.push(TwoM);
        hand.push(ThreeP);
        hand.push(FourM);

        // Invalid Chii crossing suits (2m, 3p, 4m)
        let result = hand.call_meld(Meld::Chii(TwoM), &[ThreeP, FourM]);
        assert!(result.is_err());
        assert_eq!(hand.open_melds.len(), 0);
    }

    #[test]
    fn test_call_meld_invalid_chii_not_sequential() {
        let mut hand = Hand::new();
        hand.push(TwoM);
        hand.push(ThreeM);
        hand.push(FiveM);

        // Invalid Chii not sequential (2m, 3m, 5m)
        let result = hand.call_meld(Meld::Chii(TwoM), &[ThreeM, FiveM]);
        assert!(result.is_err());
        assert_eq!(hand.open_melds.len(), 0);
    }

    #[test]
    fn test_call_meld_missing_tiles() {
        let mut hand = Hand::new();
        hand.push(TwoM);

        let result = hand.call_meld(Meld::Chii(TwoM), &[ThreeM, FourM]);
        assert!(result.is_err());
    }

    #[test]
    fn test_call_meld_pon() {
        let mut hand = Hand::new();
        hand.push(Red);
        hand.push(Red);
        hand.push(Red);

        assert!(hand.call_meld(Meld::Pon(Red), &[Red, Red]).is_ok());
        assert_eq!(hand.open_melds.len(), 1);
        assert_eq!(hand.open_melds[0], Meld::Pon(Red));
    }

    #[test]
    fn test_call_meld_pon_missing_duplicates() {
        let mut hand = Hand::new();
        hand.push(Red); // Only 1 Red in hand, but trying to consume 2

        let result = hand.call_meld(Meld::Pon(Red), &[Red, Red]);
        assert!(result.is_err());
    }
}
