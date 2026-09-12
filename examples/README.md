# Primerサンプル

[English](README.en.md)

このディレクトリには、現在のPrimerで読んで実行できるプログラムを置きます。それぞれの例は、別の構文や計算方法を小さく示します。

[意図した停止の例](runtime_failures/README.md)には、停止前の出力と失敗する式を追う4例を分けて置いています。

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

## 型から探す

8種類の整数型、`f32`・`f64`、`bool`、`string`、固定長配列、構造体を使えます。既存7経路（VM・C・LLVM・QBE・WAT・Windows x64直接アセンブリ・Linux x86-64直接アセンブリ）に`u64`まで実装しています。`u64_values.prim`は全7経路で既知の期待出力と比較しています。[native_values.prim](native_values.prim)はWindows/Linuxの混在引数と値の受け渡しを確認する例です。機械語を含むオブジェクトと実行ファイルの生成・実行・観測には、[明示した外部ツールを使う手順](../docs/design/native-code.ja.md)を用意しています。[自前エンコーダ](../docs/design/native-encoder.ja.md)でも同じ言語機能を実行できます。

### 符号付き整数：負数と正数

| 型 | 数値範囲 | サンプル | 確認すること |
| --- | --- | --- | --- |
| `i8` | −128〜127 | [sensor_calibration.prim](sensor_calibration.prim) | 小さな負の補正量を、計算前に`i16`へ広げる |
| `i16` | −32768〜32767 | [sensor_calibration.prim](sensor_calibration.prim) | 正負の測定値を保持し、集計時には`i32`へ広げる |
| `i32` | −2147483648〜2147483647 | [maximum_subarray.prim](maximum_subarray.prim) | 正負の増減を足し、大小を比較する |
| `i64` | −9223372036854775808〜9223372036854775807 | [integer_limits.prim](integer_limits.prim) | 最小値・最大値と、桁あふれ前の判定 |

### 符号なし整数：非負の数とビット列

| 型 | 数値範囲 | サンプル | 確認すること |
| --- | --- | --- | --- |
| `u8` | 0〜255 | [color_blending.prim](color_blending.prim)、[bit_flags.prim](bit_flags.prim) | 色の値と、8個のビットの設定・解除・反転 |
| `u16` | 0〜65535 | [color_blending.prim](color_blending.prim) | `u8`の色を加算前に広げ、平均を求めて戻す |
| `u32` | 0〜4294967295 | [population_statistics.prim](population_statistics.prim) | 30億前後の値を保持し、集計時には`i64`へ広げる |
| `u64` | 0〜18446744073709551615 | [u64_values.prim](u64_values.prim), [packet_counter.prim](packet_counter.prim) | 最大値、最上位ビット、符号なし比較・除算、関数・配列・コピー、正確な変換 |

u64の例は次のコマンドで実行できます。

```sh
cargo run -- run examples/u64_values.prim
```

先頭の出力は順に`u64`、`18446744073709551615`、`9223372036854775808`、`0`です。元の束縛を再代入してもコピーは保持され、最上位ビットも正の数値として扱います。[u64の設計](../docs/design/u64.ja.md)では各生成先の表現も説明しています。

整数の範囲外演算は停止し、折り返しません。型同士は暗黙に混ぜず、`i64(value)`や`convert<i64>(value)`で明示変換します。[integer_conversions.prim](integer_conversions.prim)で二つの表記を比較できます。型情報のない整数は`i64`になり、配列の添字も`i64`です。表のビット幅は数値の範囲を表し、現在の生成先では小さい整数も64ビット領域に格納します。

### 浮動小数点：小数と精度

| 型 | 表現 | サンプル | 確認すること |
| --- | --- | --- | --- |
| `f32` | 32ビット浮動小数点 | [floating_point.prim](floating_point.prim)、[logistic_map.prim](logistic_map.prim) | `f64`との丸め・計算結果の違い |
| `f64` | 64ビット浮動小数点 | [floating_point.prim](floating_point.prim)、[small_values.prim](small_values.prim) | 小さい値の表示と計算時の丸め。型情報のない浮動小数点は`f64` |

[measurement_statistics.prim](measurement_statistics.prim)と[normalized_histogram.prim](normalized_histogram.prim)では整数と浮動小数点を行き来します。明示変換は値を保てる場合だけ成功します。通常の浮動小数点演算で生じる丸めとは別の規則です。

### 真偽値と文字列

| 型 | 値 | サンプル | 確認すること |
| --- | --- | --- | --- |
| `bool` | `true` / `false` | [boolean_comparisons.prim](boolean_comparisons.prim)、[short_circuit.prim](short_circuit.prim) | 比較・否定と、評価を省く短絡評価 |
| `string` | 内容が不変のUTF-8文字列 | [string_values.prim](string_values.prim)、[string_byte_length.prim](string_byte_length.prim) | 日本語・等値比較・コピーと、文字数とは異なるUTF-8バイト数 |

[string_origins.prim](string_origins.prim)は文字列の処理をPrimer IRと出自注釈付きLLVMで辿る例です。文字列の内容は不変ですが、mutな束縛への再代入はできます。

### 配列と構造体：型を組み合わせる

| 型の形 | サンプル | 確認すること |
| --- | --- | --- |
| 固定長配列 `[T; N]` | [fixed_arrays.prim](fixed_arrays.prim)、[bubble_sort.prim](bubble_sort.prim) | 添字、要素の更新、コピー後の独立性 |
| 構造体 `type Point { ... }` | [product-point.prim](product-point.prim)、[product_arrays.prim](product_arrays.prim) | フィールド・既定値と、構造体を要素にする配列 |
| 入れ子の配列・構造体 | [function_values.prim](function_values.prim)、[u64_values.prim](u64_values.prim)、[string_lookup.prim](string_lookup.prim) | 数値や文字列を組み合わせ、関数へ値として渡す |

`infer`は独立した値の型ではなく、型を推論する指定です。`void`は値を返さない関数の戻り方を表します。[floating_point.prim](floating_point.prim)と[functions.prim](functions.prim)で確認できます。

## 基本と制御

| サンプル | 内容 |
| --- | --- |
| [hello.prim](hello.prim) | 整数に名前を付け、足し算の結果を`print`で表示する最初の例 |
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
| [packet_counter.prim](packet_counter.prim) | u64の最上位ビット、u8のフラグ、不変の文字列を構造体で渡し、自前エンコーダで実行する |
| [native_values.prim](native_values.prim) | u64・整数・小数・文字列・構造体を混在する4引数で渡し、Windows/Linuxの実行と機械語までを辿る |

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

文字列のサンプルは既存7経路に対応します。LLVMとQBEには明示的なターゲットを渡し、QBEはLinux x86-64、直接アセンブリはWindows x64 / Linux x86-64、WATは出力用ホスト関数を備えたWebAssembly環境で検証します。`emit-ir`と`emit-bytecode`でも型と内容の変換を読めます。

QBE・WAT・直接アセンブリの実行比較は`cargo test --test string_routes`で確認できます。[文字列の設計](../docs/design/strings.ja.md#検証範囲)にツールの指定と検証範囲を記載しています。

例えば、文字列をキーにした検索をCへ変換できます。

```sh
cargo run --quiet -- emit-c examples/string_lookup.prim -o target/string_lookup.c
clang -std=c11 target/string_lookup.c -o target/string_lookup
```

生成した実行ファイルをBashでは`./target/string_lookup`、Windowsでは`.\target\string_lookup.exe`で実行します。外部のCコンパイラが必要です。

LLVMの場合は、[CLIリファレンス](../docs/reference/cli.ja.md#llvmのターゲット指定)にあるWindows/Linuxのコマンド例を使ってください。`cargo test --test llvm_strings`で、文字列のVM・生成C・生成LLVMの出力をバイト単位で比較できます。`PRIMER_TEST_LLVM_CLANG`と`PRIMER_TEST_CC`を設定すると、指定したコンパイラがない場合もテスト失敗になります。

`cargo test --test c_strings`はC生成物を最適化あり・なしで実行し、VMの結果と比較します。既定のCコンパイラがない環境では実行比較をスキップしますが、`PRIMER_TEST_CC`にコンパイラを指定すると検証を必須にできます。CIではClangを必須とし、AddressSanitizerとUndefinedBehaviorSanitizerでも検査します。

`xor_neural_network.prim`は、あらかじめ決めた重みを使う推論の例です。`linear_regression.prim`では、勾配降下法で直線の傾きと切片をデータから学びます。XORニューラルネット自体の学習はまだ含みません。

### 文字列の変換元を辿る

`string_origins.prim`は関数呼び出し、文字列の内容比較、短絡評価を観察する例です。出力をエスケープ表記にすると`日本語\0\ntrue\nfalse\n`です。`skipped`は出力されません。

`emit-ir`と`emit-llvm --annotate-origins`を並べると、`#7`の内容比較や`#14`の短絡評価からLLVMの呼び出し・分岐へ辿れます。[実行手順と出自注釈](../docs/reference/cli.ja.md#llvmの出自を辿る)を参照してください。

### 文字列のバイト数を確かめる

`string_byte_length.prim`は`byte_len`を使い、UTF-8の長さ、コピー済みの値、関数・配列・既定値、評価順を確認する例です。

```sh
cargo run -- run examples/string_byte_length.prim
cargo run -- emit-ir examples/string_byte_length.prim
cargo run -- emit-llvm examples/string_byte_length.prim --target x86_64-unknown-linux-gnu --annotate-origins -o string-byte-length.ll
```

出力は順に`0, 9, 3, 2, 3, 4, 7, 3, 9, left, right, 9, false, false, 6, 10`で、各値の後にLFが付きます。`left`と`right`は各一回だけ出力され、`skipped`は出力されません。C・LLVM・QBE・WAT・直接アセンブリも実行して既知の期待バイト列と比較します。小さい入力と各経路の表現は[観測fixture](../tests/fixtures/observation/string-byte-length/)で読めます。
