//! ビルドスクリプト: 数牌スーツ別ルックアップテーブル (LUT) の事前生成
//!
//! コンパイル時に 1,953,125 パターンの SuitEntry テーブルを事前計算し、
//! OUT_DIR/suit_table.bin にバイナリファイルとして出力します。
//! これにより実行時・起動時の初期化コストを 0ms に抑えます。

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

const SUIT_PATTERN_COUNT: usize = 1_953_125;

#[repr(C)]
#[derive(Clone, Copy)]
struct SuitEntry {
    no_head: [i8; 5],
    with_head: [i8; 5],
}

impl Default for SuitEntry {
    fn default() -> Self {
        Self {
            no_head: [-1; 5],
            with_head: [-1; 5],
        }
    }
}

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

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("suit_table.bin");

    // すでに存在していればスキップ（サイズが正しい場合）
    let expected_len = SUIT_PATTERN_COUNT * std::mem::size_of::<SuitEntry>();
    if let Ok(metadata) = std::fs::metadata(&dest_path) {
        if metadata.len() == expected_len as u64 {
            return;
        }
    }

    let mut table = vec![SuitEntry::default(); SUIT_PATTERN_COUNT];

    // 最上位桁 c8 (5^8 = 390,625) の 5分割で並列処理
    let chunk_size = SUIT_PATTERN_COUNT / 5;
    let chunks: Vec<&mut [SuitEntry]> = table.chunks_mut(chunk_size).collect();

    std::thread::scope(|s| {
        for (c8, chunk) in chunks.into_iter().enumerate() {
            s.spawn(move || {
                let mut counts = [0u8; 9];
                counts[8] = c8 as u8;

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

    let file = File::create(&dest_path).expect("Failed to create suit_table.bin");
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);

    // バイト列として書き出し
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            table.as_ptr() as *const u8,
            SUIT_PATTERN_COUNT * std::mem::size_of::<SuitEntry>(),
        )
    };
    writer
        .write_all(bytes)
        .expect("Failed to write suit_table.bin");
    writer.flush().expect("Failed to flush suit_table.bin");
}
