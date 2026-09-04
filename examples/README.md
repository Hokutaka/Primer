# Primerサンプル

[English](README.en.md)

このディレクトリには、現在のPrimerで読んで実行できるプログラムを置きます。それぞれの例は、別の構文や計算方法を小さく示します。

## 基本

| サンプル | 内容 |
| --- | --- |
| [hello.prim](hello.prim) | 束縛、四則演算、`print` |
| [floating_point.prim](floating_point.prim) | `i64`、`f32`、`f64` |
| [boolean_comparisons.prim](boolean_comparisons.prim) | 真偽値と比較演算 |
| [conditional.prim](conditional.prim) | `if` / `else`とscope |
| [loop_control.prim](loop_control.prim) | `while`、`break`、`continue` |
| [for_sum.prim](for_sum.prim) | `for`と開始文の再代入 |
| [product-point.prim](product-point.prim) | 名前付きproduct type、既定値、field access |
| [functions.prim](functions.prim) | 型付き関数、parameter、戻り値、`void`関数 |

## 数値計算

| サンプル | 内容 |
| --- | --- |
| [square_root.prim](square_root.prim) | 手順を展開した平方根の近似 |
| [while_square_root.prim](while_square_root.prim) | `while`で繰り返す平方根の近似 |
| [logistic_map.prim](logistic_map.prim) | `f32`と`f64`で生まれる計算結果の違い |

## アルゴリズム

| サンプル | 内容 |
| --- | --- |
| [euclidean_gcd.prim](euclidean_gcd.prim) | ユークリッドの互除法による最大公約数 |
| [fibonacci.prim](fibonacci.prim) | Fibonacci数列と複数の値の更新順 |
| [factorial.prim](factorial.prim) | `for`による階乗 |
| [collatz.prim](collatz.prim) | Collatz予想と条件ごとの状態遷移 |
| [prime_check.prim](prime_check.prim) | 試し割りによる素数判定と早期終了 |
| [integer_square_root.prim](integer_square_root.prim) | 二分探索による整数平方根 |
| [exponentiation_by_squaring.prim](exponentiation_by_squaring.prim) | 繰り返し二乗法による累乗 |
| [pythagorean_triples.prim](pythagorean_triples.prim) | 入れ子の`for`によるピタゴラス数の探索 |

## 現在の範囲

これらは、数値、真偽値、束縛、関数、条件分岐、ループ、名前付きproduct typeで表現できるプログラムです。

現在は配列、文字列、再帰がないため、ソート、配列の探索、木やグラフの探索、一般的な動的計画法などはまだ自然に表現できません。将来それらの機能が入ったとき、対応するサンプルを追加します。
