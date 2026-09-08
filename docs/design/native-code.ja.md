# ASMから機械語までを辿る

[English](native-code.en.md)

## 言語の意味と実行環境

Linux x86-64直接アセンブリにも、既存の整数8型（u64を含む）、f32/f64、真偽値、文字列、配列、構造体、関数、制御構文、正確な変換を実装しました。Windows版と`codegen::x86_64`の命令表現・演算処理を共有し、ターゲットによって呼出規約、標準出力、セクション配置、スタック確保を選びます。

`emit-asm`は`--target x86_64-unknown-linux-gnu`と`--target x86_64-pc-windows-msvc`を受け付けます。省略時は従来どおりWindows固定です。実行中のOSから生成先を決めません。

SysVでは整数と浮動小数点の引数レジスタを別々に割り当て、printfへの可変長呼び出しではALにXMM引数の数を渡します。Windowsでは位置に対応するレジスタ、shadow space、必要時の`__chkstk`を使います。Linuxでも大きなスタック確保は各ページへ触れてガード領域を飛び越しません。構造体と配列は独立した値コピーで受け渡します。集約戻り値の保存先はPrimer内部規約でRAXへ渡し、外部C ABI互換性は保証しません。

文字列は長さ付きの不変な静的データです。Linuxはバイト値をputcharへ渡し、Windowsは標準出力をバイナリモードへ切り替えてから出力します。日本語・NUL・CR/LFを保持し、正規化しません。

## 観測の各段階

```text
Primerソース → Primer IR → 共通x86-64命令表現
                                 ↓
                           ASM（出自注釈）
                                 ↓ 自前エンコーダ、または明示した外部アセンブラ
                    ELF/COFFオブジェクト（命令バイト・再配置）
                                 ↓ 明示した外部リンカ
                      実行ファイル（機械語）→ 実行比較
```

機械語の経路は言語の意味を実装し直さず、同じASMから構築します。符号化・オブジェクト生成は`--encoder primer`で[自前エンコーダ](native-encoder.ja.md)、既定の`--encoder external`で外部アセンブラを選びます。リンクは明示した外部ツールの担当です。`emit-*`は引き続き成果物を返すだけで、外部ツールの起動は明示したスクリプトに分離します。

`--annotate-origins`は`# primer-asm-origins v1`、`# primer-origin: #N bytes start..end`と`primer_origin_nN_...`ラベルを加えます。NodeIdとUTF-8バイト範囲はlowererが保持します。定数・補助関数・起動処理などはsyntheticです。ラベルはオブジェクトのシンボルとして残り、逆アセンブルの位置とIRの式を対応付けます。注釈を取り除いたASMは通常出力と一致します。観測はメモリの公開や外部からの書き換え機能を追加しません。

## 実行手順

LinuxでRust、Node、cc、objdumpが利用できる場合:

```sh
cargo build
node scripts/observe-native.cjs --source examples/native_values.prim --target x86_64-unknown-linux-gnu --primer target/debug/primer --cc cc --objdump objdump --output-dir target/native-demo-linux --run
```

WindowsでClang、MSVC CRT/リンカ、Node、llvm-objdumpが利用できる場合:

```powershell
cargo build
node scripts/observe-native.cjs --source examples/native_values.prim --target x86_64-pc-windows-msvc --primer target/debug/primer.exe --cc clang --objdump llvm-objdump --output-dir target/native-demo-windows --run
```

WSLで`CARGO_TARGET_DIR=target/unix`を使う場合は、`--primer target/unix/debug/primer`へ変更します。出力先は新規ディレクトリに限定し、親ディレクトリは先に用意します。同名の既存成果物を上書きしません。`--run`を省略すると生成と観測だけを行います。実行はホストと指定ターゲットが一致するときに限ります。アセンブラが出したオブジェクト形式も検査します。

| 成果物 | 読めるもの |
| --- | --- |
| `source.prim` / `program.pir` | 入力と型・式・NodeId |
| `program.s` | 出自とターゲットに対応するASM |
| `program.o` / `program.obj` | 命令バイトと未解決の再配置を持つオブジェクト |
| `object.txt` / `text.txt` | セクション・シンボル・再配置・逆アセンブルと.textのバイト列 |
| `program` / `program.exe`、`executable.txt` | リンク済み実行ファイルと逆アセンブル |
| `vm.stdout` / `native.stdout`、各stderr | `--run`で取得した出力 |
| `manifest.json` | スキーマ、エンコーダ、ターゲット、ツールと版、実行引数、各段階の成否、成果物のSHA-256 |

オブジェクトの位置はセクション内オフセット、実行ファイルの位置はリンク時のアドレスです。実行時ASLRのアドレスではありません。リンカがローカルシンボルを落とす場合は、実行ファイル側まで出自名が残るとは限りません。オブジェクト段階での対応を保存します。外部ツールの版・形式・リンク環境が異なる場合、バイナリ全体の一致は保証しません。

## 成功と想定した停止を分ける

通常実行はVMとネイティブの終了成功、stderrが空であること、stdoutの一致を確認します。文字列はバイト単位で比較します。Windowsの数値のみの既存CRT出力はCRLFをLFへ揃え、その比較条件をmanifestに明記します。

`--run --expect-trap`は異常系用です。VMとネイティブの`runtime-v1`記録（理由・NodeId・バイト範囲）、停止前のstdoutの一致、およびネイティブのSIGILL／Windows不正命令終了を要求します。成功実行、アクセス違反、起動失敗、タイムアウトは合格にしません。manifestでは`output-matched`、`expected-failure-confirmed`、`generated-not-executed`、`failed`を分け、照合できた記録を`runtimeFailure`へ残します。[共通診断の契約](runtime-diagnostics.ja.md)を参照してください。各ツールの時間上限は30秒です。

Cのu64検査は失敗理由をstderrへ出し、テストでVMのエラー種別に対応する診断を照合します。Windowsではabortと他のfast-failが同じ終了コードを使い得るため、コードだけでは合格にしません。[Microsoftのabort仕様](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/abort)と[fast-fail仕様](https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail)も参照してください。QBEはSIGABRT、LLVMは不正命令、WATはunreachableを確認します。C・LLVM・QBE・WATへの共通記録の展開は今後の課題です。

`cargo test --test native_assembly`で全example、文字列・u64境界値、混在する4引数、大きなスタック、コピー、出自、期待する停止を検証します。機械語経路のテストではNode・Cドライバ・objdumpが必要です。`PRIMER_TEST_NODE`、`PRIMER_TEST_CC`（Linux）、`PRIMER_TEST_ASM_CLANG`（Windows）、`PRIMER_TEST_OBJDUMP`で指定できます。CIでも両OSの実行を必須にしています。
