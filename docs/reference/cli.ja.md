# Primer CLIリファレンス

[English](cli.en.md)

この文書では、Primer v0.1のコマンドラインインターフェースを定義します。

## コマンド

現在のCLIは次のコマンドを提供します。

```text
primer check <file>
primer emit-sources <file> [-o <sources.json>]
primer emit-ir <file> [-o <output.pir>]
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [--target <triple>] [-o <output.ll>]
primer emit-wat <file> [-o <output.wat>]
primer emit-qbe <file> [--target <triple>] [-o <output.ssa>]
primer emit-asm <file> [--target <triple>] [--annotate-origins] [-o <output.s>]
primer emit-obj <file> --target <triple> [--annotate-origins] -o <output.o>
primer emit-bytecode <file> [-o <output.pbc>]
primer run <file> [--diagnostic-format runtime-v1]
primer --version
```

## 検証

各コマンドの`<file>`は入口ファイルです。[モジュール](../design/modules.ja.md)のimportは宣言元のディレクトリから解決し、全依存を検査します。CLIを別ディレクトリから起動してもimport先は変わりません。対象の生成経路に必要な`--target`指定は従来通りです。

### 依存ファイルの観測

```sh
primer emit-sources examples/modules/main.prim -o sources.json
```

ファイル名と本文を明示的に出力します。JSONの`schema`は`primer-sources-v1`、`files`は登録順の`id`・`name`・`text`です。`text`は正規化前の正確なUTF-8本文をJSONエスケープで保持します。import/pubのない入力のIDは既存の匿名ソース0、モジュール入力は入口1からです。実行時記録の`file`とそのファイル内の`bytes`を、この本文で照合します。生成プログラムへ本文を埋め込む操作ではありません。`-o`がなければstdoutへ出します。

`run`の既定診断は依存ファイルの名前・行・列を表示します。`--diagnostic-format runtime-v1`では数値の`file`を含む記録を出します。コンパイル失敗時は出力成果物を書き換えません。

### 構文・型の検証

```text
primer check <file>
```

`primer check`は、入力されたソースファイルの構文解析、意味検証、型検査を行います。

`check`の成功は、すべての出力経路がそのプログラムに対応することを保証しません。文字列は全経路で使用できます。LLVMとQBEのターゲットが未指定の場合は、文字列を含む型定義や式をソース位置付きで診断し、成果物を生成しません。この診断では`-o`で指定した既存ファイルも変更しません。

## Primer IRの出力

```text
primer emit-ir <file> [-o <output.pir>]
```

`primer emit-ir`は、意味と型が解決されたバックエンド非依存のPrimer IRを出力します。

## 出力成果物の生成

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [--target <triple>] [-o <output.ll>]
primer emit-qbe <file> [--target <triple>] [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [--target <triple>] [--annotate-origins] [-o <output.s>]
primer emit-obj <file> --target <triple> [--annotate-origins] -o <output.o>
primer emit-bytecode <file> [-o <output.pbc>]
```

各コマンドが出力する成果物は次のとおりです。

| コマンド | 出力経路 | 現在のターゲット | 成果物 |
| --- | --- | --- | --- |
| `emit-c` | C | Primerでは指定しない | `.c` |
| `emit-llvm` | LLVM IR | 未指定、または明示的なWindows x64 / Linux x86-64 | `.ll` |
| `emit-qbe` | QBE IR | 未指定、または明示的なLinux x86-64 | `.ssa` |
| `emit-wat` | WebAssembly Text | WebAssembly | `.wat` |
| `emit-asm` | ネイティブアセンブリ | x86-64、Windows / Linux、各OSの呼出規約 | `.s` |
| `emit-obj` | 自前のネイティブオブジェクト | 明示的なWindows x64 / Linux x86-64 | `.obj` / `.o` |
| `emit-bytecode` | Primer bytecode | Primer VM | `.pbc` |

テキストを出力する`emit-*`コマンドは、`-o`を指定しない場合、観測結果を標準出力へ書き出します。`-o`を指定した場合は、利用者が出力先のパスを決定します。バイナリの`emit-obj`では`-o`が必須です。

`emit-asm --target x86_64-unknown-linux-gnu`でLinux、`--target x86_64-pc-windows-msvc`でWindows向けのアセンブリを生成します。省略時は互換性のためWindows固定で、ホストOSから推測しません。`--annotate-origins`で式と命令の対応を注釈とラベルとして残せます。[機械語までの観測](../design/native-code.ja.md)も参照してください。

### LLVMのターゲット指定

`--target`は`x86_64-unknown-linux-gnu`または`x86_64-pc-windows-msvc`を受け付けます。文字列を使うプログラムでは、未使用の型・関数も含めて指定が必須です。数値だけでも、検査付き演算・変換・配列添字など実行時診断を持つ場合は指定が必要です。たとえば`print(1 + 2);`は指定が必要で、`print(1);`は省略できます。ホストOSからは推測しません。`--target`と`-o`（`--output`も可）の順序は自由ですが、同じオプションの重複、値の省略、未対応ターゲットはエラーです。

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

### QBEのターゲット指定

文字列を含むQBE出力では`--target x86_64-unknown-linux-gnu`を指定します。省略・未対応のターゲット・オプションの重複は診断し、既存の出力ファイルを変更しません。数値だけの既存の呼び出しでは省略できます。

```sh
primer emit-qbe examples/string_lookup.prim --target x86_64-unknown-linux-gnu -o target/string_lookup.ssa
qbe -t amd64_sysv -o target/string_lookup.s target/string_lookup.ssa
cc target/string_lookup.s -o target/string_lookup
./target/string_lookup
```

生成物のコメントにターゲットを残します。QBEとCリンカの起動は利用側の操作です。ライブラリでは`compile_to_qbe_with_target(source, Some(codegen::qbe::Target::X86_64UnknownLinuxGnu))`を使います。

### WATと直接アセンブリの文字列

`emit-wat`はWebAssembly固定です。`emit-asm`は明示的にWindows/Linuxを選べ、省略時は従来のWindows固定です。

文字列を使うWATは`primer.write_byte(i32) -> void`をimportし、各バイトと末尾LFを渡します。メモリは公開しません。数値・真偽値の既存のホスト関数も含め、ホストは[文字列の出力契約](../design/strings.ja.md#watの出力と外部との境界)を実装します。`emit-wat`自体はホストを起動しません。

実行時検査を持つWATは`primer.write_error_byte(i32) -> void`もimportします。ホストはこのASCII診断をstderrとして出力し、`unreachable`で停止しても先行stdoutを保持します。[実行時診断の契約](../design/runtime-diagnostics.ja.md)を参照してください。

Windows x64の直接アセンブリは`primer emit-asm examples/string_lookup.prim -o target/string_lookup.s`で生成し、`clang --target=x86_64-pc-windows-msvc target/string_lookup.s -o target/string_lookup.exe`でビルドできます。文字列の出力前に標準出力をバイナリモードへ切り替えます。

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

`run --diagnostic-format runtime-v1`では、言語の検査失敗を停止理由・NodeId・UTF-8バイト範囲を持つ1行の記録で出力します。コンパイル診断やVM内部エラーは従来の形式です。停止前に実行した`print`はstdoutに残します。C・LLVM・QBE・WAT・Windows/LinuxのASMと自前オブジェクトも同じ停止記録を使います。[共通診断の契約](../design/runtime-diagnostics.ja.md)と[意図した停止の例](../../examples/runtime_failures/README.md)を参照してください。

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

## LLVMの出自を辿る

```sh
cargo run -- emit-ir examples/string_origins.prim
cargo run -- emit-llvm examples/string_origins.prim --target x86_64-unknown-linux-gnu --annotate-origins -o string-origins.ll
cargo run -- run examples/string_origins.prim
```

Windows向けには`--target x86_64-pc-windows-msvc`を指定します。`--annotate-origins`はLLVMだけの任意指定です。通常の出力は従来どおりです。APIでは`compile_to_llvm_with_options(source, llvm::Options { target, annotate_origins: true })`を使います。コメントの意味と対応範囲は[可観測性の契約](../design/observability.ja.md#llvmの出自注釈)を参照してください。

## WATのu64出力

`u64`を表示する成果物は`primer.print_u64(i64) -> void`をimportします。ホストは符号なし64ビットの十進数とLFを出力します。JavaScriptでは`BigInt.asUintN(64, value).toString()`を使います。[u64の設計](../design/u64.ja.md)を参照してください。

## 自前オブジェクトの生成

`emit-obj`は`--target x86_64-pc-windows-msvc`または`x86_64-unknown-linux-gnu`と`-o`を必須とします。外部ツールなしでCOFF/ELFを生成し、標準出力へバイナリは書きません。`--annotate-origins`で出自ラベルを残せます。誤ったオプションやコンパイル診断では既存出力を変更しません。リンク・実行は別の明示操作です。[自前エンコーダ](../design/native-encoder.ja.md)を参照してください。ライブラリでは`compile_to_native_object(source, target, annotate_origins)`が`Vec<u8>`を返します。
