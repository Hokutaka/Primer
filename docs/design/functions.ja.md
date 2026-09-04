# 関数の設計

[English](functions.en.md)

Status: Implemented

この文書は、Primerの関数、呼び出し、戻り値、entry pointの設計判断と観測上の約束を記録します。現在の構文は[言語リファレンス](../reference/language.ja.md)で定義します。

## 目的

関数は処理を再利用するためだけの機能ではありません。Primerでは、一つの処理がソースからPrimer IR、backend IR、成果物へ変わる単位でもあります。

守ることは次の三つです。

- 関数名、parameter、呼び出し、戻り値の対応を変換後も追える
- backendのABIをPrimerの意味と混ぜない
- 観測用の識別子からcompilerや実行中の値を変更できない

## 現在の意味

```primer
fn add(left: i64, right: i64) -> i64 {
    return left + right;
}
```

- `fn`で関数定義を明示する
- parameterと戻り値は具体型を明示する
- 戻り値がなければ`void`を明示する
- 値を返すときは`return expression;`を明示する
- parameterは不変のローカル束縛として扱う
- 関数はトップレベルの実行時束縛を参照しない
- 関数名はファイル全体で先に登録し、forward callを認める

明示的な`main`は`fn main() -> void`だけを認めます。トップレベル実行文があるプログラムではcompilerがentry pointを生成するため、両方は同時に使えません。

## 観測できるもの

| 段階 | 保持する情報 |
| --- | --- |
| AST | ソース上の名前、型名、block、Span |
| Primer IR | `FunctionId`、`BindingId`、解決済み型、構造化されたcallとreturn |
| Bytecode | 関数番号、parameter slot、関数内命令番号、callの引数数、returnの有無 |
| Backend IR | backend固有のlocal、temporary、control flow、呼び出し表現 |
| 成果物 | 関数symbol、引数表現、stackやmemory、実際のcallとreturn |

VMエラーは、関数番号とその関数内の命令番号を保持します。命令の出自からソースのSpanへ戻れるため、関数内の実行時エラーもentry pointの命令と混同しません。

## Backendの境界

- Cは型付き関数とprototypeを生成する
- LLVM IRは関数parameterを明示し、観測しやすいlocal slotへ保存する
- QBE IRは関数parameterとstack allocationを明示する
- WebAssembly Textはtyped parameter/resultと`call`を生成する
- Windows x86-64はWindows x64 ABIに従い、引数位置を汎用registerまたはXMM registerへloweringする
- Primer bytecodeとVMは呼び出しごとに独立したframeを作る

Primer IRはABI registerやstack offsetを決めません。これらはbackend loweringの判断であり、成果物で観測します。

## 現在の制約

関数シグネチャはscalar型に限り、parameterは最大4個です。これは最初のABI実装をすべての出力経路で一致させるための制約です。関数本体では名前付きproduct typeをローカル値として使用できます。

再帰は直接呼び出しと間接呼び出しの両方を診断します。現在のWebAssembly backendではproduct型の一時memoryを呼び出しごとに分離していないためです。再帰を許可すると、一部の出力経路だけで値が壊れる可能性があります。

再帰を追加するときは、少なくとも次を同時に満たします。

- すべてのbackendで呼び出しframeが独立する
- product型のlocalとtemporaryも呼び出しごとに独立する
- 観測結果からcall stackと出自を区別できる
- stackまたはmemory消費へ明示的な上限を設けられる

## セキュリティ境界

`FunctionId`、`BindingId`、命令番号は、一回のcompile結果の関係を読むための識別子です。関数を差し替えるhandle、実行中のframeを変更する参照、compiler内部へ書き戻す権限ではありません。

将来、外部関数やpluginを追加する場合も、観測APIと実行・変更権限は別に設計します。観測機能を有効にしただけで呼び出し先や生成結果が変わってはいけません。
