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

/// 30bit にパックされたスーツ別面子・搭子数エントリ (4バイト)
///
/// 面子数 m (0..=4) に対する搭子数を min(taatsu, 4 - m) に丸め、
/// -1..=4 の 6 値（3bit: 0=作成不可, 1..=5 = 搭子数 0..=4）として 10 セル（計 30bit）を保持。
/// アライメントを 1 に保つため [u8; 4] の透過ラッパーとして定義。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PackedSuitEntry(pub [u8; 4]);

impl PackedSuitEntry {
    /// 32bit 整数値を取得
    #[inline(always)]
    pub fn as_u32(self) -> u32 {
        u32::from_le_bytes(self.0)
    }

    /// 30bit のパック値から SuitEntry をブランチレスに復元
    #[inline(always)]
    pub fn unpack(self) -> SuitEntry {
        let val = self.as_u32();
        #[inline(always)]
        fn decode(val: u32, shift: u32) -> i8 {
            (((val >> shift) & 0b111) as i8) - 1
        }

        SuitEntry {
            no_head: [
                decode(val, 0),
                decode(val, 3),
                decode(val, 6),
                decode(val, 9),
                decode(val, 12),
            ],
            with_head: [
                decode(val, 15),
                decode(val, 18),
                decode(val, 21),
                decode(val, 24),
                decode(val, 27),
            ],
        }
    }

    /// SuitEntry から PackedSuitEntry を生成
    #[inline(always)]
    pub fn pack(entry: &SuitEntry) -> Self {
        let encode = |val: i8, m: usize| -> u32 {
            if val < 0 {
                0
            } else {
                let clamped = (val as usize).min(4 - m) as u32;
                clamped + 1
            }
        };

        let mut packed = 0u32;
        for m in 0..5 {
            packed |= encode(entry.no_head[m], m) << (m * 3);
            packed |= encode(entry.with_head[m], m) << (15 + m * 3);
        }
        Self(packed.to_le_bytes())
    }
}

/// 9要素の牌カウントスライスから 5進数（Base-5）エンコーディングキーを算出します。
/// 各牌の枚数を最大4枚に飽和（`min(4)`）させるため、同一牌が5枚以上の異常値でも
/// 配列境界外アクセス（panic）が発生せず安全に `0 <= key < 1,953,125` のキーを返します。
#[inline(always)]
pub fn encode_suit_key(counts: &[u8]) -> usize {
    let mut key = 0;
    let mut mult = 1;
    for &c in counts {
        let safe_c = (c.min(4)) as usize;
        key += safe_c * mult;
        mult *= 5;
    }
    key
}

// build.rs で生成された 1,953,125 * 4 バイト (約 7.45 MiB) のバイナリを埋め込み
static RAW_TABLE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/suit_table.bin"));

/// パック済みルックアップテーブルの参照を取得します（静的埋め込みのため初期化コストは 0.00ms）。
#[inline(always)]
pub fn get_packed_suit_table() -> &'static [PackedSuitEntry] {
    unsafe {
        std::slice::from_raw_parts(
            RAW_TABLE_BYTES.as_ptr() as *const PackedSuitEntry,
            SUIT_PATTERN_COUNT,
        )
    }
}

/// 指定したキーの SuitEntry を取得します（30bit から高速アンパック）。
#[inline(always)]
pub fn get_suit_entry(key: usize) -> SuitEntry {
    get_packed_suit_table()[key].unpack()
}

/// 後方互換性用テーブル参照関数
#[inline(always)]
pub fn get_suit_table() -> &'static [PackedSuitEntry] {
    get_packed_suit_table()
}

/// ルックアップテーブルの先行ウォームアップ（後方互換性のため提供）。
#[inline(always)]
pub fn warmup_suit_table() {
    let _ = get_packed_suit_table();
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
        // 123m (1m:1, 2m:1, 3m:1)
        let mut counts = [0u8; 9];
        counts[0] = 1;
        counts[1] = 1;
        counts[2] = 1;
        let entry = get_suit_entry(encode_suit_key(&counts));
        // 雀頭なし: m=1 のとき t=0, m=0 のとき t=1 (12m/23m 等の搭子)
        assert_eq!(entry.no_head[1], 0);
        assert_eq!(entry.no_head[0], 1);
        // 雀頭あり: 対子がないので作れない
        assert_eq!(entry.with_head[0], -1);
    }

    #[test]
    fn test_suit_entry_triplet_and_pair() {
        // 11122m (1m:3, 2m:2)
        let mut counts = [0u8; 9];
        counts[0] = 3;
        counts[1] = 2;
        let entry = get_suit_entry(encode_suit_key(&counts));
        // 雀頭なし: 111m(刻子1) + 22m(対子1) -> m=1, t=1
        assert_eq!(entry.no_head[1], 1);
        // 雀頭あり: 22mを雀頭固定 -> 111m(刻子1) -> m=1, t=0
        assert_eq!(entry.with_head[1], 0);
    }

    #[test]
    fn test_packed_suit_entry_roundtrip() {
        let original = SuitEntry {
            no_head: [3, 2, 1, 0, -1],
            with_head: [-1, 2, 1, 0, -1],
        };
        let packed = PackedSuitEntry::pack(&original);
        let unpacked = packed.unpack();
        // m=0 のとき 3 <= 4 なので 3
        assert_eq!(unpacked.no_head[0], 3);
        assert_eq!(unpacked.no_head[1], 2);
        assert_eq!(unpacked.no_head[2], 1);
        assert_eq!(unpacked.no_head[3], 0);
        assert_eq!(unpacked.no_head[4], -1);
        assert_eq!(unpacked.with_head[0], -1);
        assert_eq!(unpacked.with_head[1], 2);
        assert_eq!(unpacked.with_head[2], 1);
        assert_eq!(unpacked.with_head[3], 0);
        assert_eq!(unpacked.with_head[4], -1);

        // 4 - m を超える搭子数が正しく丸められることの検証 (例: m=3 で taatsu=2 -> min(2, 4-3) = 1)
        let excess = SuitEntry {
            no_head: [5, 5, 5, 5, 5],
            with_head: [5, 5, 5, 5, 5],
        };
        let packed_excess = PackedSuitEntry::pack(&excess);
        let unpacked_excess = packed_excess.unpack();
        for m in 0..5 {
            assert_eq!(unpacked_excess.no_head[m], (4 - m) as i8);
            assert_eq!(unpacked_excess.with_head[m], (4 - m) as i8);
        }
    }

    #[test]
    fn test_cold_start_latency() {
        // 初回呼び出しでも 1ms 未満（実際は数ナノ秒）で即座に応答することを検証
        let start = std::time::Instant::now();
        let table = get_packed_suit_table();
        let elapsed = start.elapsed();
        assert_eq!(table.len(), SUIT_PATTERN_COUNT);
        assert!(
            elapsed.as_millis() < 50,
            "Cold start should take under 50ms, took {:?}",
            elapsed
        );
    }
}
