# Primer CLIリファレンス

[English](cli.en.md)

この文書では、Primer v0.1のコマンドラインインターフェースを定義します。

## コマンド

現在のCLIは次のコマンドを提供します。

```text
primer check <file>
primer emit-ir <file> [-o <output.pir>]
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [--target <triple>] [-o <output.ll>]
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

`check`の成功は、すべての出力経路がそのプログラムに対応することを保証しません。文字列は`check`・`emit-ir`・`emit-c`・`emit-bytecode`・`run`と、明示的な`--target`付きの`emit-llvm`で使用できます。LLVMのターゲット未指定やQBE・WAT・アセンブリへの出力は、文字列を含む型定義や式をソース位置付きで診断し、成果物を生成しません。この診断では`-o`で指定した既存ファイルも変更しません。

## Primer IRの出力

```text
primer emit-ir <file> [-o <output.pir>]
```

`primer emit-ir`は、意味と型が解決されたバックエンド非依存のPrimer IRを出力します。

## 出力成果物の生成

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [--target <triple>] [-o <output.ll>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
```

各コマンドが出力する成果物は次のとおりです。

| コマンド | 出力経路 | 現在のターゲット | 成果物 |
| --- | --- | --- | --- |
| `emit-c` | C | Primerでは指定しない | `.c` |
| `emit-llvm` | LLVM IR | 未指定、または明示的なWindows x64 / Linux x86-64 | `.ll` |
| `emit-qbe` | QBE IR | Primerでは指定しない | `.ssa` |
| `emit-wat` | WebAssembly Text | WebAssembly | `.wat` |
| `emit-asm` | ネイティブアセンブリ | x86-64、Windows、Windows x64 ABI | `.s` |
| `emit-bytecode` | Primer bytecode | Primer VM | `.pbc` |

`emit-*`コマンドは、`-o`を指定しない場合、観測結果を標準出力へ書き出します。`-o`を指定した場合は、利用者が出力先のパスを決定します。

現在の`emit-asm`にはターゲットを選択するオプションはなく、x86-64 Windows向けのアセンブリを生成します。

### LLVMのターゲット指定

`--target`は`x86_64-unknown-linux-gnu`または`x86_64-pc-windows-msvc`を受け付けます。文字列を使うプログラムでは、未使用の型・関数も含めて指定が必須です。数値だけの従来の呼び出しでは省略できます。ホストOSからは推測しません。`--target`と`-o`（`--output`も可）の順序は自由ですが、同じオプションの重複、値の省略、未対応ターゲットはエラーです。

Linux x86-64上での例:

```sh
primer emit-llvm examples/string_lookup.prim --target x86_64-unknown-linux-gnu -o target/string_lookup.ll
clang --target=x86_64-unknown-linux-gnu target/string_lookup.ll -o target/string_lookup
./target/string_lookup
```

Windows x64（MSVC CRTとリンカが利用可能な環境）での例:

```powershell
primer emit-llvm examples/string_lookup.prim --target x86_64-pc-windows-msvc -o target/string_lookup.ll
clang --target=x86_64-pc-windows-msvc target/string_lookup.ll -o target/string_lookup.exe
.\target\string_lookup.exe
```

PrimerはLLVMの生成だけを行い、Clangや実行ファイルを起動しません。指定は`target triple`に記録されます。下流ツールにも同じターゲットを渡します。Windowsでは文字列を含むプログラムの標準出力をバイナリモードにし、NUL・CR・LFを保持します。詳しくは[文字列の設計](../design/strings.ja.md#llvmでの表現とターゲット)を参照してください。

ライブラリでは`compile_to_llvm_with_target(source, Some(codegen::llvm::Target::X86_64UnknownLinuxGnu))`、または`X86_64PcWindowsMsvc`を使います。既存の`compile_to_llvm(source)`はターゲット未指定のAPIとして残ります。

## 実行

```text
primer run <file>
```

`primer run`はPrimer bytecodeへloweringし、生成された`BytecodeProgram`をPrimer VMで実行します。

実行結果は検証や実験に利用できますが、[コンパイラ設計](../design/architecture.ja.md)で定める二つのコンパイラ観測境界とは区別します。

実行時エラーがソースコードに由来するbytecode命令で発生した場合、診断にはソース位置とbytecode命令番号の両方を表示します。

```text
primer: cannot divide an integer by zero at 1:7 (bytecode instruction 0002)
```

対応するソース位置がない場合も、bytecode命令番号は表示します。簡潔な診断には、ソース本文や入力ファイルのパスを含めません。

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
