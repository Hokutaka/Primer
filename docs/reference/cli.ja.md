# Primer CLIリファレンス

[English](cli.en.md)

この文書では、Primer v0.1のコマンドラインインターフェースを定義します。

## コマンド

現在のCLIは次のコマンドを提供します。

```text
primer check <file>
primer emit-ir <file> [-o <output.pir>]
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-wat <file> [-o <output.wat>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
primer run <file>
primer --version
```

## 検証

```text
primer check <file>
```

`primer check`は、入力されたソースファイルの構文解析、意味検証、型検査を行います。

## Primer IRの出力

```text
primer emit-ir <file> [-o <output.pir>]
```

`primer emit-ir`は、意味と型が解決されたバックエンド非依存のPrimer IRを出力します。

## 出力成果物の生成

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
```

各コマンドが出力する成果物は次のとおりです。

| コマンド | 出力経路 | 現在のターゲット | 成果物 |
| --- | --- | --- | --- |
| `emit-c` | C | Primerでは指定しない | `.c` |
| `emit-llvm` | LLVM IR | Primerでは指定しない | `.ll` |
| `emit-qbe` | QBE IR | Primerでは指定しない | `.ssa` |
| `emit-wat` | WebAssembly Text | WebAssembly | `.wat` |
| `emit-asm` | ネイティブアセンブリ | x86-64、Windows、Windows x64 ABI | `.s` |
| `emit-bytecode` | Primer bytecode | Primer VM | `.pbc` |

`emit-*`コマンドは、`-o`を指定しない場合、観測結果を標準出力へ書き出します。`-o`を指定した場合は、利用者が出力先のパスを決定します。

現在の`emit-asm`にはターゲットを選択するオプションはなく、x86-64 Windows向けのアセンブリを生成します。

## 実行

```text
primer run <file>
```

`primer run`はPrimer bytecodeへloweringし、生成された`BytecodeProgram`をPrimer VMで実行します。

実行結果は検証や実験に利用できますが、[コンパイラ設計](../design/architecture.ja.md)で定める二つのコンパイラ観測境界とは区別します。

## バージョン表示

```text
primer --version
```

`primer --version`はPrimerのバージョンを表示します。

## Primerが扱わない外部設定

Primerは、次のような外部実験の方針を決定しません。

- GCCとClangのどちらを使用するか
- 外部コンパイラの最適化レベル
- 外部ツールチェーンの対象CPU
- ベンチマーク設定
- 測定方法
- 比較方法

これらはPrimerを呼び出す側が決定し、必要に応じて記録します。
