use crate::tile::TileName;

/// プレイヤーが親か子か
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSeat {
    East,  // 親
    Other, // 子
}

/// 和了の得点計算結果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreResult {
    pub han: usize,
    pub fu: usize,
    pub total_points: usize,
    pub points_description: &'static str,
    // ツモ時の各プレイヤーの支払い (親の場合: 全員均等払い, 子の場合: (子払い, 親払い))
    pub tsumo_payments: Option<(usize, usize)>,
}

/// 100点単位で切り上げます (例: 1120 -> 1200, 1000 -> 1000)
pub fn ceil100(val: usize) -> usize {
    val.div_ceil(100) * 100
}

/// 10符単位で切り上げます (例: 22 -> 30, 30 -> 30)
pub fn ceil10_fu(fu: usize) -> usize {
    fu.div_ceil(10) * 10
}

/// 翻数と符から得点を算出します。
/// `han`: 役の翻数 + ドラの翻数
/// `fu`: 符数 (七対子は25, 平和ツモは20, その他は30..110)
/// `is_dealer`: 親（東家）かどうか
/// `is_tsumo`: ツモ和了かどうか
/// `is_yakuman`: 役満かどうか
pub fn calculate_score(
    han: usize,
    fu: usize,
    is_dealer: bool,
    is_tsumo: bool,
    is_yakuman: bool,
) -> ScoreResult {
    // 役満の計算
    if is_yakuman || han >= 13 {
        let (total_points, desc) = if is_yakuman {
            if is_dealer {
                (48000, "役満 (親: 48000点)")
            } else {
                (32000, "役満 (子: 32000点)")
            }
        } else if is_dealer {
            (48000, "数え役満 (親: 48000点)")
        } else {
            (32000, "数え役満 (子: 32000点)")
        };

        let tsumo_payments = if is_tsumo {
            if is_dealer {
                Some((16000, 16000)) // 16000点オール
            } else {
                Some((8000, 16000)) // 8000/16000
            }
        } else {
            None
        };

        return ScoreResult {
            han,
            fu,
            total_points,
            points_description: desc,
            tsumo_payments,
        };
    }

    // 翻数による区分判定
    if han >= 11 {
        // 三倍満
        let total_points = if is_dealer { 36000 } else { 24000 };
        let desc = "三倍満";
        let tsumo_payments = if is_tsumo {
            if is_dealer {
                Some((12000, 12000))
            } else {
                Some((6000, 12000))
            }
        } else {
            None
        };
        return ScoreResult {
            han,
            fu,
            total_points,
            points_description: desc,
            tsumo_payments,
        };
    }

    if han >= 8 {
        // 倍満
        let total_points = if is_dealer { 24000 } else { 16000 };
        let desc = "倍満";
        let tsumo_payments = if is_tsumo {
            if is_dealer {
                Some((8000, 8000))
            } else {
                Some((4000, 8000))
            }
        } else {
            None
        };
        return ScoreResult {
            han,
            fu,
            total_points,
            points_description: desc,
            tsumo_payments,
        };
    }

    if han >= 6 {
        // 跳満
        let total_points = if is_dealer { 18000 } else { 12000 };
        let desc = "跳満";
        let tsumo_payments = if is_tsumo {
            if is_dealer {
                Some((6000, 6000))
            } else {
                Some((3000, 6000))
            }
        } else {
            None
        };
        return ScoreResult {
            han,
            fu,
            total_points,
            points_description: desc,
            tsumo_payments,
        };
    }

    // 5翻、または基本点が満貫に達する場合
    // 基本点: A = fu * 2^(2 + han)
    let basic_points = fu * (1 << (2 + han));
    if han == 5 || basic_points >= 2000 {
        // 満貫
        let total_points = if is_dealer { 12000 } else { 8000 };
        let desc = "満貫";
        let tsumo_payments = if is_tsumo {
            if is_dealer {
                Some((4000, 4000))
            } else {
                Some((2000, 4000))
            }
        } else {
            None
        };
        return ScoreResult {
            han,
            fu,
            total_points,
            points_description: desc,
            tsumo_payments,
        };
    }

    // 満貫未満（通常計算）
    let desc = "通常手";
    if is_dealer {
        if is_tsumo {
            let payment_each = ceil100(basic_points * 2);
            let total_points = payment_each * 3;
            ScoreResult {
                han,
                fu,
                total_points,
                points_description: desc,
                tsumo_payments: Some((payment_each, payment_each)),
            }
        } else {
            let total_points = ceil100(basic_points * 6);
            ScoreResult {
                han,
                fu,
                total_points,
                points_description: desc,
                tsumo_payments: None,
            }
        }
    } else if is_tsumo {
        let child_pay = ceil100(basic_points);
        let dealer_pay = ceil100(basic_points * 2);
        let total_points = child_pay * 2 + dealer_pay;
        ScoreResult {
            han,
            fu,
            total_points,
            points_description: desc,
            tsumo_payments: Some((child_pay, dealer_pay)),
        }
    } else {
        let total_points = ceil100(basic_points * 4);
        ScoreResult {
            han,
            fu,
            total_points,
            points_description: desc,
            tsumo_payments: None,
        }
    }
}

/// 和了形の手牌と状況から符（fu）を計算します。
/// `is_pinfu`: 平和が成立しているか
/// `is_chitoitsu`: 七対子か
/// `is_closed`: 門前か
/// `is_tsumo`: ツモ和了か
/// `pair`: 雀頭の牌
/// `seat_wind`: 自風
/// `round_wind`: 場風
/// `wait_is_isolated`: 待ち形が単騎・嵌張・辺張か（両面・双ポンでなければtrue）
#[allow(clippy::too_many_arguments)]
pub fn calculate_fu(
    is_pinfu: bool,
    is_chitoitsu: bool,
    is_closed: bool,
    is_tsumo: bool,
    pair: Option<TileName>,
    seat_wind: Option<TileName>,
    round_wind: Option<TileName>,
    wait_is_isolated: bool,
    meld_fu_sum: usize,
) -> usize {
    // 七対子は25符固定
    if is_chitoitsu {
        return 25;
    }

    // 門前ピンフツモは20符固定
    if is_pinfu && is_closed && is_tsumo {
        return 20;
    }

    // 副底: 20符
    let mut fu = 20;

    // 門前ロン加符: 10符
    if is_closed && !is_tsumo {
        fu += 10;
    }

    // ツモ符: 2符 (ピンフツモ以外)
    if is_tsumo && !is_pinfu {
        fu += 2;
    }

    // 待ち形符: 単騎・嵌張・辺張なら2符
    if wait_is_isolated {
        fu += 2;
    }

    // 雀頭符: 三元牌、自風、場風なら各2符
    if let Some(p) = pair {
        if matches!(p, TileName::White | TileName::Green | TileName::Red) {
            fu += 2;
        }
        if Some(p) == seat_wind {
            fu += 2;
        }
        if Some(p) == round_wind {
            fu += 2;
        }
    }

    // 面子符の加算
    fu += meld_fu_sum;

    // 鳴き平和（食い平和形のロンなど、符加算がなく20符のままの場合）は30符に切り上げ
    ceil10_fu(fu).max(30)
}

/// 手牌の面子・雀頭・待ち形から符を総合計算します。
#[allow(clippy::too_many_arguments, clippy::needless_range_loop)]
pub fn calculate_hand_fu(
    counts: &[u8; 35],
    open_melds: &[crate::hand::Meld],
    win_tile: TileName,
    is_tsumo: bool,
    is_pinfu: bool,
    is_chitoitsu: bool,
    seat_wind: Option<TileName>,
    round_wind: Option<TileName>,
) -> usize {
    if is_chitoitsu {
        return 25;
    }
    let is_closed = open_melds.is_empty();
    if is_pinfu && is_closed && is_tsumo {
        return 20;
    }

    // 副底: 20符
    let mut fu = 20;

    // 門前ロン加符: 10符
    if is_closed && !is_tsumo {
        fu += 10;
    }

    // ツモ符: 2符 (ピンフツモ以外)
    if is_tsumo && !is_pinfu {
        fu += 2;
    }

    // 副露からの面子符
    let mut meld_fu = 0;
    for m in open_melds {
        match m {
            crate::hand::Meld::Chii { .. } => {}
            crate::hand::Meld::Pon(t) => {
                let idx = *t as usize;
                let is_yaojiu = matches!(
                    t,
                    TileName::OneM
                        | TileName::NineM
                        | TileName::OneP
                        | TileName::NineP
                        | TileName::OneS
                        | TileName::NineS
                ) || idx >= 28;
                meld_fu += if is_yaojiu { 4 } else { 2 };
            }
            crate::hand::Meld::Daiminkan(t) | crate::hand::Meld::Kakan(t) => {
                let idx = *t as usize;
                let is_yaojiu = matches!(
                    t,
                    TileName::OneM
                        | TileName::NineM
                        | TileName::OneP
                        | TileName::NineP
                        | TileName::OneS
                        | TileName::NineS
                ) || idx >= 28;
                meld_fu += if is_yaojiu { 16 } else { 8 };
            }
            crate::hand::Meld::Ankan(t) => {
                let idx = *t as usize;
                let is_yaojiu = matches!(
                    t,
                    TileName::OneM
                        | TileName::NineM
                        | TileName::OneP
                        | TileName::NineP
                        | TileName::OneS
                        | TileName::NineS
                ) || idx >= 28;
                meld_fu += if is_yaojiu { 32 } else { 16 };
            }
        }
    }

    // 門前手牌中の刻子 (counts >= 3)
    let mut pair_tile = None;
    for i in 1..=34 {
        let c = counts[i];
        let t = TileName::from_usize(i);
        let is_yaojiu = matches!(
            t,
            TileName::OneM
                | TileName::NineM
                | TileName::OneP
                | TileName::NineP
                | TileName::OneS
                | TileName::NineS
        ) || i >= 28;
        if c >= 3 {
            let is_ron_agari_tile = !is_tsumo && t == win_tile;
            let val = if is_ron_agari_tile {
                if is_yaojiu {
                    4
                } else {
                    2
                }
            } else if is_yaojiu {
                8
            } else {
                4
            };
            meld_fu += val;
        } else if c == 2 && pair_tile.is_none() {
            pair_tile = Some(t);
        }
    }

    // 雀頭符
    if let Some(p) = pair_tile {
        let p_idx = p as usize;
        // 三元牌
        if (32..=34).contains(&p_idx) {
            fu += 2;
        }
        // 自風
        if Some(p) == seat_wind {
            fu += 2;
        }
        // 場風
        if Some(p) == round_wind {
            fu += 2;
        }
    }

    // 待ち形符（単騎待ちなど、和了牌が雀頭と一致する場合は単騎待ちで2符）
    if pair_tile == Some(win_tile) {
        fu += 2;
    }

    fu += meld_fu;

    ceil10_fu(fu).max(30)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mangan_ron_child() {
        // 子の満貫ロン -> 8000点
        let res = calculate_score(5, 30, false, false, false);
        assert_eq!(res.total_points, 8000);
        assert_eq!(res.points_description, "満貫");
    }

    #[test]
    fn test_mangan_tsumo_child() {
        // 子の満貫ツモ -> 8000点 (2000, 4000)
        let res = calculate_score(5, 30, false, true, false);
        assert_eq!(res.total_points, 8000);
        assert_eq!(res.tsumo_payments, Some((2000, 4000)));
    }

    #[test]
    fn test_mangan_ron_dealer() {
        // 親の満貫ロン -> 12000点
        let res = calculate_score(5, 30, true, false, false);
        assert_eq!(res.total_points, 12000);
    }

    #[test]
    fn test_haneman_child() {
        // 子の跳満 -> 12000点 (3000, 6000)
        let res = calculate_score(6, 30, false, true, false);
        assert_eq!(res.total_points, 12000);
        assert_eq!(res.tsumo_payments, Some((3000, 6000)));
    }

    #[test]
    fn test_normal_score_child_ron() {
        // 子の1翻30符ロン -> 基本点 30 * 2^3 = 240 -> 240 * 4 = 960 -> 1000点
        let res = calculate_score(1, 30, false, false, false);
        assert_eq!(res.total_points, 1000);
    }

    #[test]
    fn test_normal_score_pinfu_tsumo_child() {
        // 子の門前平和ツモ (2翻20符)
        // 基本点 20 * 2^4 = 320 -> 子払い 400点, 親払い 700点 -> 計 1500点
        let res = calculate_score(2, 20, false, true, false);
        assert_eq!(res.total_points, 1500);
        assert_eq!(res.tsumo_payments, Some((400, 700)));
    }

    #[test]
    fn test_chitoitsu_score() {
        // 七対子 (2翻25符) ロン
        // 基本点 25 * 2^4 = 400 -> 子ロン: 400 * 4 = 1600点
        let res = calculate_score(2, 25, false, false, false);
        assert_eq!(res.total_points, 1600);
    }

    #[test]
    fn test_calculate_hand_fu_pinfu_and_chitoitsu_and_anko() {
        let empty_melds = vec![];
        let counts = [0u8; 35];

        // 1. 七対子は25符
        let fu_chitoi = calculate_hand_fu(
            &counts,
            &empty_melds,
            TileName::East,
            false,
            false,
            true, // is_chitoitsu
            None,
            None,
        );
        assert_eq!(fu_chitoi, 25);

        // 2. 門前ピンフツモは20符
        let fu_pinfu_tsumo = calculate_hand_fu(
            &counts,
            &empty_melds,
            TileName::TwoM,
            true, // is_tsumo
            true, // is_pinfu
            false,
            None,
            None,
        );
        assert_eq!(fu_pinfu_tsumo, 20);

        // 3. 役牌暗刻（白暗刻 8符）+ 門前ロン（10符）+ 副底（20符） = 38符 -> 40符
        let mut counts_anko = [0u8; 35];
        counts_anko[TileName::White as usize] = 3;
        counts_anko[TileName::TwoM as usize] = 2; // 雀頭
        let fu_anko = calculate_hand_fu(
            &counts_anko,
            &empty_melds,
            TileName::FiveS,
            false, // is_tsumo
            false,
            false,
            None,
            None,
        );
        assert_eq!(fu_anko, 40);
    }
}
