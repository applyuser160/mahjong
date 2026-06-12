use std::collections::{HashMap, HashSet};

use crate::tile::TileName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YakuId {
    Riichi,
    MenzenTsumo,
    Tanyao,
    Pinfu,
    Ipeiko,
    YakuhaiHaku,
    YakuhaiHatsu,
    YakuhaiChun,
    YakuhaiJikaze,
    YakuhaiBakaze,
    Chitoitsu,
    Toitoi,
    Sanankou,
    Shousangen,
    Chantaiyao,
    Ryanpeiko,
    SanshokuDoujun,
    SanshokuDoukou,
    Honitsu,
    Junchan,
    Chinitsu,
    Chinroutou,
    Honroutou,
    Sankantsu,
    KokushiMusou,
    Suuankou,
    Daisangen,
    Shousuushi,
    Daisuushi,
    Suukantsu,
    Tsuuiisou,
    Ryuuiisou,
    ChuurenPoutou,
    Tenhou,
    Chiihou,
    RinshanKaihou,
    Chankan,
    HaiteiRaoyue,
    HouteiRaoyui,
    DoubleRiichi,
    Ippatsu,
}

#[derive(Debug, Clone, Copy)]
/// 役の情報を表す構造体です。
pub struct Yaku {
    pub id: YakuId,
    pub name_ja: &'static str,
    pub name_kana: &'static str,
    pub han_closed: i8,
    pub han_open: i8,
    pub yakuman: bool,
}

pub const ALL_YAKU: &[Yaku] = &[
    Yaku {
        id: YakuId::Riichi,
        name_ja: "立直",
        name_kana: "リーチ",
        han_closed: 1,
        han_open: 0,
        yakuman: false,
    },
    Yaku {
        id: YakuId::MenzenTsumo,
        name_ja: "門前清自摸和",
        name_kana: "メンゼンツモ",
        han_closed: 1,
        han_open: 0,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Tanyao,
        name_ja: "断么九",
        name_kana: "タンヤオ",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Pinfu,
        name_ja: "平和",
        name_kana: "ピンフ",
        han_closed: 1,
        han_open: 0,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Ipeiko,
        name_ja: "一盃口",
        name_kana: "イーペーコー",
        han_closed: 1,
        han_open: 0,
        yakuman: false,
    },
    Yaku {
        id: YakuId::YakuhaiHaku,
        name_ja: "役牌 白",
        name_kana: "ハク",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::YakuhaiHatsu,
        name_ja: "役牌 發",
        name_kana: "ハツ",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::YakuhaiChun,
        name_ja: "役牌 中",
        name_kana: "チュン",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::YakuhaiJikaze,
        name_ja: "役牌 自風牌",
        name_kana: "ジカゼ",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::YakuhaiBakaze,
        name_ja: "役牌 場風牌",
        name_kana: "バカゼ",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Chitoitsu,
        name_ja: "七対子",
        name_kana: "チートイツ",
        han_closed: 2,
        han_open: 0,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Toitoi,
        name_ja: "対々和",
        name_kana: "トイトイ",
        han_closed: 2,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Sanankou,
        name_ja: "三暗刻",
        name_kana: "サンアンコウ",
        han_closed: 2,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Shousangen,
        name_ja: "小三元",
        name_kana: "ショウサンゲン",
        han_closed: 2,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Chantaiyao,
        name_ja: "混全帯么九",
        name_kana: "チャンタ",
        han_closed: 2,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Ryanpeiko,
        name_ja: "二盃口",
        name_kana: "リャンペーコー",
        han_closed: 3,
        han_open: 0,
        yakuman: false,
    },
    Yaku {
        id: YakuId::SanshokuDoujun,
        name_ja: "三色同順",
        name_kana: "サンショクドウジュン",
        han_closed: 2,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::SanshokuDoukou,
        name_ja: "三色同刻",
        name_kana: "サンショクドウコウ",
        han_closed: 2,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Honitsu,
        name_ja: "混一色",
        name_kana: "ホンイツ",
        han_closed: 3,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Junchan,
        name_ja: "純全帯么九",
        name_kana: "ジュンチャン",
        han_closed: 3,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Chinitsu,
        name_ja: "清一色",
        name_kana: "チンイツ",
        han_closed: 6,
        han_open: 5,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Chinroutou,
        name_ja: "清老頭",
        name_kana: "チンロウトウ",
        han_closed: 0,
        han_open: 0,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Honroutou,
        name_ja: "混老頭",
        name_kana: "ホンロウトウ",
        han_closed: 2,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Sankantsu,
        name_ja: "三槓子",
        name_kana: "サンカンツ",
        han_closed: 2,
        han_open: 2,
        yakuman: false,
    },
    Yaku {
        id: YakuId::KokushiMusou,
        name_ja: "国士無双",
        name_kana: "コクシムソウ",
        han_closed: 13,
        han_open: 0,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Suuankou,
        name_ja: "四暗刻",
        name_kana: "スーアンコウ",
        han_closed: 13,
        han_open: 0,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Daisangen,
        name_ja: "大三元",
        name_kana: "ダイサンゲン",
        han_closed: 13,
        han_open: 13,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Shousuushi,
        name_ja: "小四喜",
        name_kana: "ショウスーシー",
        han_closed: 13,
        han_open: 13,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Daisuushi,
        name_ja: "大四喜",
        name_kana: "ダイスーシー",
        han_closed: 13,
        han_open: 13,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Suukantsu,
        name_ja: "四槓子",
        name_kana: "スーカンツ",
        han_closed: 13,
        han_open: 13,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Tsuuiisou,
        name_ja: "字一色",
        name_kana: "ツーイーソー",
        han_closed: 13,
        han_open: 13,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Ryuuiisou,
        name_ja: "緑一色",
        name_kana: "リューイーソー",
        han_closed: 13,
        han_open: 13,
        yakuman: true,
    },
    Yaku {
        id: YakuId::ChuurenPoutou,
        name_ja: "九蓮宝燈",
        name_kana: "チューレンポウトウ",
        han_closed: 13,
        han_open: 0,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Tenhou,
        name_ja: "天和",
        name_kana: "テンホウ",
        han_closed: 13,
        han_open: 0,
        yakuman: true,
    },
    Yaku {
        id: YakuId::Chiihou,
        name_ja: "地和",
        name_kana: "チーホウ",
        han_closed: 13,
        han_open: 0,
        yakuman: true,
    },
    Yaku {
        id: YakuId::RinshanKaihou,
        name_ja: "嶺上開花",
        name_kana: "リンシャンカイホウ",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Chankan,
        name_ja: "槍槓",
        name_kana: "チャンカン",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::HaiteiRaoyue,
        name_ja: "海底撈月",
        name_kana: "ハイテイラオユエ",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::HouteiRaoyui,
        name_ja: "河底撈魚",
        name_kana: "ホウテイラオユイ",
        han_closed: 1,
        han_open: 1,
        yakuman: false,
    },
    Yaku {
        id: YakuId::DoubleRiichi,
        name_ja: "ダブル立直",
        name_kana: "ダブルリーチ",
        han_closed: 2,
        han_open: 0,
        yakuman: false,
    },
    Yaku {
        id: YakuId::Ippatsu,
        name_ja: "一発",
        name_kana: "イッパツ",
        han_closed: 1,
        han_open: 0,
        yakuman: false,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 鳴き（副露）の種類を表す列挙型です。
pub enum MeldKind {
    Sequence(TileName),
    Triplet(TileName),
    Quad(TileName),
}

#[derive(Debug, Clone)]
/// 手牌のパターン（雀頭と面子の組み合わせ）を表す構造体です。
struct HandPattern {
    pair: TileName,
    melds: Vec<MeldKind>,
    open_melds: Vec<MeldKind>,
}

impl HandPattern {
    fn all_melds(&self) -> impl Iterator<Item = &MeldKind> {
        self.melds.iter().chain(self.open_melds.iter())
    }
}

#[derive(Debug, Clone, Copy)]
/// 和了（アガリ）時の状況を表す文脈情報です。
pub struct WinContext {
    pub is_closed: bool,
    pub is_tsumo: bool,
    pub seat_wind: Option<TileName>,
    pub round_wind: Option<TileName>,
    pub riichi: bool,
    pub kan_count: usize,
    pub tenhou: bool,
    pub chiihou: bool,
    pub win_tile: Option<TileName>,
    pub is_rinshan: bool,
    pub is_chankan: bool,
    pub is_haitei: bool,
    pub is_houtei: bool,
    pub is_double_riichi: bool,
    pub is_ippatsu: bool,
}

impl Default for WinContext {
    fn default() -> Self {
        Self {
            is_closed: true,
            is_tsumo: false,
            seat_wind: None,
            round_wind: None,
            riichi: false,
            kan_count: 0,
            tenhou: false,
            chiihou: false,
            win_tile: None,
            is_rinshan: false,
            is_chankan: false,
            is_haitei: false,
            is_houtei: false,
            is_double_riichi: false,
            is_ippatsu: false,
        }
    }
}

/// 手牌と副露、および和了（アガリ）時の状況から、成立している役を判定します。
/// 役満が成立している場合は、通常の役は除外されます。
pub fn judge_yaku(
    tiles: &[TileName],
    open_melds_input: &[crate::hand::Meld],
    mut ctx: WinContext,
) -> HashSet<YakuId> {
    let mut result = HashSet::new();

    if tiles.is_empty() {
        return result;
    }

    let mut has_open_meld = false;
    for meld in open_melds_input {
        match meld {
            crate::hand::Meld::Chii { .. }
            | crate::hand::Meld::Pon(_)
            | crate::hand::Meld::Daiminkan(_)
            | crate::hand::Meld::Kakan(_) => {
                has_open_meld = true;
            }
            crate::hand::Meld::Ankan(_) => {}
        }
    }

    if has_open_meld {
        ctx.is_closed = false;
    }

    let mut counts = [0usize; 35];
    for tile in tiles.iter().copied() {
        let idx = tile as usize;
        if idx < counts.len() {
            counts[idx] += 1;
        }
    }

    for meld in open_melds_input {
        match meld {
            crate::hand::Meld::Chii { called, consumed } => {
                for tile in [*called, consumed[0], consumed[1]] {
                    let idx = tile as usize;
                    if idx < counts.len() {
                        counts[idx] += 1;
                    }
                }
            }
            crate::hand::Meld::Pon(t) => {
                for tile in [*t, *t, *t] {
                    let idx = tile as usize;
                    if idx < counts.len() {
                        counts[idx] += 1;
                    }
                }
            }
            crate::hand::Meld::Daiminkan(t)
            | crate::hand::Meld::Ankan(t)
            | crate::hand::Meld::Kakan(t) => {
                for tile in [*t, *t, *t, *t] {
                    let idx = tile as usize;
                    if idx < counts.len() {
                        counts[idx] += 1;
                    }
                }
            }
        }
    }

    let mut open_melds = Vec::new();
    let mut closed_melds = Vec::new();
    let mut kan_count = 0;

    open_melds_input.iter().for_each(|meld| match meld {
        crate::hand::Meld::Chii { called, .. } => open_melds.push(MeldKind::Sequence(*called)),
        crate::hand::Meld::Pon(tile) => open_melds.push(MeldKind::Triplet(*tile)),
        crate::hand::Meld::Daiminkan(tile) | crate::hand::Meld::Kakan(tile) => {
            open_melds.push(MeldKind::Quad(*tile));
            kan_count += 1;
        }
        crate::hand::Meld::Ankan(tile) => {
            closed_melds.push(MeldKind::Quad(*tile));
            kan_count += 1;
        }
    });

    // 文脈（Context）で提供された数よりも導出されたカンの数が多い場合、三槓子等のために導出されたカン数を使用します
    if kan_count > ctx.kan_count {
        ctx.kan_count = kan_count;
    }

    let mut closed_counts = [0usize; 35];
    for tile in tiles.iter().copied() {
        let idx = tile as usize;
        if idx < closed_counts.len() {
            closed_counts[idx] += 1;
        }
    }

    // 手牌のパターン生成には、副露（鳴き）を除外した門前（メンゼン）の牌のみを使用します。
    // そうしないと、副露した牌を再度パースしようとしてしまいます。
    let patterns = generate_patterns(&closed_counts, &open_melds, &closed_melds);

    result.extend(check_situational_yaku(&ctx));

    let yakuman_list = check_yakuman_yaku(&counts, tiles.len());
    if !yakuman_list.is_empty() {
        result.extend(yakuman_list);
        retain_yakuman_only(&mut result);
        return result;
    }

    if ctx.is_closed {
        if has_ryanpeiko(&patterns) {
            result.insert(YakuId::Ryanpeiko);
        } else if has_ipeiko(&patterns) {
            result.insert(YakuId::Ipeiko);
        }
    }

    if is_chitoitsu(&counts) && ctx.is_closed && !result.contains(&YakuId::Ryanpeiko) {
        result.insert(YakuId::Chitoitsu);
    }

    if is_tanyao(&counts) {
        result.insert(YakuId::Tanyao);
    }

    if let Some(y) = detect_pinfu(&patterns, &ctx) {
        if y {
            result.insert(YakuId::Pinfu);
        }
    }

    if let Some(p) = patterns.first() {
        if contains_yakuhai(&counts, ctx.seat_wind, ctx.round_wind) {
            if counts[TileName::White as usize] >= 3 {
                result.insert(YakuId::YakuhaiHaku);
            }
            if counts[TileName::Green as usize] >= 3 {
                result.insert(YakuId::YakuhaiHatsu);
            }
            if counts[TileName::Red as usize] >= 3 {
                result.insert(YakuId::YakuhaiChun);
            }
            if let Some(seat) = ctx.seat_wind {
                let idx = seat as usize;
                if counts[idx] >= 3 {
                    result.insert(YakuId::YakuhaiJikaze);
                }
            }
            if let Some(round) = ctx.round_wind {
                let idx = round as usize;
                if counts[idx] >= 3 {
                    result.insert(YakuId::YakuhaiBakaze);
                }
            }
        }

        if is_toitoi(&patterns) {
            result.insert(YakuId::Toitoi);
        }
        if is_sanankou(&patterns, &ctx) {
            result.insert(YakuId::Sanankou);
        }
        if is_shousangen(p) {
            result.insert(YakuId::Shousangen);
        }
        if is_chantaiyao(&patterns) {
            result.insert(YakuId::Chantaiyao);
        }
        if has_sanshoku_doujun(&patterns) {
            result.insert(YakuId::SanshokuDoujun);
        }
        if has_sanshoku_doukou(&patterns) {
            result.insert(YakuId::SanshokuDoukou);
        }
        if is_junchan(&patterns) {
            result.insert(YakuId::Junchan);
        }
        if ctx.kan_count >= 3 {
            result.insert(YakuId::Sankantsu);
        }
        if ctx.kan_count >= 4 {
            result.insert(YakuId::Suukantsu);
        }
        if is_suuankou(&patterns, ctx.is_closed, &ctx) {
            result.insert(YakuId::Suuankou);
        }
    }

    if is_honitsu(&counts) {
        result.insert(YakuId::Honitsu);
    }
    if is_chinitsu(&counts) {
        result.insert(YakuId::Chinitsu);
    }
    if is_honroutou(&counts) {
        result.insert(YakuId::Honroutou);
    }
    retain_yakuman_only(&mut result);

    result
}

pub fn is_number_tile(tile: TileName) -> Option<(usize, usize)> {
    match tile {
        TileName::OneM => Some((0, 1)),
        TileName::TwoM => Some((0, 2)),
        TileName::ThreeM => Some((0, 3)),
        TileName::FourM => Some((0, 4)),
        TileName::FiveM => Some((0, 5)),
        TileName::SixM => Some((0, 6)),
        TileName::SevenM => Some((0, 7)),
        TileName::EightM => Some((0, 8)),
        TileName::NineM => Some((0, 9)),
        TileName::OneP => Some((1, 1)),
        TileName::TwoP => Some((1, 2)),
        TileName::ThreeP => Some((1, 3)),
        TileName::FourP => Some((1, 4)),
        TileName::FiveP => Some((1, 5)),
        TileName::SixP => Some((1, 6)),
        TileName::SevenP => Some((1, 7)),
        TileName::EightP => Some((1, 8)),
        TileName::NineP => Some((1, 9)),
        TileName::OneS => Some((2, 1)),
        TileName::TwoS => Some((2, 2)),
        TileName::ThreeS => Some((2, 3)),
        TileName::FourS => Some((2, 4)),
        TileName::FiveS => Some((2, 5)),
        TileName::SixS => Some((2, 6)),
        TileName::SevenS => Some((2, 7)),
        TileName::EightS => Some((2, 8)),
        TileName::NineS => Some((2, 9)),
        _ => None,
    }
}

fn is_terminal(tile: TileName) -> bool {
    matches!(
        tile,
        TileName::OneM
            | TileName::NineM
            | TileName::OneP
            | TileName::NineP
            | TileName::OneS
            | TileName::NineS
    )
}

fn is_honor(tile: TileName) -> bool {
    matches!(
        tile,
        TileName::East
            | TileName::South
            | TileName::West
            | TileName::North
            | TileName::Red
            | TileName::Green
            | TileName::White
    )
}

fn is_terminal_or_honor(tile: TileName) -> bool {
    is_terminal(tile) || is_honor(tile)
}

fn is_simple(tile: TileName) -> bool {
    matches!(
        tile,
        TileName::TwoM
            | TileName::ThreeM
            | TileName::FourM
            | TileName::FiveM
            | TileName::SixM
            | TileName::SevenM
            | TileName::EightM
            | TileName::TwoP
            | TileName::ThreeP
            | TileName::FourP
            | TileName::FiveP
            | TileName::SixP
            | TileName::SevenP
            | TileName::EightP
            | TileName::TwoS
            | TileName::ThreeS
            | TileName::FourS
            | TileName::FiveS
            | TileName::SixS
            | TileName::SevenS
            | TileName::EightS
    )
}

fn generate_patterns(
    counts: &[usize; 35],
    open_melds: &[MeldKind],
    closed_melds: &[MeldKind],
) -> Vec<HandPattern> {
    let mut patterns = Vec::new();

    for i in 1..counts.len() {
        if counts[i] < 2 {
            continue;
        }
        let mut working = *counts;
        working[i] -= 2;
        let pair = TileName::from_usize(i);
        let mut melds = closed_melds.to_vec();
        search_melds(&mut working, &mut melds, &mut patterns, pair, open_melds);
    }

    patterns
}

fn search_melds(
    counts: &mut [usize; 35],
    melds: &mut Vec<MeldKind>,
    patterns: &mut Vec<HandPattern>,
    pair: TileName,
    open_melds: &[MeldKind],
) {
    let Some(i) = counts
        .iter()
        .enumerate()
        .skip(1)
        .position(|(_, &c)| c > 0)
        .map(|p| p + 1)
    else {
        patterns.push(HandPattern {
            pair,
            melds: melds.clone(),
            open_melds: open_melds.to_vec(),
        });
        return;
    };

    let tile = TileName::from_usize(i);

    if counts[i] >= 3 {
        counts[i] -= 3;
        melds.push(MeldKind::Triplet(tile));
        search_melds(counts, melds, patterns, pair, open_melds);
        melds.pop();
        counts[i] += 3;
    }

    let Some((suit, rank)) = is_number_tile(tile) else {
        return;
    };
    if rank > 7 {
        return;
    }

    let next1 = i + 1;
    let next2 = i + 2;

    let Some((s1, r1)) = is_number_tile(TileName::from_usize(next1)) else {
        return;
    };
    let Some((s2, r2)) = is_number_tile(TileName::from_usize(next2)) else {
        return;
    };

    if s1 != suit || s2 != suit || r1 != rank + 1 || r2 != rank + 2 {
        return;
    }
    if counts[next1] == 0 || counts[next2] == 0 {
        return;
    }

    counts[i] -= 1;
    counts[next1] -= 1;
    counts[next2] -= 1;
    melds.push(MeldKind::Sequence(tile));
    search_melds(counts, melds, patterns, pair, open_melds);
    melds.pop();
    counts[i] += 1;
    counts[next1] += 1;
    counts[next2] += 1;
}

fn is_tanyao(counts: &[usize; 35]) -> bool {
    counts
        .iter()
        .enumerate()
        .skip(1)
        .all(|(i, count)| *count == 0 || is_simple(TileName::from_usize(i)))
}

fn has_ipeiko(patterns: &[HandPattern]) -> bool {
    patterns.iter().any(|pattern| {
        let mut sequences: HashMap<TileName, usize> = HashMap::new();
        for meld in pattern.all_melds() {
            if let MeldKind::Sequence(tile) = meld {
                *sequences.entry(*tile).or_default() += 1;
            }
        }
        sequences.values().any(|v| *v >= 2)
    })
}

fn has_ryanpeiko(patterns: &[HandPattern]) -> bool {
    patterns.iter().any(|pattern| {
        if pattern
            .all_melds()
            .any(|m| !matches!(m, MeldKind::Sequence(_)))
        {
            return false;
        }
        let mut sequences: HashMap<TileName, usize> = HashMap::new();
        for meld in pattern.all_melds() {
            if let MeldKind::Sequence(tile) = meld {
                *sequences.entry(*tile).or_default() += 1;
            }
        }
        let mut pairs = 0;
        for value in sequences.values() {
            pairs += value / 2;
        }
        pairs >= 2
    })
}

fn detect_pinfu(patterns: &[HandPattern], ctx: &WinContext) -> Option<bool> {
    if !ctx.is_closed {
        return Some(false);
    }

    let win_tile = ctx.win_tile?;
    let (win_suit, win_rank) = is_number_tile(win_tile)?;

    for pattern in patterns {
        if pattern
            .all_melds()
            .any(|m| matches!(m, MeldKind::Triplet(_) | MeldKind::Quad(_)))
        {
            continue;
        }

        if is_value_pair(pattern.pair, ctx) || win_tile == pattern.pair {
            continue;
        }

        // アガリの形（14枚）から順子を探すのではなく、
        // アガリ牌を抜いた13枚のテンパイ形において、
        // アガリ牌が「両面塔子」を完成させているかを検証します。
        // パターン内の各順子からアガリ牌を抜いて塔子を作り、
        // それが両面塔子（両面待ち）であるかをチェックします。
        let is_ryamen = pattern.all_melds().any(|meld| {
            let MeldKind::Sequence(start_tile) = meld else {
                return false;
            };
            let Some((suit, rank)) = is_number_tile(*start_tile) else {
                return false;
            };

            if suit != win_suit {
                return false;
            }

            // アガリ牌がこの順子を構成している場合、抜くと塔子になります
            if win_rank == rank {
                // アガリ牌が下端の場合、残りの塔子は (rank+1, rank+2)
                // これが両面待ちである条件は、上端の次が存在すること (rank+2 < 9 すなわち rank < 7) です
                rank < 7
            } else if win_rank == rank + 2 {
                // アガリ牌が上端の場合、残りの塔子は (rank, rank+1)
                // これが両面待ちである条件は、下端の前が存在すること (rank > 1) です
                rank > 1
            } else {
                false
            }
        });

        if is_ryamen {
            return Some(true);
        }
    }

    Some(false)
}

fn contains_yakuhai(
    counts: &[usize; 35],
    seat_wind: Option<TileName>,
    round_wind: Option<TileName>,
) -> bool {
    counts[TileName::White as usize] >= 3
        || counts[TileName::Green as usize] >= 3
        || counts[TileName::Red as usize] >= 3
        || seat_wind.map(|w| counts[w as usize] >= 3).unwrap_or(false)
        || round_wind.map(|w| counts[w as usize] >= 3).unwrap_or(false)
}

fn is_value_pair(tile: TileName, ctx: &WinContext) -> bool {
    matches!(tile, TileName::Red | TileName::Green | TileName::White)
        || ctx.seat_wind.map(|w| w == tile).unwrap_or(false)
        || ctx.round_wind.map(|w| w == tile).unwrap_or(false)
}

fn is_chitoitsu(counts: &[usize; 35]) -> bool {
    let pair_count = counts.iter().skip(1).filter(|c| **c == 2).count();
    pair_count == 7
}

const TERMINALS_AND_HONORS: [usize; 13] = [
    TileName::OneM as usize,
    TileName::NineM as usize,
    TileName::OneP as usize,
    TileName::NineP as usize,
    TileName::OneS as usize,
    TileName::NineS as usize,
    TileName::East as usize,
    TileName::South as usize,
    TileName::West as usize,
    TileName::North as usize,
    TileName::Red as usize,
    TileName::Green as usize,
    TileName::White as usize,
];

fn is_kokushi(counts: &[usize; 35]) -> bool {
    let (missing, has_pair) = TERMINALS_AND_HONORS
        .iter()
        .fold((0, false), |(m, p), &idx| {
            if counts[idx] == 0 {
                (m + 1, p)
            } else if counts[idx] >= 2 {
                (m, true)
            } else {
                (m, p)
            }
        });

    missing == 0 && has_pair
}

fn is_toitoi(patterns: &[HandPattern]) -> bool {
    patterns.iter().any(|pattern| {
        pattern
            .all_melds()
            .all(|m| matches!(m, MeldKind::Triplet(_) | MeldKind::Quad(_)))
    })
}

fn is_sanankou(patterns: &[HandPattern], ctx: &WinContext) -> bool {
    patterns.iter().any(|pattern| {
        let mut count = pattern
            .melds
            .iter()
            .filter(|m| matches!(m, MeldKind::Triplet(_) | MeldKind::Quad(_)))
            .count();

        if !ctx.is_tsumo {
            if let Some(win_tile) = ctx.win_tile {
                // ロンアガリの牌が暗刻（アンコウ）を完成させた場合、それは明刻（ミンコウ）扱いになるため1を引きます
                let matches_ron = pattern.melds.iter().any(|m| match m {
                    MeldKind::Triplet(t) | MeldKind::Quad(t) => *t == win_tile,
                    _ => false,
                });
                if matches_ron {
                    count -= 1;
                }
            }
        }
        count >= 3
    })
}

fn is_shousangen(pattern: &HandPattern) -> bool {
    let mut dragon_triplets = 0;
    let mut dragon_pair = false;

    for meld in pattern.all_melds() {
        match meld {
            MeldKind::Triplet(tile) | MeldKind::Quad(tile) => {
                if matches!(tile, TileName::Red | TileName::Green | TileName::White) {
                    dragon_triplets += 1;
                }
            }
            _ => {}
        }
    }

    if matches!(
        pattern.pair,
        TileName::Red | TileName::Green | TileName::White
    ) {
        dragon_pair = true;
    }

    dragon_triplets == 2 && dragon_pair
}

fn is_daisangen(counts: &[usize; 35]) -> bool {
    counts[TileName::Red as usize] >= 3
        && counts[TileName::Green as usize] >= 3
        && counts[TileName::White as usize] >= 3
}

fn is_chantaiyao(patterns: &[HandPattern]) -> bool {
    patterns.iter().any(|pattern| {
        if !is_terminal_or_honor(pattern.pair) {
            return false;
        }

        pattern.all_melds().all(|meld| match meld {
            MeldKind::Sequence(tile) => matches!(is_number_tile(*tile), Some((_, 1 | 7))),
            MeldKind::Triplet(tile) | MeldKind::Quad(tile) => is_terminal_or_honor(*tile),
        })
    })
}

fn has_sanshoku_doujun(patterns: &[HandPattern]) -> bool {
    patterns.iter().any(|pattern| {
        let mut map: HashMap<usize, HashSet<usize>> = HashMap::new();
        for meld in pattern.all_melds() {
            if let MeldKind::Sequence(tile) = meld {
                if let Some((suit, rank)) = is_number_tile(*tile) {
                    map.entry(rank).or_default().insert(suit);
                }
            }
        }
        map.values().any(|set| set.len() == 3)
    })
}

fn has_sanshoku_doukou(patterns: &[HandPattern]) -> bool {
    patterns.iter().any(|pattern| {
        let mut map: HashMap<usize, HashSet<usize>> = HashMap::new();
        for meld in pattern.all_melds() {
            match meld {
                MeldKind::Triplet(tile) | MeldKind::Quad(tile) => {
                    if let Some((suit, rank)) = is_number_tile(*tile) {
                        map.entry(rank).or_default().insert(suit);
                    }
                }
                _ => {}
            }
        }
        map.values().any(|set| set.len() == 3)
    })
}

fn is_honitsu(counts: &[usize; 35]) -> bool {
    let mut suit_seen = None;
    let mut has_number = false;
    for (i, count) in counts.iter().enumerate().skip(1) {
        if *count == 0 {
            continue;
        }
        let tile = TileName::from_usize(i);
        if let Some((suit, _)) = is_number_tile(tile) {
            has_number = true;
            match suit_seen {
                None => suit_seen = Some(suit),
                Some(s) if s != suit => return false,
                _ => {}
            }
        }
    }
    suit_seen.is_some() && has_number
}

fn is_chinitsu(counts: &[usize; 35]) -> bool {
    let mut suit_seen = None;
    for (i, count) in counts.iter().enumerate().skip(1) {
        if *count == 0 {
            continue;
        }
        let tile = TileName::from_usize(i);
        if is_honor(tile) {
            return false;
        }
        if let Some((suit, _)) = is_number_tile(tile) {
            match suit_seen {
                None => suit_seen = Some(suit),
                Some(s) if s != suit => return false,
                _ => {}
            }
        }
    }
    suit_seen.is_some()
}

fn is_junchan(patterns: &[HandPattern]) -> bool {
    patterns.iter().any(|pattern| {
        if !is_terminal(pattern.pair) {
            return false;
        }
        pattern.all_melds().all(|meld| match meld {
            MeldKind::Sequence(tile) => matches!(is_number_tile(*tile), Some((_, 1 | 7))),
            MeldKind::Triplet(tile) | MeldKind::Quad(tile) => is_terminal(*tile),
        })
    })
}

fn is_honroutou(counts: &[usize; 35]) -> bool {
    counts
        .iter()
        .enumerate()
        .skip(1)
        .all(|(i, c)| *c == 0 || is_terminal_or_honor(TileName::from_usize(i)))
}

fn is_chinroutou(counts: &[usize; 35]) -> bool {
    counts
        .iter()
        .enumerate()
        .skip(1)
        .all(|(i, c)| *c == 0 || is_terminal(TileName::from_usize(i)))
}

fn detect_suushi(counts: &[usize; 35]) -> Option<(bool, bool)> {
    let winds = [
        TileName::East as usize,
        TileName::South as usize,
        TileName::West as usize,
        TileName::North as usize,
    ];

    let triplets = winds.iter().filter(|i| counts[**i] >= 3).count();
    let pairs = winds.iter().filter(|i| counts[**i] >= 2).count();
    let all = winds.iter().all(|i| counts[*i] >= 3);
    let small = triplets == 3 && pairs == 4;
    Some((small, all))
}

fn is_tsuuiisou(counts: &[usize; 35]) -> bool {
    counts
        .iter()
        .enumerate()
        .skip(1)
        .all(|(i, c)| *c == 0 || is_honor(TileName::from_usize(i)))
}

fn is_ryuuiisou(counts: &[usize; 35]) -> bool {
    let allowed = [
        TileName::TwoS as usize,
        TileName::ThreeS as usize,
        TileName::FourS as usize,
        TileName::SixS as usize,
        TileName::EightS as usize,
        TileName::Green as usize,
    ];
    let allowed_set: HashSet<usize> = allowed.iter().copied().collect();

    counts
        .iter()
        .enumerate()
        .skip(1)
        .all(|(i, c)| *c == 0 || allowed_set.contains(&i))
}

fn is_suuankou(patterns: &[HandPattern], closed: bool, ctx: &WinContext) -> bool {
    closed
        && patterns.iter().any(|pattern| {
            let mut all_triplets = pattern
                .melds
                .iter()
                .all(|m| matches!(m, MeldKind::Triplet(_) | MeldKind::Quad(_)));

            if all_triplets && !ctx.is_tsumo {
                if let Some(win_tile) = ctx.win_tile {
                    let completes_triplet = pattern.melds.iter().any(|m| match m {
                        MeldKind::Triplet(t) | MeldKind::Quad(t) => *t == win_tile,
                        _ => false,
                    });

                    if completes_triplet {
                        // 四暗刻の場合、暗刻に対するロンアガリは「暗刻3＋明刻1」となり、四暗刻ではなく三暗刻・対々和になります。
                        // ただし、単騎待ち（すでに暗刻が4つあり、雀頭を待つ形）の場合は例外です。
                        // これを正確に判定するため、雀頭がロンアガリ牌であるかを検証します。
                        if pattern.pair != win_tile {
                            all_triplets = false;
                        }
                    }
                }
            }
            all_triplets
        })
}

fn is_chuuren_poutou(counts: &[usize; 35], tiles_len: usize) -> bool {
    if tiles_len != 14 {
        return false;
    }
    let suits = [0usize, 1, 2];
    for suit in suits {
        let mut required = [0usize; 9];
        required[0] = 3;
        required[1] = 1;
        required[2] = 1;
        required[3] = 1;
        required[4] = 1;
        required[5] = 1;
        required[6] = 1;
        required[7] = 1;
        required[8] = 3;

        let mut valid = true;
        let mut extra = 0;
        for rank in 1..=9 {
            let tile = match suit {
                0 => TileName::from_usize(rank),
                1 => TileName::from_usize(rank + 9),
                _ => TileName::from_usize(rank + 18),
            };
            let count = counts[tile as usize];
            if count < required[rank - 1] {
                valid = false;
                break;
            }
            extra += count - required[rank - 1];
        }
        if valid && extra == 1 {
            return true;
        }
    }
    false
}

fn check_situational_yaku(ctx: &WinContext) -> Vec<YakuId> {
    let mut yaku = Vec::new();
    if ctx.is_double_riichi && ctx.is_closed {
        yaku.push(YakuId::DoubleRiichi);
    } else if ctx.riichi && ctx.is_closed {
        yaku.push(YakuId::Riichi);
    }

    if ctx.is_ippatsu && ctx.is_closed && (ctx.riichi || ctx.is_double_riichi) {
        yaku.push(YakuId::Ippatsu);
    }

    if ctx.is_rinshan && ctx.is_tsumo {
        yaku.push(YakuId::RinshanKaihou);
    }

    if ctx.is_chankan && !ctx.is_tsumo {
        yaku.push(YakuId::Chankan);
    }

    if ctx.is_haitei && ctx.is_tsumo {
        yaku.push(YakuId::HaiteiRaoyue);
    }

    if ctx.is_houtei && !ctx.is_tsumo {
        yaku.push(YakuId::HouteiRaoyui);
    }

    if ctx.is_closed && ctx.is_tsumo {
        yaku.push(YakuId::MenzenTsumo);
    }

    if ctx.tenhou {
        yaku.push(YakuId::Tenhou);
    }
    if ctx.chiihou {
        yaku.push(YakuId::Chiihou);
    }
    yaku
}

fn check_yakuman_yaku(counts: &[usize; 35], tiles_len: usize) -> Vec<YakuId> {
    let mut yaku = Vec::new();
    if is_kokushi(counts) {
        yaku.push(YakuId::KokushiMusou);
    }
    if is_chinroutou(counts) {
        yaku.push(YakuId::Chinroutou);
    }
    if is_daisangen(counts) {
        yaku.push(YakuId::Daisangen);
    }
    if let Some((small, big)) = detect_suushi(counts) {
        if big {
            yaku.push(YakuId::Daisuushi);
        }
        if small {
            yaku.push(YakuId::Shousuushi);
        }
    }
    if is_tsuuiisou(counts) {
        yaku.push(YakuId::Tsuuiisou);
    }
    if is_ryuuiisou(counts) {
        yaku.push(YakuId::Ryuuiisou);
    }
    if is_chuuren_poutou(counts, tiles_len) {
        yaku.push(YakuId::ChuurenPoutou);
    }
    yaku
}

fn retain_yakuman_only(result: &mut std::collections::HashSet<YakuId>) {
    let has_yakuman = result.iter().any(|&id| {
        ALL_YAKU
            .iter()
            .find(|y| y.id == id)
            .is_some_and(|y| y.yakuman)
    });
    if has_yakuman {
        result.retain(|&id| {
            ALL_YAKU
                .iter()
                .find(|y| y.id == id)
                .is_some_and(|y| y.yakuman)
        });
    }
}
