//! 麻雀の数牌スーツ別ルックアップテーブル (LUT)
//!
//! 各数牌スーツ（萬子・筒子・索子の各9種、枚数0〜4）の牌カウントパターンに対する
//! 面子数別の最大搭子数を事前計算し、$O(1)$ で提供します。

use std::sync::OnceLock;

/// 各スーツ（9牌、各0〜4枚）の牌カウントパターン総数: 5^9 = 1,953,125
pub const SUIT_PATTERN_COUNT: usize = 1_953_125;

/// スーツ内の面子数 0..=4 に対する最大搭子数テーブル
/// 値が -1 の場合はその面子数が物理的に作成不可能であることを表す
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SuitEntry {
    /// 雀頭なしの場合の各面子数 m (0..=4) に対する最大搭子数
    pub no_head: [i8; 5],
    /// 雀頭ありの場合の各面子数 m (0..=4) に対する最大搭子数
    pub with_head: [i8; 5],
}

impl Default for SuitEntry {
    fn default() -> Self {
        Self {
            no_head: [-1; 5],
            with_head: [-1; 5],
        }
    }
}

/// 9要素の牌カウントスライスから 5進数（Base-5）エンコーディングキーを算出します。
/// キー範囲: `0 <= key < 1,953,125`
#[inline(always)]
pub fn encode_suit_key(counts: &[u8]) -> usize {
    let mut key = 0;
    let mut mult = 1;
    for &c in counts {
        key += (c as usize) * mult;
        mult *= 5;
    }
    key
}

static SUIT_TABLE: OnceLock<Vec<SuitEntry>> = OnceLock::new();

/// ルックアップテーブルの参照を取得します（初回アクセス時に並列生成してキャッシュ）。
pub fn get_suit_table() -> &'static [SuitEntry] {
    SUIT_TABLE.get_or_init(generate_suit_table)
}

/// ルックアップテーブルの先行ウォームアップ（必要に応じて起動時に別スレッドで呼出可能）。
pub fn warmup_suit_table() {
    let _ = get_suit_table();
}

/// スーツ単体の深さ優先探索（テーブル生成用）
fn search_suit(
    counts: &mut [u8; 9],
    idx: usize,
    melds: usize,
    taatsu: usize,
    max_taatsu: &mut [i8; 5],
) {
    let mut next_idx = idx;
    while next_idx < 9 && counts[next_idx] == 0 {
        next_idx += 1;
    }
    if next_idx >= 9 {
        if melds <= 4 && (taatsu as i8) > max_taatsu[melds] {
            max_taatsu[melds] = taatsu as i8;
        }
        return;
    }

    let i = next_idx;

    // 1. 刻子 (AAA)
    if counts[i] >= 3 {
        counts[i] -= 3;
        search_suit(counts, i, melds + 1, taatsu, max_taatsu);
        counts[i] += 3;
    }

    // 2. 順子 (ABC)
    if i <= 6 && counts[i + 1] > 0 && counts[i + 2] > 0 {
        counts[i] -= 1;
        counts[i + 1] -= 1;
        counts[i + 2] -= 1;
        search_suit(counts, i, melds + 1, taatsu, max_taatsu);
        counts[i] += 1;
        counts[i + 1] += 1;
        counts[i + 2] += 1;
    }

    // 3. 対子 (AA)
    if counts[i] >= 2 {
        counts[i] -= 2;
        search_suit(counts, i, melds, taatsu + 1, max_taatsu);
        counts[i] += 2;
    }

    // 4. 両面・辺張搭子 (AB)
    if i <= 7 && counts[i + 1] > 0 {
        counts[i] -= 1;
        counts[i + 1] -= 1;
        search_suit(counts, i, melds, taatsu + 1, max_taatsu);
        counts[i] += 1;
        counts[i + 1] += 1;
    }

    // 5. 嵌張搭子 (AC)
    if i <= 6 && counts[i + 2] > 0 {
        counts[i] -= 1;
        counts[i + 2] -= 1;
        search_suit(counts, i, melds, taatsu + 1, max_taatsu);
        counts[i] += 1;
        counts[i + 2] += 1;
    }

    // 6. スキップ（孤立牌）
    search_suit(counts, i + 1, melds, taatsu, max_taatsu);
}

/// 牌カウントパターンから SuitEntry を算出
fn compute_suit_entry(counts: &mut [u8; 9]) -> SuitEntry {
    let mut no_head = [-1i8; 5];
    search_suit(counts, 0, 0, 0, &mut no_head);

    let mut with_head = [-1i8; 5];
    for i in 0..9 {
        if counts[i] >= 2 {
            counts[i] -= 2;
            let mut temp = [-1i8; 5];
            search_suit(counts, 0, 0, 0, &mut temp);
            for m in 0..5 {
                if temp[m] > with_head[m] {
                    with_head[m] = temp[m];
                }
            }
            counts[i] += 2;
        }
    }

    SuitEntry { no_head, with_head }
}

/// 全スーツパターンのルックアップテーブルを生成（std::thread::scope による並列処理）
fn generate_suit_table() -> Vec<SuitEntry> {
    let mut table = vec![SuitEntry::default(); SUIT_PATTERN_COUNT];

    // 最上位桁 c8 (5^8 = 390,625) の 5分割で並列処理
    let chunk_size = SUIT_PATTERN_COUNT / 5;
    let chunks: Vec<&mut [SuitEntry]> = table.chunks_mut(chunk_size).collect();

    std::thread::scope(|s| {
        for (c8, chunk) in chunks.into_iter().enumerate() {
            s.spawn(move || {
                let mut counts = [0u8; 9];
                counts[8] = c8 as u8;

                // c0..c7 を再帰で生成
                fn rec(
                    idx: usize,
                    sum: u8,
                    local_key: usize,
                    counts: &mut [u8; 9],
                    chunk: &mut [SuitEntry],
                ) {
                    if idx == 8 {
                        chunk[local_key] = compute_suit_entry(counts);
                        return;
                    }
                    let mut mult = 1;
                    for _ in 0..idx {
                        mult *= 5;
                    }
                    for c in 0..=4 {
                        if sum + c <= 14 {
                            counts[idx] = c;
                            rec(
                                idx + 1,
                                sum + c,
                                local_key + (c as usize) * mult,
                                counts,
                                chunk,
                            );
                        }
                    }
                }

                rec(0, c8 as u8, 0, &mut counts, chunk);
            });
        }
    });

    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_suit_key() {
        let counts = [0u8; 9];
        assert_eq!(encode_suit_key(&counts), 0);

        let mut counts2 = [0u8; 9];
        counts2[0] = 1;
        assert_eq!(encode_suit_key(&counts2), 1);

        counts2[1] = 1;
        assert_eq!(encode_suit_key(&counts2), 1 + 5);

        let max_counts = [4u8; 9];
        assert_eq!(encode_suit_key(&max_counts), SUIT_PATTERN_COUNT - 1);
    }

    #[test]
    fn test_suit_entry_pure_sequence() {
        let table = get_suit_table();
        // 123m (1m:1, 2m:1, 3m:1)
        let mut counts = [0u8; 9];
        counts[0] = 1;
        counts[1] = 1;
        counts[2] = 1;
        let entry = table[encode_suit_key(&counts)];
        // 雀頭なし: m=1 のとき t=0, m=0 のとき t=1 (12m/23m 等の搭子)
        assert_eq!(entry.no_head[1], 0);
        assert_eq!(entry.no_head[0], 1);
        // 雀頭あり: 対子がないので作れない
        assert_eq!(entry.with_head[0], -1);
    }

    #[test]
    fn test_suit_entry_triplet_and_pair() {
        let table = get_suit_table();
        // 11122m (1m:3, 2m:2)
        let mut counts = [0u8; 9];
        counts[0] = 3;
        counts[1] = 2;
        let entry = table[encode_suit_key(&counts)];
        // 雀頭なし: 111m(刻子1) + 22m(対子1) -> m=1, t=1
        assert_eq!(entry.no_head[1], 1);
        // 雀頭あり: 22mを雀頭固定 -> 111m(刻子1) -> m=1, t=0
        assert_eq!(entry.with_head[1], 0);
    }
}
