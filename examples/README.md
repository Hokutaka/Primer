# Primerサンプル

[English](README.en.md)

このディレクトリには、現在のPrimerで読んで実行できるプログラムを置きます。それぞれの例は、別の構文や計算方法を小さく示します。

## まとめて実行

リポジトリのルートから次を実行すると、すべてのサンプルについて名前、実行結果、成否、最後の集計を表示します。

```powershell
.\scripts\run-examples.ps1
```

`-Pattern "matrix*.prim"`で対象を絞れます。`-SkipBuild`を指定すると、既にbuildされているPrimerを使います。

WSL / Bashでは次を使います。WSL側にもRustの開発環境が必要です。

```bash
bash scripts/run-examples.sh
bash scripts/run-examples.sh --pattern 'matrix*.prim' --skip-build
```

`.sh`側は既定で`target/unix/debug/primer`を使い、Windowsの生成物と分離します。`CARGO_TARGET_DIR`を指定した場合はその出力先を使います。`--skip-build`は、同じ出力先へ一度ビルドした後に指定してください。

一括実行は各サンプルの終了状態を確認します。期待する出力まで照合するには`cargo test --test examples`、fmt・clippy・全テストをまとめて実行するには`bash scripts/test.sh`を使います。

## 基本

| サンプル | 内容 |
| --- | --- |
| [hello.prim](hello.prim) | 整数に名前を付け、足し算の結果を`print`で表示する最初の例 |
| [string_values.prim](string_values.prim) | 日本語の表示、等値比較、改行、再代入しても保存済みの文字列が変わらないこと |
| [floating_point.prim](floating_point.prim) | `f32`と`f64`の精度の違い、`infer`による型推論 |
| [small_values.prim](small_values.prim) | 小さな数値を指数表記で観測し、表示と計算時の丸めを区別する |
| [integer_limits.prim](integer_limits.prim) | `i64`の最小値・最大値と、桁あふれする前の判定 |
| [integer_conversions.prim](integer_conversions.prim) | 同じ意味になる二つの変換表記で`i32`を`i64`へ広げる |
| [bit_flags.prim](bit_flags.prim) | 8個のビットを独立したスイッチとして使い、設定・解除・反転・判定を行う |
| [boolean_comparisons.prim](boolean_comparisons.prim) | 真偽値と比較演算 |
| [short_circuit.prim](short_circuit.prim) | `&&`・`\|\|`で条件を組み合わせ、不要な割り算・配列参照・関数呼び出しを省略する |
| [conditional.prim](conditional.prim) | `if` / `else`とscope |
| [loop_control.prim](loop_control.prim) | `while`、`break`、`continue` |
| [for_sum.prim](for_sum.prim) | `for`と開始文の再代入 |
| [functions.prim](functions.prim) | 型付き関数、parameter、戻り値、`void`関数 |

## データ構造

複数の値をどうまとめ、取り出し、受け渡すかを学ぶ例です。現在は構造体（名前付きproduct type）と固定長配列を使います。

| サンプル | 内容 |
| --- | --- |
| [ring_buffer.prim](ring_buffer.prim) | `%`で保存位置を循環させ、直近4件の値と平均を保つ |
| [string_lookup.prim](string_lookup.prim) | 文字列をキーに構造体の配列を線形探索し、対応する表示や既定値を返す |
| [product-point.prim](product-point.prim) | 点の座標を構造体にまとめる。フィールドの既定値と読み取り |
| [fixed_arrays.prim](fixed_arrays.prim) | 固定長配列の要素を読み、合計と線形探索を行う。コピーした配列が独立した値であることも確認する |
| [product_arrays.prim](product_arrays.prim) | 構造体を配列に並べ、最も近い点を探す。配列のコピーも確認する |
| [function_values.prim](function_values.prim) | 構造体と入れ子の固定長配列を、関数へ値として渡して受け取る |

## 数値計算

| サンプル | 内容 |
| --- | --- |
| [measurement_statistics.prim](measurement_statistics.prim) | 整数の測定値から小数の平均・分散を求め、値を変えずに`f32`へ保存する |
| [normalized_histogram.prim](normalized_histogram.prim) | 整数の回数を小数の割合へ変換し、保存した割合から元の回数を復元する |
| [square_root.prim](square_root.prim) | 手順を展開した平方根の近似 |
| [while_square_root.prim](while_square_root.prim) | `while`で繰り返す平方根の近似 |
| [logistic_map.prim](logistic_map.prim) | `f32`と`f64`で生まれる計算結果の違い |
| [matrix_vector_product.prim](matrix_vector_product.prim) | 入れ子の固定長配列を使った3×3行列と3要素ベクトルの積 |
| [matrix_composition.prim](matrix_composition.prim) | 構造体と入れ子配列を関数で受け渡し、2×2行列の合成とベクトル変換を行う |
| [population_statistics.prim](population_statistics.prim) | `u32`の大きな値を`i64`へ広げて集計し、平均と最大値を構造体で返す |
| [heat_diffusion.prim](heat_diffusion.prim) | 棒の熱が広がる4段階の計算。更新前の配列から次の温度を求める |
| [linear_regression.prim](linear_regression.prim) | 5点から直線を学習し、傾き・切片・誤差の変化を追う |

## アルゴリズム

| サンプル | 内容 |
| --- | --- |
| [color_blending.prim](color_blending.prim) | `u8`の色を足す前に`u16`へ広げ、平均を求めて混ぜる |
| [sensor_calibration.prim](sensor_calibration.prim) | `i16`の測定値を`i8`の補正量で調整し、`i32`へ広げて集計する |
| [maximum_subarray.prim](maximum_subarray.prim) | `i32`の増減から連続区間の最大合計を求める。`u32`の位置を添字へ変換する |
| [subset_sum_bits.prim](subset_sum_bits.prim) | シフトとビットORで、選んだ重さから作れる合計を一度に求める |
| [euclidean_gcd.prim](euclidean_gcd.prim) | ユークリッドの互除法による最大公約数 |
| [fibonacci.prim](fibonacci.prim) | Fibonacci数列と複数の値の更新順 |
| [factorial.prim](factorial.prim) | `for`による階乗 |
| [collatz.prim](collatz.prim) | Collatz予想と条件ごとの状態遷移 |
| [prime_check.prim](prime_check.prim) | 試し割りによる素数判定と早期終了 |
| [integer_square_root.prim](integer_square_root.prim) | 二分探索による整数平方根 |
| [exponentiation_by_squaring.prim](exponentiation_by_squaring.prim) | 繰り返し二乗法による累乗 |
| [pythagorean_triples.prim](pythagorean_triples.prim) | 入れ子の`for`によるピタゴラス数の探索 |
| [bubble_sort.prim](bubble_sort.prim) | `mut`な固定長配列の要素をその場で入れ替えるバブルソート |
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

これらは、数値、真偽値、文字列、束縛、関数、条件分岐、ループ、名前付きproduct type、固定長配列で表現できるプログラムです。

`mut`な配列では要素を直接更新できるため、in-place sortや配列を更新する動的計画法も表現できます。再帰、動的な長さのcollectionはまだありません。

文字列を使う2例はVM実行、C生成、明示的なターゲット付きLLVM生成に対応し、`emit-ir`と`emit-bytecode`でも変換を読めます。QBE・WAT・アセンブリへの出力にはまだ対応していません。

例えば、文字列をキーにした検索をCへ変換できます。

```sh
cargo run --quiet -- emit-c examples/string_lookup.prim -o target/string_lookup.c
clang -std=c11 target/string_lookup.c -o target/string_lookup
```

生成した実行ファイルをBashでは`./target/string_lookup`、Windowsでは`.\target\string_lookup.exe`で実行します。外部のCコンパイラが必要です。

LLVMの場合は、[CLIリファレンス](../docs/reference/cli.ja.md#llvmのターゲット指定)にあるWindows/Linuxのコマンド例を使ってください。`cargo test --test llvm_strings`で、文字列のVM・生成C・生成LLVMの出力をバイト単位で比較できます。`PRIMER_TEST_LLVM_CLANG`と`PRIMER_TEST_CC`を設定すると、指定したコンパイラがない場合もテスト失敗になります。

`cargo test --test c_strings`はC生成物を最適化あり・なしで実行し、VMの結果と比較します。既定のCコンパイラがない環境では実行比較をスキップしますが、`PRIMER_TEST_CC`にコンパイラを指定すると検証を必須にできます。CIではClangを必須とし、AddressSanitizerとUndefinedBehaviorSanitizerでも検査します。

`xor_neural_network.prim`は、あらかじめ決めた重みを使う推論の例です。`linear_regression.prim`では、勾配降下法で直線の傾きと切片をデータから学びます。XORニューラルネット自体の学習はまだ含みません。
