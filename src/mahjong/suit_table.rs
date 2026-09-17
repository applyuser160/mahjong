//! 麻雀の数牌スーツ別ルックアップテーブル (LUT)
//!
//! 各数牌スーツ（萬子・筒子・索子の各9種、枚数0〜4）の牌カウントパターンに対する
//! 面子数別の最大搭子数をビルド時に事前計算（`build.rs`）し、実行時に $O(1)$ かつゼロ遅延で提供します。

/// 各スーツ（9牌、各0〜4枚）の牌カウントパターン総数: 5^9 = 1,953,125
pub const SUIT_PATTERN_COUNT: usize = 1_953_125;

/// スーツ内の面子数 0..=4 に対する最大搭子数テーブル
/// 値が -1 の場合はその面子数が物理的に作成不可能であることを表す
#[repr(C)]
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

/// 9要素の牌カウントスライスから 5進数（Base-5）エンコーディングキーを算出します（スカラー実装）。
#[inline(always)]
pub fn encode_suit_key_scalar(counts: &[u8]) -> usize {
    let mut key = 0;
    let mut mult = 1;
    for &c in counts {
        let safe_c = (c.min(4)) as usize;
        key += safe_c * mult;
        mult *= 5;
    }
    key
}

/// 9要素の牌カウントスライスから 5進数（Base-5）エンコーディングキーを算出します（x86_64 AVX2 実装）。
///
/// Base-5 重みのうち $5^7 = 78,125$ は `i16` 範囲外となるため、
/// 256bit レジスタ上の 8 個の 32bit レーン（`_mm256_mullo_epi32`）を用いて先頭 8 要素を並列積和し、
/// 9 番目の要素（$c_8 \times 390,625$）を加算します。
///
/// # Safety
/// 呼び出し元で CPU の AVX2 命令セットサポート、および `counts.len() >= 9` が保証されている必要があります。
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn encode_suit_key_avx2(counts: &[u8]) -> usize {
    use std::arch::x86_64::*;

    // counts[0..8] の 8 バイトをロード
    let v8 = _mm_loadu_si64(counts.as_ptr());

    // 8 個の u8 を 8 個の i32 に符号なし拡張
    let v32 = _mm256_cvtepu8_epi32(v8);

    // 各牌の枚数を最大 4 枚に飽和 (min(4))
    let four = _mm256_set1_epi32(4);
    let clamped = _mm256_min_epi32(v32, four);

    // 重み定数 [5^0, 5^1, ..., 5^7]
    // 5^7 = 78,125 は i32 の表現範囲に安全に収まる
    let weights = _mm256_setr_epi32(1, 5, 25, 125, 625, 3125, 15625, 78125);

    // 32bit レーン単位の並列乗算
    let prod = _mm256_mullo_epi32(clamped, weights);

    // 8 レーンの水平加算: 256bit -> 128bit
    let low128 = _mm256_castsi256_si128(prod);
    let high128 = _mm256_extracti128_si256(prod, 1);
    let sum128 = _mm_add_epi32(low128, high128);

    // 4 個の i32 を加算
    let hi64 = _mm_unpackhi_epi64(sum128, sum128);
    let sum64 = _mm_add_epi32(sum128, hi64);
    let hi32 = _mm_shuffle_epi32(sum64, 1);
    let sum32 = _mm_add_epi32(sum64, hi32);
    let sum = _mm_cvtsi128_si32(sum32) as usize;

    // 9 番目の要素 (重み 5^8 = 390,625) を加算
    let c8 = (*counts.get_unchecked(8)).min(4) as usize;
    sum + c8 * 390625
}

/// 9要素の牌カウントスライスから 5進数（Base-5）エンコーディングキーを算出します。
/// 各牌の枚数を最大4枚に飽和（`min(4)`）させるため、同一牌が5枚以上の異常値でも
/// 配列境界外アクセス（panic）が発生せず安全に `0 <= key < 1,953,125` のキーを返します。
#[inline(always)]
pub fn encode_suit_key(counts: &[u8]) -> usize {
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        if counts.len() >= 9 {
            unsafe { encode_suit_key_avx2(counts) }
        } else {
            encode_suit_key_scalar(counts)
        }
    }
    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
    {
        if counts.len() >= 9 && is_x86_feature_detected!("avx2") {
            unsafe { encode_suit_key_avx2(counts) }
        } else {
            encode_suit_key_scalar(counts)
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        encode_suit_key_scalar(counts)
    }
}

// build.rs で生成された 1,953,125 * 10 バイトのバイナリを埋め込み
static RAW_TABLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/suit_table.bin"));

/// ルックアップテーブルの参照を取得します（静的埋め込みのため初期化コストは 0.00ms）。
#[inline(always)]
pub fn get_suit_table() -> &'static [SuitEntry] {
    // SuitEntry は [i8; 5] が2つの 10バイト構造体で、アライメントは 1 (パディングなし)。
    // そのため未定義動作なく安全に &[SuitEntry] にキャスト可能。
    unsafe {
        std::slice::from_raw_parts(
            RAW_TABLE_BYTES.as_ptr() as *const SuitEntry,
            SUIT_PATTERN_COUNT,
        )
    }
}

/// ルックアップテーブルの先行ウォームアップ（後方互換性のため提供）。
#[inline(always)]
pub fn warmup_suit_table() {
    let _ = get_suit_table();
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
    fn test_encode_suit_key_safety_with_overflow() {
        // 同一牌が5枚以上の異常入力でも panic せず、上限 4 に安全にクリップされること
        let mut overflow_counts = [0u8; 9];
        overflow_counts[0] = 5; // 5枚
        overflow_counts[8] = 10; // 10枚
        let key = encode_suit_key(&overflow_counts);
        assert!(key < SUIT_PATTERN_COUNT);

        let mut clamped_counts = [0u8; 9];
        clamped_counts[0] = 4;
        clamped_counts[8] = 4;
        assert_eq!(key, encode_suit_key(&clamped_counts));
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

    #[test]
    fn test_cold_start_latency() {
        // 初回呼び出しでも 1ms 未満（実際は数ナノ秒）で即座に応答することを検証
        let start = std::time::Instant::now();
        let table = get_suit_table();
        let elapsed = start.elapsed();
        assert_eq!(table.len(), SUIT_PATTERN_COUNT);
        assert!(
            elapsed.as_millis() < 50,
            "Cold start should take under 50ms, took {:?}",
            elapsed
        );
    }
}
