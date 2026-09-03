# Primer

日本語 | [English](README.en.md)

[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

Primerは、コンパイラによる変換を観測可能にするための実験用プログラミング言語です。

コンパイラは変換境界を明示します。同じ意味に解決されたPrimerプログラムを、C、LLVM IR、QBE IR、WebAssembly Text、Windows x86-64直接アセンブリ、Primer bytecodeへloweringできます。

Primer bytecodeは、Primer自身の小さな仮想マシンでも実行できます。

## コンパイラ構成

すべてのバックエンドは、同じフロントエンドと、型付きでバックエンドに依存しない同じPrimer IRを共有します。

```text
Primer Source
      ↓
Lexer / Parser
      ↓
AST
      ↓
Primer IR Builder
  - semantic validation
  - type resolution
  - contextual float resolution
      ↓
Primer IR
      │
      ├── emit-ir / .pir
      │
      ↓
Backend Lowering
      ↓
Backend-specific Rust IR
      ↓
Emitter
      ↓
Backend Artifact
```

この構成で重要な境界はPrimer IRです。

フロントエンドは、ソースプログラムの意味を決定します。各バックエンドは、解決済みの意味を自身の内部Rust表現へloweringしてから成果物を出力します。

Primerのバックエンドemitterは、ASTを再解釈したり、意味や型の解決を繰り返したりしません。

詳しい構成と不変条件は、[コンパイラ設計](docs/design/architecture.ja.md)を参照してください。

## 観測点

Primerは二つの主要な観測境界を公開します。

**観測1: 解決済みのPrimerの意味**

```sh
primer emit-ir examples/hello.prim
primer emit-ir examples/hello.prim -o hello.pir
```

Primer IRは、意味と型の解決後、バックエンド固有のloweringより前に生成されます。

**観測2: 出力成果物**

```text
emit-c         → .c
emit-llvm      → .ll
emit-qbe       → .ssa
emit-wat       → .wat
emit-asm       → .s
emit-bytecode  → .pbc
```

既存の`emit-*`コマンドが観測面です。バックエンド固有のRust IRは、内部のlowering境界として扱います。

## v0.1の範囲

Primerは現在、次の機能を備えています。

- 静的型付け
- `bool`、`i64`、`f32`、`f64`
- 明示的な型宣言
- `infer`による明示的な型推論
- 不変な変数束縛
- `+`、`-`、`*`、`/`
- 単項`-`
- `==`、`!=`、`<`、`<=`、`>`、`>=`
- 単項`!`
- `if` / `else`とブロックスコープ
- `while`と、最内側のループを対象にする`break` / `continue`
- 初期化、条件、更新を明示する`for`
- 丸括弧
- `print(expr);`
- `//`による行コメント
- `mut`による可変な束縛
- 型を保った再代入
- Primer IRの出力
- Cコード生成
- LLVM IR生成
- QBE IR生成
- WebAssembly Text生成
- Windows x86-64直接アセンブリ生成
- Primer bytecode生成
- Primer VMによる実行

例を示します。

```primer
integer: i64 = 1 + 2;

single: f32 = 0.1 + 0.2;
double: f64 = 0.1 + 0.2;

inferred: infer = single + single;

print(integer);
print(single);
print(double);
print(inferred);
```

現在の機能だけで動く例として、[平方根の近似](examples/square_root.prim)、[`while`による平方根の近似](examples/while_square_root.prim)、[ユークリッドの互除法](examples/euclidean_gcd.prim)、[`break`と`continue`によるループ制御](examples/loop_control.prim)、[`for`による合計](examples/for_sum.prim)、[f32・f64のロジスティック写像](examples/logistic_map.prim)、[真偽値と比較演算](examples/boolean_comparisons.prim)、[条件分岐とスコープ](examples/conditional.prim)があります。

型の指定は常に必要です。型を省略する代わりに、`infer`で型推論を明示的に要求します。

```primer
x: infer = 1 + 2;
```

接尾辞のない浮動小数点リテラルは、可能な場合、文脈に基づいて型付けされます。

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
```

この決定は、バックエンドへのloweringより前にPrimer IRで解決されます。

Primerは現在、`i64`、`f32`、`f64`の間で暗黙の数値変換を行いません。

## インストール

リポジトリをcheckoutしたディレクトリで、次のコマンドを実行します。

```sh
cargo install --path .
```

Primer自体を開発している場合は、変更後にCLIを再インストールします。

```sh
cargo install --path . --force
```

## CLI

ソースファイルを検証します。

```sh
primer check examples/hello.prim
```

解決済みのPrimer IRを出力します。

```sh
primer emit-ir examples/hello.prim
primer emit-ir examples/hello.prim -o hello.pir
```

各出力経路の成果物を生成します。

```sh
primer emit-c examples/hello.prim -o hello.c
primer emit-llvm examples/hello.prim -o hello.ll
primer emit-qbe examples/hello.prim -o hello.ssa
primer emit-wat examples/hello.prim -o hello.wat
primer emit-asm examples/hello.prim -o hello.s
primer emit-bytecode examples/hello.prim -o hello.pbc
```

Primer bytecodeとPrimer VMを通して実行します。

```sh
primer run examples/hello.prim
```

`-o`を指定しない場合、emitコマンドは標準出力へ書き出します。

## 出力経路

各出力経路を実現するバックエンドは、同じ構造に従います。

```text
Primer IR
    ↓
backend::lower()
    ↓
Backend-specific Rust IR
    ↓
backend::emit()
    ↓
Artifact
```

現在の経路は次のとおりです。

| 出力経路 | 現在のターゲット | 成果物 | 代表的な次の処理 |
| --- | --- | --- | --- |
| C | Primerでは指定しない | `.c` | GCC / Clang |
| LLVM IR | Primerでは指定しない | `.ll` | LLVM / Clang |
| QBE IR | Primerでは指定しない | `.ssa` | QBE |
| WebAssembly Text | WebAssembly | `.wat` | WebAssembly toolchain |
| ネイティブアセンブリ | x86-64、Windows、Windows x64 ABI | `.s` | assembler / linker |
| Primer bytecode | Primer VM | `.pbc` | Primer VM |

たとえば、次のような経路になります。

```text
Primer Source
      ↓
Primer IR
      ├──→ C IR        → .c   → GCC / Clang
      ├──→ LLVM IR     → .ll  → LLVM / Clang
      ├──→ QBE IR      → .ssa → QBE
      ├──→ WAT IR      → .wat → WebAssembly toolchain
      ├──→ ASM IR      → .s   → assembler / linker
      └──→ Bytecode IR → .pbc → Primer VM
```

Primerは、成果物を出力するまでの変換を担当します。外部コンパイラのバージョン、最適化レベル、対象CPU、ベンチマーク設定、測定方法は、Primerを呼び出す側が決定します。

出力経路、ターゲット、成果物、バックエンドの区別は、[出力経路とターゲット](docs/design/targets.ja.md)で定義します。

## 設計方針

Primerは、洗練された実装と可観測性の両立を目指します。実装や変換が高度になっても、その境界と結果を観測可能に保ちます。

具体的には、次の方針に従います。

- フロントエンドで型に関する決定を解決してから、バックエンドへloweringします。
- バックエンド固有の決定は、明示的なlowering境界の内側で行います。
- emitterはPrimerの意味を再解釈せず、バックエンドIRを出力形式へ変換します。
- 最適化を行う場合は、明示的で観測可能なパスとして導入します。
- 生成する観測結果から、可能な限り偶発的な非決定性を取り除きます。

目的は、単にコードを生成することではありません。ソースの意味から対象表現に至る経路を観察可能に保つことです。

## Tint\*

[Tint\*](https://github.com/Hokutaka/Tint-St.)は、Primerのための視覚的な開発・観察環境です。

Tint*はPrimerが公開するCLI出力を利用し、ソースと生成された表現を並べて表示します。外部ツールを呼び出し、C、LLVM、QBEから生成されたアセンブリなどの後続表現を表示することもできます。

Primerは言語ツールチェーンであり、Tint*はその内部を観察するための窓です。

## Whitebase

[Whitebase](https://github.com/Hokutaka/Whitebase)は、Primerの成果物を利用する外部ツールです。

Primerは次の処理を担当します。

```text
Transform → Lower → Emit → Observe
```

Whitebaseは、それらの成果物を受け取り、次の処理を行えます。

```text
Route → Build → Run → Measure → Compare
```

これにより、コンパイラの意味はPrimerの内側に保ち、ツールチェーンの選択、ベンチマーク、比較方法をPrimerの外側に置きます。

## ドキュメント

Primerの文書は、目的ごとに整理されています。

- [コンパイラ設計](docs/design/architecture.ja.md)
- [可観測性の契約](docs/design/observability.ja.md)
- [出力経路とターゲット](docs/design/targets.ja.md)
- [コンパイラ進化計画（Draft）](docs/design/evolution-plan.ja.md)
- [設計判断のための利用シナリオ（Draft）](docs/design/use-case-analysis.ja.md)
- [Secret値の設計（Draft）](docs/design/secrets.ja.md)
- [名前付きproduct typeの設計（Draft）](docs/design/product-types.ja.md)
- [言語リファレンス](docs/reference/language.ja.md)
- [CLIリファレンス](docs/reference/cli.ja.md)

英語版を含む文書全体の索引は、[docs/README.md](docs/README.md)にあります。

## ライセンス

[MIT License](LICENSE)の条件で公開しています。
