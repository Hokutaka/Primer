# モジュールを使う例

[English](README.en.md)

| ファイル | 型・表現 | 確認内容 |
| --- | --- | --- |
| [values.prim](values.prim) | `u64`・`string`・product・公開関数 | 公開型の既定値と、公開関数から呼ぶ非公開補助関数 |
| [main.prim](main.prim) | `values::Reading`・固定配列・コピー・短絡評価 | 最大u64、再代入後の独立性、文字列バイトと等値比較、実行しない右辺 |
| [single.prim](single.prim) | 同じ処理を一つのファイルで表現 | 分割前後で値と出力順が変わらないこと |
| [failure.prim](failure.prim) | 正常呼び出し後のゼロ除算 | 停止前の出力と、`values.prim`の除算への出自 |
| [array_update.prim](array_update.prim) | 配列代入の左辺検査と公開関数の呼び出し | 添字が範囲外なら右辺の関数を実行しない |

```sh
cargo run -- run examples/modules/main.prim
cargo run -- run examples/modules/single.prim
cargo run -- run examples/modules/failure.prim --diagnostic-format runtime-v1
cargo run -- emit-sources examples/modules/main.prim -o target/module-sources.json
```

正常な2例のstdoutをエスケープ表記すると、`18446744073709551615\n2\n観測\0\r\n\ntrue\nfalse\n計算\n5\n`です。`values.prim`を読んだだけでは`announce`は実行されず、`divide`を実際に呼んだときだけ「計算」が出ます。

`failure.prim`は意図した失敗例です。先行出力`開始\n計算\n5\n計算\n`の後、終了コード1になり、`code=division-by-zero`と`file=2`を記録します。人向け診断では`values.prim:17:12`、本文では`value / divisor`に対応します。NodeIdとバイト位置は編集・改行方式により変わります。成功・意図した停止・想定外の失敗は分けて確認します。

`array_update.prim`も意図した失敗例で、`添字\n`の後に`array-index-out-of-bounds`・`file=1`を報告します。入口ファイルの`[1]`で止まるため、右辺の`divide`は実行されず「計算」は出ません。

`primer emit-c`、`emit-llvm`、`emit-qbe`、`emit-wat`、`emit-asm`、`emit-obj`にも同じ入口を渡せます。LLVM/QBE/ネイティブオブジェクトのターゲット指定は従来通り必要です。`cargo test --test modules --test source_files -- --nocapture`で構文・公開範囲と、利用可能な生成経路の実行結果を確認できます。

ここにある部品・失敗例はルートの正常example一括実行とは別に扱います。[モジュールの規則](../../docs/design/modules.ja.md)と[CLI](../../docs/reference/cli.ja.md)も参照してください。
