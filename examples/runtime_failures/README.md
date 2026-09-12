# 意図した実行時停止の例

[English](README.en.md)

この4例は正常終了せず、指定した理由で停止することを確認します。ルートの正常終了サンプル一括実行には含めません。

| 例 | 停止前の出力 | 停止理由・観測する箇所 |
| --- | --- | --- |
| [overflow.prim](overflow.prim) | `カウンタを更新` | `integer-overflow`、`counter + 1` |
| [array_update.prim](array_update.prim) | `4` | `array-index-out-of-bounds`、代入先の`[2]`。右辺を実行しない |
| [function_division.prim](function_division.prim) | `false`、`除算を開始` | `division-by-zero`、関数内の`value / divisor`。短絡した最初の呼び出しは実行しない |
| [call_sequence.prim](call_sequence.prim) | `除算する値`、`2`、`5`、`false`、`除算する値`、`0` | `division-by-zero`、同じ関数の`10 / value`。正常な呼び出しの後に短絡した呼び出しを飛ばし、最後の呼び出しで停止 |

```sh
cargo run -- run examples/runtime_failures/array_update.prim --diagnostic-format runtime-v1
```

VMは終了コード1で停止し、stderrに`primer: runtime-v1 code=... node=... bytes=.....`を1行出します。`bytes`は元ソースのUTF-8バイト範囲で、終端は含みません。NodeIdと合わせて`emit-ir`へ辿れます。CRLF/LFを変更するとバイト位置も変わります。

同じソースをC・LLVM・QBE・WATでも生成・実行し、このレコードと停止前の出力を比較するテストは`cargo test --test runtime_routes`です。外部ツールの指定は[診断の検証手順](../../docs/design/runtime-diagnostics.ja.md#検証)を参照してください。未設定で利用できない経路は実行済みと数えません。WATホストには`primer.write_error_byte`が必要です。

Windowsで自前エンコーダと比較する例です。出力ディレクトリには未作成の場所を指定します。

```powershell
node scripts/observe-native.cjs --source examples/runtime_failures/array_update.prim --target x86_64-pc-windows-msvc --primer target/debug/primer.exe --cc clang --objdump llvm-objdump --output-dir target/observe-array-failure --encoder primer --run --expect-trap
```

Linuxではターゲットを`x86_64-unknown-linux-gnu`、Primerを`target/unix/debug/primer`、外部ツールを`cc`と`objdump`へ明示的に変更します。`--encoder external`なら外部アセンブラで比較できます。成功判定には、停止理由・NodeId・バイト範囲・停止前の出力の一致と、ネイティブの意図したillegal-instruction停止をすべて要求します。詳細は[実行時診断の契約](../../docs/design/runtime-diagnostics.ja.md)を参照してください。
