# Primerサンプル

[English](README.en.md)

このディレクトリには、現在のPrimerで読んで実行できるプログラムを置きます。それぞれの例は、別の構文や計算方法を小さく示します。

## まとめて実行

リポジトリのルートから次を実行すると、すべてのサンプルについて名前、実行結果、成否、最後の集計を表示します。

```powershell
.\scripts\run-examples.ps1
```

`-Pattern "matrix*.prim"`で対象を絞れます。`-SkipBuild`を指定すると、既にbuildされているPrimerを使います。

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
| [function_values.prim](function_values.prim) | product typeと入れ子の固定長配列を関数へ値として渡す |
| [fixed_arrays.prim](fixed_arrays.prim) | 固定長配列、添字参照、配列の値コピー |
| [product_arrays.prim](product_arrays.prim) | 点の配列、product typeの添字参照、配列の値コピー |
| [matrix_vector_product.prim](matrix_vector_product.prim) | 入れ子の固定長配列と二段の添字参照 |
| [matrix_composition.prim](matrix_composition.prim) | product typeと入れ子配列を関数で受け渡す数値計算 |

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
| [fixed_arrays.prim](fixed_arrays.prim) | 配列の合計と線形探索 |
| [product_arrays.prim](product_arrays.prim) | 点の配列から最も近い点を探す線形探索 |
| [matrix_vector_product.prim](matrix_vector_product.prim) | 3×3行列と3要素vectorの積 |
| [matrix_composition.prim](matrix_composition.prim) | 2×2行列の合成と、合成した行列によるvector変換 |
| [xor_neural_network.prim](xor_neural_network.prim) | 固定長配列の重みを使う小さなニューラルネットのXOR推論 |

## 現在の範囲

これらは、数値、真偽値、束縛、関数、条件分岐、ループ、名前付きproduct type、固定長配列で表現できるプログラムです。

配列の要素は読み取れますが、まだ直接書き換えられません。そのため、探索や集計は自然に書けますが、in-place sortや配列を更新する動的計画法には要素の更新機能が必要です。文字列、再帰、動的な長さのcollectionもまだありません。

`xor_neural_network.prim`は、あらかじめ決めた重みを使う推論の例です。重みを変えて答えを学ぶ処理はまだ含みません。
