# 意図した実行時停止の例

[English](README.en.md)

この3例は正常終了せず、指定した理由で停止することを確認します。ルートの正常終了サンプル一括実行には含めません。

| 例 | 停止前の出力 | 停止理由・観測する箇所 |
| --- | --- | --- |
| [overflow.prim](overflow.prim) | `カウンタを更新` | `integer-overflow`、`counter + 1` |
| [array_update.prim](array_update.prim) | `4` | `array-index-out-of-bounds`、代入先の`[2]`。右辺を実行しない |
| [function_division.prim](function_division.prim) | `false`、`除算を開始` | `division-by-zero`、関数内の`value / divisor`。短絡した最初の呼び出しは実行しない |

```sh
cargo run -- run examples/runtime_failures/array_update.prim --diagnostic-format runtime-v1
```

VMは終了コード1で停止し、stderrに`primer: runtime-v1 code=... node=... bytes=.....`を1行出します。`bytes`は元ソースのUTF-8バイト範囲で、終端は含みません。NodeIdと合わせて`emit-ir`へ辿れます。CRLF/LFを変更するとバイト位置も変わります。

Windowsで自前エンコーダと比較する例です。出力ディレクトリには未作成の場所を指定します。

```powershell
node scripts/observe-native.cjs --source examples/runtime_failures/array_update.prim --target x86_64-pc-windows-msvc --primer target/debug/primer.exe --cc clang --objdump llvm-objdump --output-dir target/observe-array-failure --encoder primer --run --expect-trap
```

Linuxではターゲットを`x86_64-unknown-linux-gnu`、Primerを`target/unix/debug/primer`、外部ツールを`cc`と`objdump`へ明示的に変更します。`--encoder external`なら外部アセンブラで比較できます。成功判定には、停止理由・NodeId・バイト範囲・停止前の出力の一致と、ネイティブの意図したillegal-instruction停止をすべて要求します。詳細は[実行時診断の契約](../../docs/design/runtime-diagnostics.ja.md)を参照してください。
