# ファイルごとの出自を観測する例

[English](README.en.md)

モジュール化の最初の段階として、ファイルを分けても正しい式に戻れるか検証します。この低水準API例自体はimportを使いません。実際のimportを使う例は[modules](../modules/README.md)にあります。次のRust exampleが、各Primerファイルを別々に解析して共通IRを作ります。

```sh
cargo run --example source_files
cargo run --example source_files -- failure
```

二つ目は意図したゼロ除算で終了コード1になります。成功扱いにせず、停止理由・先行出力・定義ファイルの位置を確認します。これらの`.prim`を個別に`primer run`へ渡しても、別ファイルの定義は読み込まれません。

| ファイル | 型・表現 | 確認すること |
| --- | --- | --- |
| [values.prim](values.prim) | `u64`・`string`・product・関数 | 定義元の位置、文字列の既定値、関数内の除算 |
| [main.prim](main.prim) | `[Reading; 1]`・コピー・短絡評価 | 最大u64の保持、再代入後の独立性、実行しない右辺 |
| [failure.prim](failure.prim) | 正常呼び出しの後のゼロ除算 | 呼び出し元ではなく`values.prim`の`value / divisor`を指す |

正常例のstdoutは、バイト列をエスケープ表記すると次の通りです。

```text
18446744073709551615\n2\n観測\0\r\n\nfalse\n計算\n5\n
```

失敗例の先行出力は`開始\n計算\n5\n計算\n`です。構造化記録は`code=division-by-zero`、`file=1`を持ち、人向け表示では`values.prim:13:12`へ解決します。NodeIdとバイト位置はソースに対応するため、改行方式や編集によって変わり得ます。

`cargo test --test source_files -- --nocapture`で、ツールがある経路を実行比較できます。分割ASTと単一ソースの両方を比較し、字句・構文・型エラーや、不正な添字で右辺が実行されない場合も確認します。詳細は[設計](../../docs/design/source-files.ja.md)にあります。
