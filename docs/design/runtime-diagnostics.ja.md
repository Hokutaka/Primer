# 実行時停止の共通記録

[English](runtime-diagnostics.en.md)

## 目的と今回の対応範囲

失敗を「非ゼロで終了した」とだけ記録すると、言語が意図した停止と、アクセス違反や起動失敗を区別できません。Primerの可観測性として、停止理由と失敗したソース範囲を出力経路間で照合します。内部状態の変更や実行への介入は公開しません。

| 経路 | 現在の実行時診断 |
| --- | --- |
| VM | 既定の人向け診断と、明示的な`runtime-v1`形式 |
| Windows/Linux直接ASM | 言語の検査失敗時に`runtime-v1`をstderrへ出し、不正命令で停止 |
| 自前エンコーダのCOFF/ELF | ASMと同じlowering・診断処理を符号化 |
| C | 既存の診断。一部のu64検査は理由付き。共通形式とソース範囲への対応は未実装 |
| LLVM / QBE / WAT | 既存のtrap / abort / unreachable。共通形式は未実装 |

まず直前に追加した自前エンコーダまでをVMと照合できる状態にしました。次は同じ契約をC・LLVM・QBE・WATへ展開します。全経路の診断が揃ったとは扱いません。

## レコード

```text
primer: runtime-v1 code=division-by-zero node=1 bytes=6..11
```

これは`print(1 / 0);`の除算の例です。形式はASCIIの1行で、次を保持します。

- `code`: 下表の停止理由。
- `node`: 対応するPrimer IRのNodeId。呼び出し先で失敗した場合は、呼び出し元の式ではなく失敗した式を指す。
- `bytes`: 元ソースのUTF-8バイト範囲。0起点、終端を含まない。行番号や表示上の文字数ではない。

パス、ソース本文、実行時の値は含めません。元ソースとコンパイラ版を合わせて解釈します。NodeIdは編集や版の変更をまたぐ永続IDではありません。ネイティブ観測manifestは元ソースのSHA-256と使用ツールも記録します。

| code | 意味 |
| --- | --- |
| `integer-overflow` | 加算・減算・乗算・符号反転・左シフトの結果が型の範囲外 |
| `division-by-zero` | 整数のゼロ除算 |
| `division-overflow` | 整数の除算結果が型の範囲外 |
| `remainder-by-zero` | 整数の剰余演算の除数がゼロ |
| `invalid-shift-count` | シフト量が0未満または型のビット数以上 |
| `integer-conversion-out-of-range` | 整数型間の明示変換で範囲外 |
| `conversion-out-of-range` | 浮動小数点を含む変換で範囲外 |
| `conversion-inexact` | 変換で値が変わる |
| `conversion-not-finite` | NaNまたは無限大を整数へ変換できない |
| `conversion-nan` | floatの型変更でNaNを保持できない |
| `conversion-negative-zero` | 整数への変換で負のゼロの符号を保持できない |
| `array-index-out-of-bounds` | 配列の添字が範囲外 |

配列代入ではNodeIdは代入文、バイト範囲は失敗した添字の`[...]`を指します。入れ子の添字は左から検査し、失敗した場合は右辺を評価しません。整数からfloatへの変換で丸めにより範囲外へ出ても、元の整数値を保存できない理由は`conversion-inexact`です。

この形式はコンパイル診断、壊れたbytecodeの内部エラー、OSの資源不足、出力のI/O失敗を言語の検査失敗として分類しません。VMではそれらに既存の診断を使い、生成物に予期しないクラッシュが起きた場合も共通レコードを捏造しません。

## 使い方と出力の保持

```sh
primer run examples/runtime_failures/function_division.prim --diagnostic-format runtime-v1
```

通常の`run`は従来のソース位置・bytecode位置付き診断を使います。`--diagnostic-format runtime-v1`で言語の検査失敗を共通形式にします。VMは終了コード1です。`ExecutionError::runtime_failure()`で構造化された理由と範囲を取得できます。

失敗より前に実行した`print`は取り消しません。VMは`VmError::output()`にその出力を保持し、CLIもstdoutへ表示します。ネイティブでは失敗時に既存のstdoutバッファをflushしてからstderrへレコードを書き、不正命令`ud2`で停止します。stdoutとstderrを混ぜた表示順までは保証しません。Windowsで文字列を含むプログラムのstdoutは従来通りバイナリ出力です。

書き込みは明示したターゲットのLinux `write` / Windows `_write`を使います。stderrの改行は比較時にCRLF/LFを揃えます。標準エラーが閉じられている、書き込みが途中で失敗する、等の場合にも元の不正命令停止を試みますが、完全な診断の配送までは保証しません。観測ツールは欠けたレコードを合格にしません。

## 実装上の判断

ソース由来の停止箇所ごとに、読み取り専用のレコードを生成します。生成物は大きくなりますが、失敗分岐だけが診断処理を呼び、正常経路に「現在位置」を書き換える可変状態を持ち込みません。レコードのための動的確保や、外部から書き換える診断変数はありません。数値変換で複数の失敗理由を区別するために必要な検査は、演算の意味を保つ検査として常に生成します。

`--annotate-origins`は引き続きコメント・シンボルによる観測だけを切り替えます。実行時診断を無効にはしません。注釈の有無で命令・データの内容と実行結果が一致することをテストします。検査のABIと書き込み処理は実行中のOSから推測せず、生成時のターゲットで決めます。

## 検証

[意図した停止の3例](../../examples/runtime_failures/README.md)で、桁あふれ、入れ子の配列代入、関数内のゼロ除算を辿れます。正常終了のサンプルと混ぜず、期待する停止理由を表で示しています。

`observe-native.cjs --run --expect-trap`はVMを共通形式で実行し、理由・NodeId・バイト範囲・停止前の出力をネイティブと比較します。さらにSIGILL / Windows不正命令終了を確認します。診断だけ一致する通常終了、レコードなしのクラッシュ、追加エラー、タイムアウトは合格にしません。manifestの`runtimeFailure`には照合できたレコードを保持します。

49種類の失敗を既知の理由と比較し、Windows/Linux双方のASMと自前オブジェクトをVMと照合します。Unicode・CRLF、短絡評価、関数の呼び出し、既定フィールドの式、変換の境界値、出力の保持も含みます。通常の50サンプルと全既存テストも検証対象です。
