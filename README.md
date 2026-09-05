[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

# Primer

日本語 | [English](README.en.md)

Primerは、コンパイラによる変換を観測可能にするための実験用プログラミング言語です。

計算結果だけでなく、「どの型で計算し、どんなコードへ変換されたか」を調べられることを重視します。意味と型を解決した共通のPrimer IR（中間表現）から、各出力先のコードやbytecodeを生成します。洗練された実装と可観測性の両立を目指し、観測することと内部を書き換えることは区別します。

## まず動かす

Rustの開発環境（rustupとCargo）が必要です。リポジトリを取得し、そのルートでCLIをインストールします。

```sh
git clone https://github.com/Hokutaka/Primer.git
cd Primer
cargo install --path .
```

[examples/floating_point.prim](examples/floating_point.prim)は、同じ足し算を異なる型で行う例です。

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
c: infer = 0.1 + 0.2;

print(a);
print(b);
print(c);
```

```sh
primer run examples/floating_point.prim
```

Primer VMでの実行結果は次のとおりです。

```text
0.300000012
0.30000000000000004
0.30000000000000004
```

`f32`と`f64`では数値を表せる精度が違います。`infer`は型推論を明示する指定で、この例の`c`は`f64`になります。

開発中は`primer`の代わりに`cargo run --quiet --`を使うと、再インストールせずに変更後のコードを実行できます。

## 計算と変換を観測する

同じソースを、実行するだけでなく中間表現や生成コードとして確認できます。

```sh
primer emit-ir examples/floating_point.prim
primer emit-c examples/floating_point.prim
```

`emit-ir`では解決済みの型と演算を、`emit-c`ではそれらをCでどう表現したかを読めます。バックエンドは共通のPrimer IRを受け取り、ソースの意味を解釈し直しません。

`emit-*`は標準出力へ書き出します。ファイルに残す場合は、例えば`primer emit-c examples/floating_point.prim -o floating_point.c`と指定します。構文や型の検証だけなら`primer check examples/floating_point.prim`を使います。

公開している観測点はPrimer IRと出力成果物です。バックエンド固有のRust IRは内部の変換境界として扱います。詳しくは[コンパイラ設計](docs/design/architecture.ja.md)と[可観測性の契約](docs/design/observability.ja.md)を参照してください。

## 現在できること

- **型と変数:** 静的型付け。`bool`、`i8`・`u8`・`i16`・`u16`・`i32`・`u32`・`i64`、`f32`・`f64`、`string`。型宣言、`infer`、不変な束縛と`mut`による可変な束縛。
- **データ構造:** 名前付き構造体（product type）、フィールドの既定値と参照、入れ子にできる固定長配列。値コピーと配列要素の更新。
- **関数と制御:** 型付き関数、`void`、明示的な`return`。トップレベル実行文または`fn main() -> void`。`if` / `else`、`while`、`for`、`break` / `continue`。
- **演算:** 算術、整数の剰余とビット演算、比較、`!`、短絡評価する`&&`・`||`。
- **明示変換:** `f64(value)`と`convert<f64>(value)`など、同じ意味の二つの表記。実装済みの数値型の間で、値を保てる場合だけ変換。
- **出力と実行:** `print(expr);`、Primer IRと各出力先の成果物の生成、Primer VMによる実行。

整数の桁あふれ、不正な整数除算、配列の範囲外参照、値を保てない変換では実行を停止します。暗黙の数値変換はしません。通常の浮動小数点計算には丸めがあります。

文字列はUTF-8の不変な値で、表示・等値比較・関数やデータ構造での受け渡しに対応します。現在は`check`・`emit-ir`・`emit-bytecode`・`run`で使用できます。C・LLVM・QBE・WAT・アセンブリへの出力は未対応としてソース位置付きで診断します。連結や文字列の添字参照は未実装です。

`u64`、動的な長さの配列、再帰、失敗からの回復、明示的な丸め・切り捨て操作は未実装です。現在の生成先では小さい整数型も64ビット領域に格納し、値の範囲を検査します。

### 出力先

| コマンド | 成果物 | その後の処理 |
| --- | --- | --- |
| `emit-c` | C（`.c`） | GCC / Clangなどでコンパイル |
| `emit-llvm` | LLVM IR（`.ll`） | LLVM / Clangでコンパイル |
| `emit-qbe` | QBE IR（`.ssa`） | QBEで処理 |
| `emit-wat` | WebAssembly Text（`.wat`） | WebAssembly用ツールとホストで実行 |
| `emit-asm` | Windows x86-64アセンブリ（`.s`） | アセンブル・リンク |
| `emit-bytecode` | Primer bytecode（`.pbc`） | 命令列を確認。VM実行はソースに対する`run`を使用 |

Primerは成果物の生成までを担当します。外部ツールの選択、対象CPUや最適化設定、測定方法は呼び出す側が決定します。詳細は[出力経路とターゲット](docs/design/targets.ja.md)を参照してください。

## サンプルと文書

| 分類 | サンプル |
| --- | --- |
| 基本 | [小さな数値の表示](examples/small_values.prim)、[短絡評価](examples/short_circuit.prim) |
| データ構造 | [リングバッファ](examples/ring_buffer.prim)、[構造体と配列の受け渡し](examples/function_values.prim) |
| 数値計算 | [測定値の平均・分散](examples/measurement_statistics.prim)、[直線の学習](examples/linear_regression.prim) |
| アルゴリズム | [最短経路](examples/shortest_paths.prim)、[部分和のビット集合](examples/subset_sum_bits.prim) |

[サンプル一覧](examples/README.md)から、ほかの例も探せます。リポジトリのルートからまとめて実行できます。

PowerShell:

```powershell
.\scripts\run-examples.ps1
```

WSL / Bash（WSL側にもRustの開発環境が必要です）:

```bash
bash scripts/run-examples.sh
bash scripts/test.sh
```

`run-examples`はサンプルの実行結果を表示します。PowerShellでは`-Pattern "matrix*.prim"`、Bashでは`--pattern 'matrix*.prim'`で対象を絞れます。`test.sh`はfmt・clippy・全テストを実行し、期待する結果との照合も行います。サンプルのテストだけなら`cargo test --test examples`を使います。

`.sh`側のビルド先は既定で`target/unix`です。Windowsの生成物とは分離し、`CARGO_TARGET_DIR`が指定されていればそちらを使います。

- [言語リファレンス](docs/reference/language.ja.md): 現在の構文、型、演算、変換の規則。
- [CLIリファレンス](docs/reference/cli.ja.md): コマンドとオプション。
- [文書一覧](docs/README.md): 設計判断を記録する`docs/design/`と、現在の仕様を記録する`docs/reference/`の日英索引。

## 関連ツール

- [Tint\*](https://github.com/Hokutaka/Tint-St.): ソースと生成された表現を並べて観察するための開発・観察環境。
- [Whitebase](https://github.com/Hokutaka/Whitebase): Rust・C++・Assemblyの組み込み演算を実行・測定・比較する実験環境。Primerの生成物との連携は未実装です。

言語の意味とコンパイル処理はPrimerが担当します。Whitebaseとの連携では、生成物を使った実験を利用側に分ける方針です。現在の実装と連携時の境界は[ツールの責務](docs/design/architecture.ja.md#ツールの責務)を参照してください。

## ライセンス

[MIT License](LICENSE)の条件で公開しています。
