fix(yaku): implement ryamen wait requirement for Pinfu detection

Pinfu requires the winning tile to complete a sequence with a ryamen
(two-sided) wait. Previously, the logic only checked if all melds
were sequences and the pair was not a value pair, which incorrectly
allowed kanchan (closed), penchan (edge), and nobetan (tanki) waits.

This commit updates `detect_pinfu` to strictly verify that the winning
tile acts as a two-sided completion for at least one sequence in the
hand pattern.

Additionally, `ron_tile` in `WinContext` has been renamed to `win_tile`
to uniformly represent the winning tile for both Ron and Tsumo wins,
ensuring robust wait detection. Tests have been updated and expanded
to explicitly cover ryamen, kanchan, penchan, and nobetan edge cases.
