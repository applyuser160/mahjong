use std::{collections::HashSet, iter::zip};

use bitvec::prelude::*;

use crate::tile::{Tile, TileName};

/// 手牌の数
#[allow(dead_code)]
pub const TILE_COUNT: usize = 14;

#[allow(dead_code)]
pub const STANDARD_SET_COUNT: usize = 4;

#[allow(dead_code)]
pub const KONG_TILE_COUNT: usize = 4;

#[allow(dead_code)]
pub const SET_TILE_COUNT: usize = 3;

#[allow(dead_code)]
pub const PAIR_TILE_COUNT: usize = 2;

#[allow(dead_code)]
pub enum HandName {
    AllSimples,
    AllRuns,
    DoubleRuns,
    ValueTiles,
    SevenPairs,
    AllTriples,
    ThreeConcealedTriples,
    ThreeColorTriples,
    ThreeColorRuns,
    AllTerminalsAndHonors,
    FullStraight,
    MixedOutsideHand,
    LittleDragons,
    ThreeQuads,
    HalfFlash,
    PureOutsideHand,
    TwoDoubleRuns,
    FullFlash,
    AllGreen,
    BigDragons,
    LittleFourWinds,
    AllHonors,
    ThirteenOrphans,
    NineTreasure,
    FourConcealedTriples,
    AllTerminals,
    FourQuads,
}

impl HandName {
    pub fn get_calling_length(&self) -> usize {
        match self {
            HandName::AllSimples => 1,
            HandName::AllRuns => 0,
            HandName::DoubleRuns => 0,
            HandName::ValueTiles => 1,
            HandName::SevenPairs => 0,
            HandName::AllTriples => 2,
            HandName::ThreeConcealedTriples => 2,
            HandName::ThreeColorTriples => 2,
            HandName::ThreeColorRuns => 1,
            HandName::AllTerminalsAndHonors => 2,
            HandName::FullStraight => 1,
            HandName::MixedOutsideHand => 1,
            HandName::LittleDragons => 2,
            HandName::ThreeQuads => 2,
            HandName::HalfFlash => 2,
            HandName::PureOutsideHand => 2,
            HandName::TwoDoubleRuns => 0,
            HandName::FullFlash => 5,
            HandName::AllGreen => 5,
            HandName::BigDragons => 5,
            HandName::LittleFourWinds => 5,
            HandName::AllHonors => 5,
            HandName::ThirteenOrphans => 0,
            HandName::NineTreasure => 0,
            HandName::FourConcealedTriples => 0,
            HandName::AllTerminals => 5,
            HandName::FourQuads => 5,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum HonorType {
    Chow,
    Pung,
    Kong,
    Pair,
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Honor {
    pub chows: Vec<usize>,
    pub pungs: Vec<usize>,
    pub kongs: Vec<usize>,
    pub pairs: Vec<usize>,
}

impl Honor {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            chows: Vec::new(),
            pungs: Vec::new(),
            kongs: Vec::new(),
            pairs: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from(
        chows: Vec<usize>,
        pungs: Vec<usize>,
        kongs: Vec<usize>,
        pairs: Vec<usize>,
    ) -> Self {
        Self {
            chows,
            pungs,
            kongs,
            pairs,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct HonorBit {
    pub chows: Vec<BitVec>,
    pub pungs: Vec<BitVec>,
    pub kongs: Vec<BitVec>,
    pub pairs: Vec<BitVec>,
}

impl HonorBit {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            chows: Vec::new(),
            pungs: Vec::new(),
            kongs: Vec::new(),
            pairs: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn from_index(
        chows: Vec<usize>,
        pungs: Vec<usize>,
        kongs: Vec<usize>,
        pairs: Vec<usize>,
    ) -> Self {
        Self {
            chows: Hand::convert_to_bit(&chows, SET_TILE_COUNT),
            pungs: Hand::convert_to_bit(&pungs, SET_TILE_COUNT),
            kongs: Hand::convert_to_bit(&kongs, KONG_TILE_COUNT),
            pairs: Hand::convert_to_bit(&pairs, PAIR_TILE_COUNT),
        }
    }

    #[allow(dead_code)]
    pub fn from_honor(honor: Honor) -> Self {
        Self {
            chows: Hand::convert_to_bit(&honor.chows, SET_TILE_COUNT),
            pungs: Hand::convert_to_bit(&honor.pungs, SET_TILE_COUNT),
            kongs: Hand::convert_to_bit(&honor.kongs, KONG_TILE_COUNT),
            pairs: Hand::convert_to_bit(&honor.pairs, PAIR_TILE_COUNT),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct HonorInfo {
    index: usize,
    bit: BitVec,
    honor_type: HonorType,
}

impl HonorInfo {
    #[allow(dead_code)]
    pub fn from(index: usize, bit: BitVec, honor_type: HonorType) -> Self {
        Self {
            index,
            bit,
            honor_type,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Hand {
    pub tiles: Vec<Tile>,
    pub draw: Option<Tile>,
    pub chows: Vec<Tile>,
    pub pungs: Vec<Tile>,
    pub kongs: Vec<Tile>,
    pub pairs: Vec<Tile>,
}

impl Hand {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tiles: Vec::new(),
            draw: None,
            chows: Vec::new(),
            pungs: Vec::new(),
            kongs: Vec::new(),
            pairs: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn push_tile(tiles: &mut Vec<Tile>, tile: Tile) {
        tiles.push(tile);
    }

    #[allow(dead_code)]
    pub fn pop_tile(tiles: &mut Vec<Tile>, index: usize) {
        tiles.remove(index);
    }

    #[allow(dead_code)]
    pub fn sort(tiles: &mut Vec<Tile>) {
        tiles.sort_by(|a, b| a.name.cmp(&b.name));
    }

    #[allow(dead_code)]
    pub fn get_delta(tiles: &Vec<Tile>) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for i in 0..tiles.len() - 1 {
            let delta = (tiles[i + 1].name as u8) - (tiles[i].name as u8);
            result.push(delta);
        }

        result
    }

    /// 順子を探す
    #[allow(dead_code)]
    pub fn get_chows(tiles: &Vec<Tile>) -> Vec<usize> {
        let deltas = Hand::get_delta(tiles);
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() - 1 {
            if deltas[i] == 1 && deltas[i + 1] == 1 {
                result.push(i);
            }
        }

        result
    }

    /// 刻子を探す
    #[allow(dead_code)]
    pub fn get_pungs(tiles: &Vec<Tile>) -> Vec<usize> {
        let deltas = Hand::get_delta(tiles);
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() - 1 {
            if deltas[i] == 0 && deltas[i + 1] == 0 {
                result.push(i);
            }
        }

        result
    }

    /// カンを探す
    #[allow(dead_code)]
    pub fn get_kongs(tiles: &Vec<Tile>) -> Vec<usize> {
        let deltas = Hand::get_delta(tiles);
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() - 2 {
            if deltas[i] == 0 && deltas[i + 1] == 0 && deltas[i + 2] == 0 {
                result.push(i);
            }
        }

        result
    }

    /// 対子を探す
    #[allow(dead_code)]
    pub fn get_pairs(tiles: &Vec<Tile>) -> Vec<usize> {
        let deltas = Hand::get_delta(tiles);
        let mut result: Vec<usize> = Vec::new();

        for i in 0..deltas.len() {
            if deltas[i] == 0 {
                result.push(i);
            }
        }

        result
    }

    /// indexのVecをBitVecのVecに変換する
    #[allow(dead_code)]
    pub fn convert_to_bit(indexes: &Vec<usize>, unit_size: usize) -> Vec<BitVec> {
        let mut bit_sets: Vec<BitVec> = Vec::new();
        for index in indexes {
            let mut bit_set = bitvec![0; TILE_COUNT];
            for i in *index..(*index) + unit_size {
                bit_set.set(i, true);
            }
            bit_sets.push(bit_set);
        }
        bit_sets
    }

    /// BitVecの重ね合わせ、重なりがなかった場合、orを返す
    #[allow(dead_code)]
    pub fn get_able_or(left: &BitVec, right: &BitVec) -> Option<BitVec> {
        let or = left.clone() | right;
        let xor = left.clone() ^ right;
        if or == xor {
            Some(or)
        } else {
            None
        }
    }

    /// 先頭の牌と、面子のタイプを指定して、残りの牌を含めた面子を返す
    #[allow(dead_code)]
    pub fn get_all_tiles(tile: Tile, honor_type: HonorType) -> Vec<Tile> {
        let addtion: usize = if honor_type == HonorType::Chow { 1 } else { 0 };
        let length = match honor_type {
            HonorType::Chow => SET_TILE_COUNT,
            HonorType::Pung => SET_TILE_COUNT,
            HonorType::Kong => KONG_TILE_COUNT,
            HonorType::Pair => PAIR_TILE_COUNT,
        };
        let mut result: Vec<Tile> = Vec::new();
        for i in 0..length {
            let tile_name = TileName::from_usize((tile.name as usize) + (i * addtion));
            let new_tile = Tile::from_name(tile_name, false);
            result.push(new_tile);
        }
        result
    }

    /// 槓子、刻子、順子、対子を指定して、探索する
    /// 優先順位は、槓子->刻子->順子->対子の順
    #[allow(dead_code)]
    pub fn find_honors(
        tiles: &Vec<Tile>,
        has_kong: bool,
        has_pung: bool,
        has_chow: bool,
        has_pair: bool,
    ) -> Vec<Honor> {
        // 各面子ができるインデックスを取得する
        let simple_honor = Honor::from(
            if has_chow {
                Hand::get_chows(tiles)
            } else {
                Vec::new()
            },
            if has_pung {
                Hand::get_pungs(tiles)
            } else {
                Vec::new()
            },
            if has_kong {
                Hand::get_kongs(tiles)
            } else {
                Vec::new()
            },
            if has_pair {
                Hand::get_pairs(tiles)
            } else {
                Vec::new()
            },
        );

        // bitvecを生成する
        let honor_bit = HonorBit::from_honor(simple_honor.clone());

        // 優先順位が高い順にマッチする組み合わせを探す
        let mut result: Vec<Honor> = Vec::new();

        let kongs_len = honor_bit.kongs.len();
        let pungs_len = honor_bit.pungs.len();
        let chows_len = honor_bit.chows.len();
        let pairs_len = honor_bit.pairs.len();

        for i in 0..=3 {
            let len = match i {
                0 => kongs_len,
                1 => kongs_len + pungs_len,
                2 => kongs_len + pungs_len + chows_len,
                3 => kongs_len + pungs_len + chows_len + pairs_len,
                _ => unreachable!(),
            };

            // 可能性のある面子を昇順に並べる
            let mut honor_infos: Vec<HonorInfo> = Vec::new();
            for j in 0..len {
                let ind: usize;
                let start_indexs: &Vec<usize>;
                let bits: &Vec<BitVec>;
                let honor_type: HonorType;
                if j < kongs_len {
                    ind = j;
                    start_indexs = &simple_honor.kongs;
                    bits = &honor_bit.kongs;
                    honor_type = HonorType::Kong;
                } else if j < kongs_len + pungs_len {
                    ind = j - kongs_len;
                    start_indexs = &simple_honor.pungs;
                    bits = &honor_bit.pungs;
                    honor_type = HonorType::Pung;
                } else if j < kongs_len + pungs_len + chows_len {
                    ind = j - (kongs_len + pungs_len);
                    start_indexs = &simple_honor.chows;
                    bits = &honor_bit.chows;
                    honor_type = HonorType::Chow;
                } else {
                    ind = j - (kongs_len + pungs_len + chows_len);
                    start_indexs = &simple_honor.pairs;
                    bits = &honor_bit.pairs;
                    honor_type = HonorType::Pair;
                }

                if bits.len() == 0 {
                    continue;
                }

                let honor_info =
                    HonorInfo::from(start_indexs[ind], bits[ind].clone(), honor_type.clone());
                honor_infos.push(honor_info);
            }

            honor_infos.sort_by(|a, b| a.index.cmp(&b.index));

            // 重ね合わせ用のbit
            let mut used_tile_bits: Vec<BitVec>;

            // indexが若い順に重ね合わせる
            for top_honor_info in &honor_infos {
                // indexが0のみ(1以降は、組み合わせが見つかっても、重複となる)
                if top_honor_info.index != 0 {
                    break;
                }

                used_tile_bits = vec![top_honor_info.bit.clone()];
                let mut top_honor = Honor::new();
                match top_honor_info.honor_type {
                    HonorType::Chow => top_honor.chows = vec![top_honor_info.index],
                    HonorType::Pung => top_honor.pungs = vec![top_honor_info.index],
                    HonorType::Kong => top_honor.kongs = vec![top_honor_info.index],
                    HonorType::Pair => top_honor.pairs = vec![top_honor_info.index],
                }
                let mut draft_results = vec![top_honor];

                while used_tile_bits.len() > 0 {
                    let used_tile_bit = used_tile_bits[0].clone();

                    for honor_info in &honor_infos {
                        let or = Hand::get_able_or(&used_tile_bit, &honor_info.bit);

                        // 重なりがあった場合
                        if or.is_none() {
                            continue;
                        }

                        let unwrap_or = or.unwrap();

                        // 隙間があった場合
                        let mut copied_or = unwrap_or.clone();
                        let copied_or_len = copied_or.len();
                        copied_or.iter_mut().for_each(|mut b| *b = !*b);
                        copied_or.shift_right(copied_or_len - honor_info.index);
                        if copied_or.any() {
                            continue;
                        }

                        used_tile_bits.push(unwrap_or);

                        let mut draft_result = draft_results[0].clone();

                        match honor_info.honor_type {
                            HonorType::Chow => draft_result.chows.push(honor_info.index),
                            HonorType::Pung => draft_result.pungs.push(honor_info.index),
                            HonorType::Kong => draft_result.kongs.push(honor_info.index),
                            HonorType::Pair => draft_result.pairs.push(honor_info.index),
                        }

                        draft_results.push(draft_result);
                    }

                    // すべて埋めきった場合
                    let mut copied_used_tile_bit = used_tile_bits[0].clone();
                    copied_used_tile_bit.iter_mut().for_each(|mut b| *b = !*b);

                    used_tile_bits.remove(0);

                    if copied_used_tile_bit.any() {
                        draft_results.remove(0);
                    }
                    if copied_used_tile_bit.not_any() {
                        if let Some(last_result) = draft_results.last() {
                            result.push(last_result.clone());
                        }
                    }
                }
            }
        }

        return result;
    }

    /// 4面子、1雀頭の形を探す
    #[allow(dead_code)]
    pub fn get_standard(tiles: &mut Vec<Tile>) -> Vec<Vec<usize>> {
        let chows = Hand::get_chows(tiles);
        let pungs = Hand::get_pungs(tiles);

        let union: HashSet<usize> = chows.iter().chain(pungs.iter()).cloned().collect();
        let mut sets: Vec<usize> = union.into_iter().collect();
        sets.sort_by(|a, b| a.cmp(&b));

        let bit_sets = Hand::convert_to_bit(&sets, SET_TILE_COUNT);

        let last_index = TILE_COUNT - STANDARD_SET_COUNT * SET_TILE_COUNT;

        let mut enable_sets: Vec<Vec<usize>> = Vec::new();
        for (i, bit_set) in zip(&sets, &bit_sets) {
            if *i > last_index {
                break;
            }

            let mut enable_set: Vec<usize> = vec![*i];
            let mut is_used_tiles = bit_set.clone();

            for (j, inner_bit_set) in zip(&sets[(i + 1)..], &bit_sets[(i + 1)..]) {
                let or = Hand::get_able_or(&is_used_tiles, &inner_bit_set);

                if or.is_none() {
                    continue;
                }

                is_used_tiles = or.unwrap();

                enable_set.push(*j);
            }

            if enable_set.len() != STANDARD_SET_COUNT {
                continue;
            }

            let pairs = Hand::get_pairs(tiles);
            let pair_bits = Hand::convert_to_bit(&pairs, PAIR_TILE_COUNT);

            for (i, bit) in zip(pairs, pair_bits) {
                let or = Hand::get_able_or(&bit, &is_used_tiles);

                if or.is_some() {
                    enable_set.push(i);
                    enable_sets.push(enable_set.clone());
                    break;
                }
            }
        }

        enable_sets
    }

    /// 鳴きの数を取得する
    #[allow(dead_code)]
    pub fn get_calling_length(&self) -> usize {
        self.chows.len() + self.pungs.len() + self.kongs.len()
    }

    /// 鳴きの制限を満たしているか
    #[allow(dead_code)]
    pub fn is_over_calling_limit(&self, calling_limig: usize) -> bool {
        self.get_calling_length() > calling_limig
    }

    /// タンヤオの判定
    #[allow(dead_code)]
    pub fn is_all_simples(&self) -> bool {
        let hand_name = HandName::AllSimples;

        // 鳴き回数を可能回数を超過している場合は、偽
        let calling_length = self.get_calling_length();
        if calling_length > hand_name.get_calling_length() {
            return false;
        }

        // ツモと手牌を足す
        let mut tiles = self.tiles.clone();
        if let Some(draw) = self.draw {
            tiles.push(draw);
        }

        true
    }

    /// 平和の判定
    #[allow(dead_code)]
    pub fn is_all_runs(&self) -> bool {
        true
    }

    /// 一盃口の判定
    #[allow(dead_code)]
    pub fn is_double_runs(&self) -> bool {
        true
    }

    /// 役牌の判定
    #[allow(dead_code)]
    pub fn is_value_tiles(&self) -> bool {
        true
    }

    /// 七対子の判定
    #[allow(dead_code)]
    pub fn is_seven_pairs(&self) -> bool {
        true
    }

    /// 対々和の判定
    #[allow(dead_code)]
    pub fn is_all_triples(&self) -> bool {
        true
    }

    /// 三暗刻の判定
    #[allow(dead_code)]
    pub fn is_three_concealed_triples(&self) -> bool {
        true
    }

    /// 三色同刻の判定
    #[allow(dead_code)]
    pub fn is_three_color_triples(&self) -> bool {
        true
    }

    /// 三色同順の判定
    #[allow(dead_code)]
    pub fn is_three_color_runs(&self) -> bool {
        true
    }

    /// 混老頭の判定
    #[allow(dead_code)]
    pub fn is_all_terminals_and_honors(&self) -> bool {
        true
    }

    /// 一気通貫の判定
    #[allow(dead_code)]
    pub fn is_full_straight(&self) -> bool {
        true
    }

    /// チャンタの判定
    #[allow(dead_code)]
    pub fn is_mixed_outside_hand(&self) -> bool {
        true
    }

    /// 小三元の判定
    #[allow(dead_code)]
    pub fn is_little_dragons(&self) -> bool {
        true
    }

    /// 三槓子の判定
    #[allow(dead_code)]
    pub fn is_three_quads(&self) -> bool {
        true
    }

    /// 混一色の判定
    #[allow(dead_code)]
    pub fn is_half_flash(&self) -> bool {
        true
    }

    /// 純チャンの判定
    #[allow(dead_code)]
    pub fn is_pure_outside_hand(&self) -> bool {
        true
    }

    /// 二盃口の判定
    #[allow(dead_code)]
    pub fn is_two_double_runs(&self) -> bool {
        true
    }

    /// 清一色の判定
    #[allow(dead_code)]
    pub fn is_full_flash(&self) -> bool {
        true
    }

    /// 緑一色の判定
    #[allow(dead_code)]
    pub fn is_all_green(&self) -> bool {
        true
    }

    /// 大三元の判定
    #[allow(dead_code)]
    pub fn is_big_dragons(&self) -> bool {
        true
    }
    /// 小四喜の判定
    #[allow(dead_code)]
    pub fn is_little_four_winds(&self) -> bool {
        true
    }

    /// 字一色の判定
    #[allow(dead_code)]
    pub fn is_all_honors(&self) -> bool {
        true
    }

    /// 国士無双の判定
    #[allow(dead_code)]
    pub fn is_thirteen_orphans(&self) -> bool {
        true
    }

    /// 九蓮宝燈の判定
    #[allow(dead_code)]
    pub fn is_nine_treasure(&self) -> bool {
        true
    }

    /// 四暗刻の判定
    #[allow(dead_code)]
    pub fn is_four_concealed_triples(&self) -> bool {
        true
    }

    /// 清老頭の判定
    #[allow(dead_code)]
    pub fn is_all_terminals(&self) -> bool {
        true
    }

    /// 四槓子の判定
    #[allow(dead_code)]
    pub fn is_four_quads(&self) -> bool {
        true
    }
}
