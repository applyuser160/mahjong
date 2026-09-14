use crate::expectation::{AnalysisContext, CandidateEvaluation};
use crate::hand::Hand;
use crate::tile::TileName;

/// 順位点ルール設定（ウマ・オカ・原点・返し点）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuleConfig {
    pub origin_score: i32, // 配給原点 (例: 25,000点)
    pub return_score: i32, // 返し点 (例: 30,000点)
    pub uma: [i32; 4],     // ウマ [1位, 2位, 3位, 4位] (例: [+50, +10, -10, -30])
    pub oka: i32,          // オカ (例: 20)
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self::mleague()
    }
}

impl RuleConfig {
    /// Mリーグ基準 (+50 / +10 / -10 / -30, 25,000点持ち 30,000点返し, オカ20ptはウマ[+50]に内包)
    pub fn mleague() -> Self {
        Self {
            origin_score: 25000,
            return_score: 30000,
            uma: [50, 10, -10, -30],
            oka: 0,
        }
    }

    /// 一般的なウマ (+20 / +10 / -10 / -20, 25,000点持ち 30,000点返し, オカ +20)
    pub fn general() -> Self {
        Self {
            origin_score: 25000,
            return_score: 30000,
            uma: [20, 10, -10, -20],
            oka: 20,
        }
    }

    /// 天鳳風完全順位戦 (+45 / +0 / -45 / -90 等ラス回避重視)
    pub fn tenhou_dan() -> Self {
        Self {
            origin_score: 25000,
            return_score: 30000,
            uma: [45, 0, -45, -90],
            oka: 20,
        }
    }

    /// 最終素点と着順（1〜4）から総合ポイント（pt）を算出
    pub fn calculate_point(&self, rank: usize, final_score: i32) -> f64 {
        let base_pt = (final_score - self.return_score) as f64 / 1000.0;
        let r_idx = rank.clamp(1, 4) - 1;
        let uma_pt = self.uma[r_idx] as f64;
        let oka_pt = if rank == 1 { self.oka as f64 } else { 0.0 };
        base_pt + uma_pt + oka_pt
    }
}

/// 対局の点況コンテキスト
#[derive(Debug, Clone, PartialEq)]
pub struct MatchContext {
    pub scores: [i32; 4],     // 4家の持ち点 (0: 自家, 1: 下家, 2: 対面, 3: 上家)
    pub round_wind: TileName, // 東: East, 南: South
    pub round_number: u8,     // 局数 (1..=4)
    pub honba: u8,            // 本場
    pub riichi_sticks: u8,    // 供託リーチ棒
    pub dealer_idx: usize,    // 親のインデックス (0..4)
    pub rule: RuleConfig,
}

impl Default for MatchContext {
    fn default() -> Self {
        Self {
            scores: [25000, 25000, 25000, 25000],
            round_wind: TileName::East,
            round_number: 1,
            honba: 0,
            riichi_sticks: 0,
            dealer_idx: 0,
            rule: RuleConfig::default(),
        }
    }
}

impl MatchContext {
    /// 各プレイヤーの現在着順 (1..=4) を取得（同点時は起家優先席順 0 > 1 > 2 > 3）
    pub fn current_ranks(&self) -> [usize; 4] {
        let mut players = [0, 1, 2, 3];
        players.sort_by(|&a, &b| self.scores[b].cmp(&self.scores[a]).then_with(|| a.cmp(&b)));
        let mut ranks = [0; 4];
        for (r, &p) in players.iter().enumerate() {
            ranks[p] = r + 1;
        }
        ranks
    }

    /// プレイヤー p から見た相手 target との点差 (相手の持ち点 - 自分の持ち点)
    pub fn score_diff(&self, p: usize, target: usize) -> i32 {
        self.scores[target] - self.scores[p]
    }

    /// オーラス（南4局）かどうか
    pub fn is_orasu(&self) -> bool {
        self.round_wind == TileName::South && self.round_number == 4
    }

    /// 残り局数の概算
    pub fn remaining_rounds(&self) -> usize {
        let current_index = match self.round_wind {
            TileName::East => self.round_number as usize,
            TileName::South => 4 + self.round_number as usize,
            _ => 8,
        };
        8usize.saturating_sub(current_index).max(1)
    }
}

/// オーラス等における上位着順への逆転条件
#[derive(Debug, Clone, PartialEq)]
pub struct WinCondition {
    pub target_rank: usize,          // 目指す着順 (例: 1位, 2位)
    pub target_player: usize,        // 捲る対象のプレイヤー
    pub diff: i32,                   // 点差 (相手 - 自分)
    pub ron_direct_req: Option<i32>, // 直撃時の必要和了素点
    pub tsumo_req: Option<i32>,      // ツモ和了時の必要和了素点（合計打点）
    pub ron_other_req: Option<i32>,  // 脇出和了時の必要和了素点
    pub summary: String,             // 日本語解説サマリー
}

/// オーラスでの自家（player_idx）の逆転条件を計算
pub fn calculate_orasu_conditions(ctx: &MatchContext, player_idx: usize) -> Vec<WinCondition> {
    let ranks = ctx.current_ranks();
    let my_rank = ranks[player_idx];
    let mut conditions = Vec::new();

    if my_rank == 1 {
        // トップ目の場合: 2位との点差と、2位のツモ/直撃に耐えられるリード状況をサマリー
        let second_player = (0..4).find(|&p| ranks[p] == 2).unwrap_or(1);
        let lead = ctx.scores[player_idx] - ctx.scores[second_player];
        conditions.push(WinCondition {
            target_rank: 1,
            target_player: second_player,
            diff: -lead,
            ron_direct_req: None,
            tsumo_req: None,
            ron_other_req: None,
            summary: format!(
                "👑 現在トップ目！ 2位と {}点差のリード（局消化・守備優先）",
                lead
            ),
        });
        return conditions;
    }

    let is_dealer = player_idx == ctx.dealer_idx;
    let honba_bonus = (ctx.honba as i32) * 300;
    let kyotaku_bonus = (ctx.riichi_sticks as i32) * 1000;

    // 自身より上位の各プレイヤーについて条件を計算
    for target_rank in 1..my_rank {
        if let Some(target_p) = (0..4).find(|&p| ranks[p] == target_rank) {
            let diff = ctx.scores[target_p] - ctx.scores[player_idx];
            // 同点時に自家が上流（起家に近い）なら同点で逆転可能、下流なら1点以上上回る必要あり
            let tie_advantage = player_idx < target_p;
            let need_diff = if tie_advantage { diff } else { diff + 1 };

            // 1. 直撃 (ターゲットからの出和了)
            // 自家加点: X + honba_bonus + kyotaku_bonus
            // 相手失点: X + honba_bonus
            // 総差詰まり: 2X + 2*honba_bonus + kyotaku_bonus >= need_diff
            let ron_direct = find_min_standard_score(
                |x| 2 * x + 2 * honba_bonus + kyotaku_bonus >= need_diff,
                is_dealer,
                false,
            );

            // 2. ツモ和了
            // 自家加点: X_total + honba_bonus + kyotaku_bonus
            // ターゲット相手の支払い:
            //   ターゲットが親: X_dealer_pay + 100*honba
            //   ターゲットが子: X_child_pay + 100*honba
            let target_is_dealer = target_p == ctx.dealer_idx;
            let tsumo = find_min_standard_score(
                |x| {
                    let target_pay = estimate_tsumo_target_payment(x, is_dealer, target_is_dealer);
                    let total_gain =
                        x + honba_bonus + kyotaku_bonus + target_pay + (ctx.honba as i32) * 100;
                    total_gain >= need_diff
                },
                is_dealer,
                true,
            );

            // 3. 脇出和了 (ターゲット以外からのロン)
            // 自家加点: X + honba_bonus + kyotaku_bonus
            let ron_other = find_min_standard_score(
                |x| x + honba_bonus + kyotaku_bonus >= need_diff,
                is_dealer,
                false,
            );

            let summary = format!(
                "{}位への逆転条件 (点差: {}点) -> 直撃: {} | ツモ: {} | 脇: {}",
                target_rank,
                diff,
                format_score_requirement(ron_direct),
                format_score_requirement(tsumo),
                format_score_requirement(ron_other)
            );

            conditions.push(WinCondition {
                target_rank,
                target_player: target_p,
                diff,
                ron_direct_req: ron_direct,
                tsumo_req: tsumo,
                ron_other_req: ron_other,
                summary,
            });
        }
    }

    conditions
}

/// 標準的な和了打点（1000〜32000点）から条件を満たす最小の打点を探索
fn find_min_standard_score<F>(predicate: F, is_dealer: bool, is_tsumo: bool) -> Option<i32>
where
    F: Fn(i32) -> bool,
{
    let candidates: &[i32] = if is_dealer {
        &[
            1500, 2000, 2400, 2900, 3900, 4800, 5800, 7700, 9600, 11600, 12000, 18000, 24000,
            36000, 48000,
        ]
    } else if is_tsumo {
        &[
            1000, 1500, 2000, 2600, 2700, 3900, 5200, 8000, 12000, 16000, 24000, 32000,
        ]
    } else {
        &[
            1000, 1300, 1600, 2000, 2600, 3200, 3900, 5200, 6400, 7700, 8000, 12000, 16000, 24000,
            32000,
        ]
    };

    candidates.iter().find(|&&score| predicate(score)).copied()
}

/// ツモ和了時のターゲット他家の支払い額概算
fn estimate_tsumo_target_payment(total_score: i32, is_dealer: bool, target_is_dealer: bool) -> i32 {
    if is_dealer {
        // 親のツモ: 子3名が均等（1/3）
        total_score / 3
    } else if target_is_dealer {
        // 子のツモでターゲットが親: 親は約半額（2/4）
        total_score / 2
    } else {
        // 子のツモでターゲットが子: 子は約1/4
        total_score / 4
    }
}

/// 打点要件の日本語表示フォーマット
fn format_score_requirement(req: Option<i32>) -> String {
    match req {
        Some(s) if s >= 32000 => "役満".to_string(),
        Some(s) if s >= 24000 => "三倍満".to_string(),
        Some(s) if s >= 16000 => "倍満".to_string(),
        Some(s) if s >= 12000 => "跳満".to_string(),
        Some(s) if s >= 8000 => "満貫".to_string(),
        Some(s) => format!("{}点", s),
        None => "条件なし(不可)".to_string(),
    }
}

/// 順位EVを考慮した打牌候補の拡張評価
#[derive(Debug, Clone, PartialEq)]
pub struct PlacementCandidateEvaluation {
    pub base: CandidateEvaluation,
    pub placement_ev: f64,  // 順位点（ウマ・オカ・素点）を含めた総合EV (pt)
    pub expected_rank: f64, // 期待着順 (1.0〜4.0)
    pub rank_probabilities: [f64; 4], // [1位率, 2位率, 3位率, 4位率]
    pub situational_note: String, // 点況に応じたAIの戦術解説
}

/// 素点評価に点況（MatchContext）を統合し、順位期待値（Placement EV）を算出
pub fn evaluate_hand_discards_with_placement(
    hand: &Hand,
    visible_counts: Option<&[u8; 35]>,
    analysis_ctx: &AnalysisContext,
    match_ctx: &MatchContext,
    player_idx: usize,
) -> Vec<PlacementCandidateEvaluation> {
    let raw_evaluations =
        crate::expectation::evaluate_hand_discards(hand, visible_counts, analysis_ctx);
    let ranks = match_ctx.current_ranks();
    let current_rank = ranks[player_idx];
    let is_orasu = match_ctx.is_orasu();

    // 放銃シナリオにおける想定失点額を動的に推定 (Issue #79: 親12000点 vs 子8000点、本場・ドラ補正)
    let deal_loss = estimate_expected_deal_loss(match_ctx, analysis_ctx, player_idx);

    let mut placement_evals = Vec::new();

    for ev in raw_evaluations {
        let win_p = ev.speed.win_probability;
        let deal_p = ev.safety.risk_score * 0.4; // 簡易放銃確率
        let other_p = (1.0 - win_p - deal_p).max(0.0);

        // 各シナリオにおける局後スコアの予測
        let score_on_win = match_ctx.scores[player_idx] + ev.value.expected_score as i32;
        let score_on_deal = match_ctx.scores[player_idx] - deal_loss;
        let score_on_other = match_ctx.scores[player_idx];

        // 各シナリオでの着順確率分布をシミュレーション
        let probs_win = estimate_rank_probabilities(match_ctx, player_idx, score_on_win);
        let probs_deal = estimate_rank_probabilities(match_ctx, player_idx, score_on_deal);
        let probs_other = estimate_rank_probabilities(match_ctx, player_idx, score_on_other);

        let mut final_probs = [0.0; 4];
        for r in 0..4 {
            final_probs[r] =
                win_p * probs_win[r] + deal_p * probs_deal[r] + other_p * probs_other[r];
        }

        // 期待順位Pt (Placement EV) の算出
        let mut pt_ev = 0.0;
        let mut exp_rank = 0.0;
        for (r_idx, &p) in final_probs.iter().enumerate() {
            let rank = r_idx + 1;
            let est_final_score = match_ctx.scores[player_idx] + ev.ev as i32;
            let pt = match_ctx.rule.calculate_point(rank, est_final_score);
            pt_ev += p * pt;
            exp_rank += p * (rank as f64);
        }

        // 点況戦術解説文
        let note =
            generate_situational_note(current_rank, is_orasu, match_ctx.scores[player_idx], &ev);

        placement_evals.push(PlacementCandidateEvaluation {
            base: ev,
            placement_ev: pt_ev,
            expected_rank: exp_rank,
            rank_probabilities: final_probs,
            situational_note: note,
        });
    }

    // 順位EV降順でソート
    placement_evals.sort_by(|a, b| {
        b.placement_ev
            .partial_cmp(&a.placement_ev)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    placement_evals
}

/// 局後スコアから着順確率（1位〜4位）をロジスティック近似で推定
fn estimate_rank_probabilities(
    ctx: &MatchContext,
    player_idx: usize,
    my_predicted_score: i32,
) -> [f64; 4] {
    let mut other_scores = Vec::new();
    for p in 0..4 {
        if p != player_idx {
            other_scores.push(ctx.scores[p]);
        }
    }
    other_scores.sort_by(|a, b| b.cmp(a)); // 降順

    // 点差に応じた他家との勝率 (ロジスティック関数)
    // 10,000点差で約88%勝率、0点差で50%
    let win_vs = |s_my: i32, s_other: i32| -> f64 {
        let x = (s_my - s_other) as f64 / 5000.0;
        0.5 * (x / (1.0 + x.abs())) + 0.5
    };

    let p_vs_0 = win_vs(my_predicted_score, other_scores[0]);
    let p_vs_1 = win_vs(my_predicted_score, other_scores[1]);
    let p_vs_2 = win_vs(my_predicted_score, other_scores[2]);

    // 1位率: 3人全員に勝つ
    let p_1st = (p_vs_0 * p_vs_1 * p_vs_2).clamp(0.01, 0.97);
    // 4位率: 3人全員に負ける
    let p_4th = ((1.0 - p_vs_0) * (1.0 - p_vs_1) * (1.0 - p_vs_2)).clamp(0.01, 0.97);
    // 中間（2位・3位）を按分
    let remaining = (1.0 - p_1st - p_4th).max(0.02);
    let p_2nd = remaining * (p_vs_1 / (p_vs_1 + (1.0 - p_vs_1)).max(0.01));
    let p_3rd = remaining - p_2nd;

    [p_1st, p_2nd, p_3rd, p_4th]
}

/// 点況に応じたAI戦術ノートの生成
fn generate_situational_note(
    current_rank: usize,
    is_orasu: bool,
    my_score: i32,
    ev: &CandidateEvaluation,
) -> String {
    if is_orasu {
        match current_rank {
            1 => "【オーラス・トップ目】放銃による着落ちが最大の敗因となります。安手でもアガリきりによる逃げ切り、または完全現物による守備が絶対正義です。".to_string(),
            4 => "【オーラス・ラス目】安手のアガリでは着順が浮上しません。逆転条件を満たす打点作り（役牌・ドラ・染め手・リーチ）に特化してください。".to_string(),
            _ => "【オーラス・着順浮上戦】直撃・ツモ条件を意識しつつ、下位からの逆転放銃を厳戒してください。".to_string(),
        }
    } else if current_rank == 1 && my_score >= 35000 {
        if ev.safety.risk_score == 0.0 {
            "【トップ目防衛】大きなリードを持っています。危険牌を抱え込まず、速度と安全度を最優先して局を消化します。".to_string()
        } else {
            "【トップ目押し引き】失点リスクを極力抑えたい場面です。他家の動向に注意してください。"
                .to_string()
        }
    } else if current_rank == 4 && my_score <= 15000 {
        "【ラス目挽回】点差が開いているため、満貫以上の高打点ルートを強く意識します。".to_string()
    } else {
        "【通常進行】素点効率とスピードのバランスを保ちつつ手を進めます。".to_string()
    }
}

/// 放銃シナリオにおける想定失点額を計算 (Issue #79)
pub fn estimate_expected_deal_loss(
    match_ctx: &MatchContext,
    analysis_ctx: &AnalysisContext,
    player_idx: usize,
) -> i32 {
    // 1. 全リーチ者を走査し、親リーチが含まれているかを判定（最大失点リスク優先）
    let riichi_opponents: Vec<usize> = (0..4)
        .filter(|&p| p != player_idx && analysis_ctx.riichi_status[p])
        .collect();

    let has_any_riichi = !riichi_opponents.is_empty();
    let has_dealer_riichi = riichi_opponents
        .iter()
        .any(|&p| p == match_ctx.dealer_idx || analysis_ctx.player_is_dealer[p]);

    let honba_pts = (match_ctx.honba as i32) * 300;

    let base_points = if has_dealer_riichi {
        12000 // 親リーチ: 12000点最優先
    } else if has_any_riichi {
        8000 // 子リーチ: 8000点
    } else if match_ctx.dealer_idx != player_idx {
        9600 // ノーリーチ時: 自家が親でなければ親の平時・副露警戒
    } else {
        5200 // ノーリーチ時: 自家が親なら子の平時・副露警戒
    };

    base_points + honba_pts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_expected_deal_loss_dealer_vs_child() {
        // Issue #79 検証: 親リーチへの放銃想定失点 vs 子リーチへの放銃想定失点
        let match_ctx = MatchContext {
            dealer_idx: 1, // 下家が親
            honba: 1,      // 1本場 (+300点)
            ..Default::default()
        };

        // 1. 親リーチに対する放銃
        let mut ctx_dealer_riichi = AnalysisContext::default();
        ctx_dealer_riichi.riichi_status[1] = true;
        let loss_dealer = estimate_expected_deal_loss(&match_ctx, &ctx_dealer_riichi, 0);
        // 12000 + 300 = 12300点
        assert_eq!(loss_dealer, 12300);

        // 2. 子リーチに対する放銃
        let mut ctx_child_riichi = AnalysisContext::default();
        ctx_child_riichi.riichi_status[2] = true; // 対面（子）がリーチ
        let loss_child = estimate_expected_deal_loss(&match_ctx, &ctx_child_riichi, 0);
        // 8000 + 300 = 8300点
        assert_eq!(loss_child, 8300);

        assert!(
            loss_dealer > loss_child,
            "Dealer deal loss ({}) must be significantly larger than child ({})",
            loss_dealer,
            loss_child
        );
    }

    #[test]
    fn test_estimate_expected_deal_loss_multiple_riichi_dealer_priority() {
        // Issue #79 レビュー対応: 子（player 1）と親（player 2）の両方がリーチしている場合
        // インデックス順（.find()）に引きずられず、親リーチ（12,000点）が最優先されること
        let match_ctx = MatchContext {
            dealer_idx: 2, // 対面が親
            honba: 1,      // 1本場 (+300点)
            ..Default::default()
        };

        let mut ctx_multi_riichi = AnalysisContext::default();
        ctx_multi_riichi.riichi_status[1] = true; // 下家（子）リーチ
        ctx_multi_riichi.riichi_status[2] = true; // 対面（親）リーチ

        let loss = estimate_expected_deal_loss(&match_ctx, &ctx_multi_riichi, 0);
        // 親リーチが優先されて 12000 + 300 = 12300点
        assert_eq!(
            loss, 12300,
            "複数リーチ時は親リーチの失点リスク（12,000点+本場）が最優先されるべき"
        );
    }

    #[test]
    fn test_estimate_expected_deal_loss_dora_no_reduction() {
        // Issue #79 レビュー対応: ドラ表示牌が3枚以上あっても失点が減衰されないこと
        let match_ctx = MatchContext {
            dealer_idx: 1,
            honba: 0,
            ..Default::default()
        };

        let doras = [
            crate::tile::TileName::OneM,
            crate::tile::TileName::TwoM,
            crate::tile::TileName::ThreeM,
        ];
        let mut riichi_status = [false; 4];
        riichi_status[1] = true; // 親リーチ
        let ctx = AnalysisContext {
            dora_indicators: &doras,
            riichi_status,
            ..Default::default()
        };

        let loss = estimate_expected_deal_loss(&match_ctx, &ctx, 0);
        assert_eq!(
            loss, 12000,
            "ドラ表示牌が3枚以上あっても減衰されず親満貫12,000点であるべき"
        );
    }
}
