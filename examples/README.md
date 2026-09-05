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
| [hello.prim](hello.prim) | 整数に名前を付け、足し算の結果を`print`で表示する最初の例 |
| [floating_point.prim](floating_point.prim) | `f32`と`f64`の精度の違い、`infer`による型推論 |
| [integer_limits.prim](integer_limits.prim) | `i64`の最小値・最大値と、桁あふれする前の判定 |
| [integer_conversions.prim](integer_conversions.prim) | 同じ意味になる二つの変換表記。現在は`i64`から`i64`への変換 |
| [boolean_comparisons.prim](boolean_comparisons.prim) | 真偽値と比較演算 |
| [conditional.prim](conditional.prim) | `if` / `else`とscope |
| [loop_control.prim](loop_control.prim) | `while`、`break`、`continue` |
| [for_sum.prim](for_sum.prim) | `for`と開始文の再代入 |
| [product-point.prim](product-point.prim) | 名前付きproduct type、既定値、field access |
| [functions.prim](functions.prim) | 型付き関数、parameter、戻り値、`void`関数 |
| [function_values.prim](function_values.prim) | product typeと入れ子の固定長配列を関数へ値として渡す |
| [fixed_arrays.prim](fixed_arrays.prim) | 固定長配列、添字参照、配列の値コピー |
| [bubble_sort.prim](bubble_sort.prim) | `mut`な固定長配列の要素更新 |
| [product_arrays.prim](product_arrays.prim) | 点の配列、product typeの添字参照、配列の値コピー |
| [matrix_vector_product.prim](matrix_vector_product.prim) | 入れ子の固定長配列と二段の添字参照 |
| [matrix_composition.prim](matrix_composition.prim) | product typeと入れ子配列を関数で受け渡す数値計算 |

## 数値計算

| サンプル | 内容 |
| --- | --- |
| [square_root.prim](square_root.prim) | 手順を展開した平方根の近似 |
| [while_square_root.prim](while_square_root.prim) | `while`で繰り返す平方根の近似 |
| [logistic_map.prim](logistic_map.prim) | `f32`と`f64`で生まれる計算結果の違い |
| [heat_diffusion.prim](heat_diffusion.prim) | 棒の熱が広がる4段階の計算。更新前の配列から次の温度を求める |
| [linear_regression.prim](linear_regression.prim) | 5点から直線を学習し、傾き・切片・誤差の変化を追う |

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
| [bubble_sort.prim](bubble_sort.prim) | 要素を入れ替えるin-placeのバブルソート |
| [product_arrays.prim](product_arrays.prim) | 点の配列から最も近い点を探す線形探索 |
| [matrix_vector_product.prim](matrix_vector_product.prim) | 3×3行列と3要素vectorの積 |
| [matrix_composition.prim](matrix_composition.prim) | 2×2行列の合成と、合成した行列によるvector変換 |
| [xor_neural_network.prim](xor_neural_network.prim) | 固定長配列の重みを使う小さなニューラルネットのXOR推論 |
| [coin_change.prim](coin_change.prim) | 少ない金額の答えを使い回して最少枚数を求め、使った硬貨も復元する動的計画法 |
| [shortest_paths.prim](shortest_paths.prim) | 途中で寄れる町を増やして、全組み合わせの最短距離を求める |

## 計算途中を読む

追加例では、答えに至る途中の数値も`print`しています。出力の順番は各ファイルの日本語コメントで説明しています。

- `coin_change.prim`: 1円から6円までの最少枚数、その後に使う硬貨の3円と3円。
- `shortest_paths.prim`: 町0から町3への距離の変化、その後に4行4列の距離表。`-1`は到達できない印です。
- `heat_diffusion.prim`: 1段階につき5区間の温度を4回、その後に保存しておいた初期の中央温度。
- `linear_regression.prim`: 学習前の誤差、10回ごとの学習回数・傾き・切片・誤差、最後に新しい入力3の予測値。

直線の学習を試すには、`rate`（1回でどれだけ動かすか）や繰り返し回数を変え、誤差の変化を比較できます。各段階の表現を見るには、たとえば次を実行します。

```powershell
cargo run --quiet -- run examples/linear_regression.prim
cargo run --quiet -- emit-ir examples/linear_regression.prim
cargo run --quiet -- emit-bytecode examples/linear_regression.prim
cargo run --quiet -- emit-c examples/linear_regression.prim
```

`integer_limits.prim`は通常は成功します。末尾のコメントアウトした式を有効にすると、桁あふれによる停止と診断位置を確認できます。

## 現在の範囲

これらは、数値、真偽値、束縛、関数、条件分岐、ループ、名前付きproduct type、固定長配列で表現できるプログラムです。

`mut`な配列では要素を直接更新できるため、in-place sortや配列を更新する動的計画法も表現できます。文字列、再帰、動的な長さのcollectionはまだありません。

`xor_neural_network.prim`は、あらかじめ決めた重みを使う推論の例です。`linear_regression.prim`では、勾配降下法で直線の傾きと切片をデータから学びます。XORニューラルネット自体の学習はまだ含みません。
