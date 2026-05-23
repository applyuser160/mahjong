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
        assert!(hand
            .call_meld(Meld::Chii {
                called: TwoM,
                consumed: [ThreeM, FourM]
            })
            .is_ok());
        assert_eq!(hand.open_melds.len(), 1);
        assert_eq!(
            hand.open_melds[0],
            Meld::Chii {
                called: TwoM,
                consumed: [ThreeM, FourM]
            }
        );
    }

    #[test]
    fn test_call_meld_invalid_chii_cross_suit() {
        let mut hand = Hand::new();
        hand.push(TwoM);
        hand.push(ThreeP);
        hand.push(FourM);

        // Invalid Chii crossing suits (2m, 3p, 4m)
        let result = hand.call_meld(Meld::Chii {
            called: TwoM,
            consumed: [ThreeP, FourM],
        });
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
        let result = hand.call_meld(Meld::Chii {
            called: TwoM,
            consumed: [ThreeM, FiveM],
        });
        assert!(result.is_err());
        assert_eq!(hand.open_melds.len(), 0);
    }

    #[test]
    fn test_call_meld_missing_tiles() {
        let mut hand = Hand::new();
        hand.push(TwoM);

        let result = hand.call_meld(Meld::Chii {
            called: TwoM,
            consumed: [ThreeM, FourM],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_call_meld_pon() {
        let mut hand = Hand::new();
        hand.push(Red);
        hand.push(Red);
        hand.push(Red);

        assert!(hand.call_meld(Meld::Pon(Red)).is_ok());
        assert_eq!(hand.open_melds.len(), 1);
        assert_eq!(hand.open_melds[0], Meld::Pon(Red));
    }

    #[test]
    fn test_call_meld_pon_missing_duplicates() {
        let mut hand = Hand::new();
        hand.push(Red); // Only 1 Red in hand, but trying to consume 2

        let result = hand.call_meld(Meld::Pon(Red));
        assert!(result.is_err());
    }

    #[test]
    fn test_call_meld_daiminkan() {
        let mut hand = Hand::new();
        hand.push(White);
        hand.push(White);
        hand.push(White);

        assert!(hand.call_meld(Meld::Daiminkan(White)).is_ok());
        assert_eq!(hand.open_melds.len(), 1);
        assert_eq!(hand.open_melds[0], Meld::Daiminkan(White));
    }

    #[test]
    fn test_call_meld_ankan() {
        let mut hand = Hand::new();
        hand.push(Green);
        hand.push(Green);
        hand.push(Green);
        hand.push(Green);

        assert!(hand.call_meld(Meld::Ankan(Green)).is_ok());
        assert_eq!(hand.open_melds.len(), 1);
        assert_eq!(hand.open_melds[0], Meld::Ankan(Green));
    }

    #[test]
    fn test_call_meld_kakan() {
        let mut hand = Hand::new();
        hand.push(East);
        hand.push(East);

        // First make a Pon
        assert!(hand.call_meld(Meld::Pon(East)).is_ok());
        assert_eq!(hand.open_melds.len(), 1);
        assert_eq!(hand.open_melds[0], Meld::Pon(East));

        // Now push the fourth tile and call Kakan
        hand.push(East);
        assert!(hand.call_meld(Meld::Kakan(East)).is_ok());
        assert_eq!(hand.open_melds.len(), 1); // length should still be 1
        assert_eq!(hand.open_melds[0], Meld::Kakan(East)); // should have replaced Pon
    }

    #[test]
    fn test_call_meld_kakan_without_pon_fails() {
        let mut hand = Hand::new();
        hand.push(East);

        // Try to call Kakan without a preceding Pon
        let result = hand.call_meld(Meld::Kakan(East));
        assert!(result.is_err());
    }
}
