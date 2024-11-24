use std::{collections::HashSet, iter::zip};

use bitvec::prelude::*;

use crate::tile::Tile;

/// 手牌の数
#[allow(dead_code)]
pub const TILE_COUNT: usize = 14;

#[allow(dead_code)]
pub const STANDARD_SET_COUNT: usize = 4;

#[allow(dead_code)]
pub const SET_TILE_COUNT: usize = 3;

#[allow(dead_code)]
pub const PAIR_TILE_COUNT: usize = 2;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Hand {
    pub tiles: Vec<Tile>,
}

impl Hand {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { tiles: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn push_tile(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }

    #[allow(dead_code)]
    pub fn pop_tile(&mut self, index: usize) {
        self.tiles.remove(index);
    }

    #[allow(dead_code)]
    pub fn sort(&mut self) {
        self.tiles.sort_by(|a, b| a.name.cmp(&b.name));
    }

    #[allow(dead_code)]
    pub fn get_delta(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        for i in 0..self.tiles.len() - 1 {
            let delta = (self.tiles[i + 1].name as u8) - (self.tiles[i].name as u8);
            result.push(delta);
        }

        result
    }

    /// 順子を探す
    #[allow(dead_code)]
    pub fn get_chows(&self) -> Vec<usize> {
        let deltas = self.get_delta();
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
    pub fn get_pungs(&self) -> Vec<usize> {
        let deltas = self.get_delta();
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
    pub fn get_kongs(&self) -> Vec<usize> {
        let deltas = self.get_delta();
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
    pub fn get_pairs(&self) -> Vec<usize> {
        let deltas = self.get_delta();
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

    /// 4面子、1雀頭の形を探す
    #[allow(dead_code)]
    pub fn get_standard(&self) -> Vec<Vec<usize>> {
        let chows = self.get_chows();
        let pungs = self.get_pungs();

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

            let pairs = self.get_pairs();
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
}
