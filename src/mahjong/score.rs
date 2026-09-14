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

use crate::yaku::{self, HandPattern, MeldKind};

/// 牌がヤオ九牌（一九字牌）かどうかを判定します
pub fn is_yaojiu(tile: TileName) -> bool {
    let idx = tile as usize;
    matches!(
        tile,
        TileName::OneM
            | TileName::NineM
            | TileName::OneP
            | TileName::NineP
            | TileName::OneS
            | TileName::NineS
    ) || idx >= 28
}

/// 門前順子と和了牌から待ち形符（嵌張・辺張なら2符、両面なら0符）を計算します
pub fn get_sequence_wait_fu(seq_start: TileName, win_tile: TileName) -> usize {
    let Some((start_suit, start_rank)) = yaku::is_number_tile(seq_start) else {
        return 0;
    };
    let Some((win_suit, win_rank)) = yaku::is_number_tile(win_tile) else {
        return 0;
    };
    if start_suit != win_suit {
        return 0;
    }
    // 嵌張待ち: start_rank + 1 == win_rank (例: 13 の 2, 24 の 3)
    if win_rank == start_rank + 1 {
        return 2;
    }
    // 辺張待ち: 12 の 3 (start_rank == 1, win_rank == 3)
    if start_rank == 1 && win_rank == 3 {
        return 2;
    }
    // 辺張待ち: 89 の 7 (start_rank == 7, win_rank == 7)
    if start_rank == 7 && win_rank == 7 {
        return 2;
    }
    0
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

/// 分解された手牌パターン（HandPattern）と和了状況から符を計算します。
pub fn calculate_pattern_fu(
    pattern: &HandPattern,
    win_tile: TileName,
    is_tsumo: bool,
    is_pinfu: bool,
    is_closed: bool,
    seat_wind: Option<TileName>,
    round_wind: Option<TileName>,
) -> usize {
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

    // 雀頭符: 三元牌(+2)、自風(+2)、場風(+2)。連風牌なら+4
    let pair_idx = pattern.pair as usize;
    if (32..=34).contains(&pair_idx) {
        fu += 2;
    }
    if Some(pattern.pair) == seat_wind {
        fu += 2;
    }
    if Some(pattern.pair) == round_wind {
        fu += 2;
    }

    // 待ち形符: 単騎・嵌張・辺張なら2符
    let mut wait_fu = 0;
    if pattern.pair == win_tile {
        wait_fu = 2;
    }
    for meld in &pattern.melds {
        if let MeldKind::Sequence(start) = meld {
            let seq_wait = get_sequence_wait_fu(*start, win_tile);
            if seq_wait > wait_fu {
                wait_fu = seq_wait;
            }
        }
    }
    fu += wait_fu;

    // 面子符
    // 1. 副露面子
    for meld in &pattern.open_melds {
        match meld {
            MeldKind::Sequence(_) => {}
            MeldKind::Triplet(t) => {
                fu += if is_yaojiu(*t) { 4 } else { 2 };
            }
            MeldKind::Quad(t) => {
                fu += if is_yaojiu(*t) { 16 } else { 8 };
            }
        }
    }
    // 2. 門前面子
    let mut ron_tile_used_for_triplet = false;
    for meld in &pattern.melds {
        match meld {
            MeldKind::Sequence(_) => {}
            MeldKind::Triplet(t) => {
                // ロン和了でこの刻子を完成させた場合、1つのみ明刻扱い
                if !is_tsumo && *t == win_tile && !ron_tile_used_for_triplet {
                    ron_tile_used_for_triplet = true;
                    fu += if is_yaojiu(*t) { 4 } else { 2 };
                } else {
                    fu += if is_yaojiu(*t) { 8 } else { 4 };
                }
            }
            MeldKind::Quad(t) => {
                fu += if is_yaojiu(*t) { 32 } else { 16 };
            }
        }
    }

    ceil10_fu(fu).max(30)
}

/// 手牌の面子・雀頭・待ち形から符を総合計算します（高点法：最大符を採用）。
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
    let is_closed = crate::hand::is_menzen(open_melds);
    if is_pinfu && is_closed && is_tsumo {
        return 20;
    }

    let patterns = yaku::get_hand_patterns(counts, open_melds);
    if !patterns.is_empty() {
        let mut max_fu = 0;
        for pattern in &patterns {
            let fu = calculate_pattern_fu(
                pattern, win_tile, is_tsumo, is_pinfu, is_closed, seat_wind, round_wind,
            );
            if fu > max_fu {
                max_fu = fu;
            }
        }
        return max_fu.max(30);
    }

    // パターンに分解できない手（テストケース等、14枚揃っていない場合）のフォールバック
    let mut fu = 20;
    if is_closed && !is_tsumo {
        fu += 10;
    }
    if is_tsumo && !is_pinfu {
        fu += 2;
    }
    let mut meld_fu = 0;
    for m in open_melds {
        match m {
            crate::hand::Meld::Chii { .. } => {}
            crate::hand::Meld::Pon(t) => {
                meld_fu += if is_yaojiu(*t) { 4 } else { 2 };
            }
            crate::hand::Meld::Daiminkan(t) | crate::hand::Meld::Kakan(t) => {
                meld_fu += if is_yaojiu(*t) { 16 } else { 8 };
            }
            crate::hand::Meld::Ankan(t) => {
                meld_fu += if is_yaojiu(*t) { 32 } else { 16 };
            }
        }
    }
    let mut pair_tile = None;
    for i in 1..=34 {
        let c = counts[i];
        let t = TileName::from_usize(i);
        if c >= 3 {
            let is_ron_agari_tile = !is_tsumo && t == win_tile;
            let val = if is_ron_agari_tile {
                if is_yaojiu(t) {
                    4
                } else {
                    2
                }
            } else if is_yaojiu(t) {
                8
            } else {
                4
            };
            meld_fu += val;
        } else if c == 2 && pair_tile.is_none() {
            pair_tile = Some(t);
        }
    }
    if let Some(p) = pair_tile {
        let p_idx = p as usize;
        if (32..=34).contains(&p_idx) {
            fu += 2;
        }
        if Some(p) == seat_wind {
            fu += 2;
        }
        if Some(p) == round_wind {
            fu += 2;
        }
        if p == win_tile {
            fu += 2;
        }
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

    #[test]
    fn test_calculate_hand_fu_continuous_sequences() {
        // 123m 234m 345m (3mが3枚あるが順子3組) + 99m (雀頭) + 555s (暗刻 4符)
        // ツモ: 5s 以外（例えば4mツモで平和形等、ここでは 3mツモ）
        let mut counts = [0u8; 35];
        // 123m, 234m, 345m: 1m:1, 2m:2, 3m:3, 4m:2, 5m:1
        counts[TileName::OneM as usize] = 1;
        counts[TileName::TwoM as usize] = 2;
        counts[TileName::ThreeM as usize] = 3;
        counts[TileName::FourM as usize] = 2;
        counts[TileName::FiveM as usize] = 1;
        counts[TileName::NineM as usize] = 2; // 雀頭
        counts[TileName::FiveS as usize] = 3; // 中張牌暗刻: 4符

        // ツモ和了 (3m ツモ)
        // 符: 副底 20 + ツモ 2 + 5s暗刻 4 = 26符 -> 30符（3m暗刻の誤判定があれば +4符で 30+4=34 -> 40符になってしまう）
        let fu = calculate_hand_fu(
            &counts,
            &[],
            TileName::ThreeM,
            true, // is_tsumo
            false,
            false,
            None,
            None,
        );
        assert_eq!(fu, 30, "連続順子の3mは暗刻符にならず30符であるべき");
    }

    #[test]
    fn test_calculate_hand_fu_penchan_and_kanchan() {
        // 1. 辺張待ち: 111m (ヤオ九牌暗刻 8符) + 234p + 456p + 89s (7s辺張待ち) + 55m (雀頭)
        let mut counts_penchan = [0u8; 35];
        counts_penchan[TileName::OneM as usize] = 3; // 1m 暗刻 (8符)
        counts_penchan[TileName::TwoP as usize] = 1;
        counts_penchan[TileName::ThreeP as usize] = 1;
        counts_penchan[TileName::FourP as usize] = 2;
        counts_penchan[TileName::FiveP as usize] = 1;
        counts_penchan[TileName::SixP as usize] = 1;
        counts_penchan[TileName::EightS as usize] = 1;
        counts_penchan[TileName::NineS as usize] = 1;
        counts_penchan[TileName::SevenS as usize] = 1; // 和了牌 7s
        counts_penchan[TileName::FiveM as usize] = 2; // 雀頭

        // 7s ツモ和了: 副底 20 + ツモ 2 + 1m暗刻 8 + 辺張 2 = 32符 -> 40符
        let fu_penchan = calculate_hand_fu(
            &counts_penchan,
            &[],
            TileName::SevenS,
            true,
            false,
            false,
            None,
            None,
        );
        assert_eq!(fu_penchan, 40, "7s辺張ツモは32符切り上げで40符になるべき");

        // 2. 嵌張待ち: 111m (ヤオ九牌暗刻 8符) + 234p + 456p + 24s (3s嵌張待ち) + 55m (雀頭)
        let mut counts_kanchan = [0u8; 35];
        counts_kanchan[TileName::OneM as usize] = 3;
        counts_kanchan[TileName::TwoP as usize] = 1;
        counts_kanchan[TileName::ThreeP as usize] = 1;
        counts_kanchan[TileName::FourP as usize] = 2;
        counts_kanchan[TileName::FiveP as usize] = 1;
        counts_kanchan[TileName::SixP as usize] = 1;
        counts_kanchan[TileName::TwoS as usize] = 1;
        counts_kanchan[TileName::FourS as usize] = 1;
        counts_kanchan[TileName::ThreeS as usize] = 1; // 和了牌 3s
        counts_kanchan[TileName::FiveM as usize] = 2; // 雀頭

        // 3s ツモ和了: 副底 20 + ツモ 2 + 1m暗刻 8 + 嵌張 2 = 32符 -> 40符
        let fu_kanchan = calculate_hand_fu(
            &counts_kanchan,
            &[],
            TileName::ThreeS,
            true,
            false,
            false,
            None,
            None,
        );
        assert_eq!(fu_kanchan, 40, "3s嵌張ツモは32符切り上げで40符になるべき");
    }

    #[test]
    fn test_score_ankan_closed_ron_10_fu() {
        // 暗槓（Meld::Ankan）のみを持つ手牌でのロン和了（出和了）
        // 手牌: 234p 456p 789s 55m (11枚門前牌) + 1m暗槓 (Meld::Ankan(1m))
        // 和了牌: 9s (ロン和了)
        // 門前清であるため:
        // - 副底: 20符
        // - 門前ロン加符: 10符 (暗槓は副露扱いされず門前ロン加符が付くこと！)
        // - 1m 暗槓 (ヤオ九牌): 32符
        // - 待ち 789s (両面): 0符
        // - 雀頭 5m: 0符
        // 計: 20 + 10 + 32 = 62符 -> 切り上げで 70符
        let mut counts = [0u8; 35];
        counts[TileName::TwoP as usize] = 1;
        counts[TileName::ThreeP as usize] = 1;
        counts[TileName::FourP as usize] = 2;
        counts[TileName::FiveP as usize] = 1;
        counts[TileName::SixP as usize] = 1;
        counts[TileName::SevenS as usize] = 1;
        counts[TileName::EightS as usize] = 1;
        counts[TileName::NineS as usize] = 1; // 和了牌 9s (ロン)
        counts[TileName::FiveM as usize] = 2; // 雀頭

        let open_melds = vec![crate::hand::Meld::Ankan(TileName::OneM)];

        let fu = calculate_hand_fu(
            &counts,
            &open_melds,
            TileName::NineS,
            false, // ロン和了
            false,
            false,
            None,
            None,
        );
        assert_eq!(
            fu, 70,
            "暗槓のみの手牌の出和了は門前ロン加符10符が付き70符になるべき"
        );
    }
}
