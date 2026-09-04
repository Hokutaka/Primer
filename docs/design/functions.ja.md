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
- 名前付きproduct typeと固定長配列もparameterや戻り値に使える
- 関数はトップレベルの実行時束縛を参照しない
- 関数名はファイル全体で先に登録し、forward callを認める

明示的な`main`は`fn main() -> void`だけを認めます。トップレベル実行文があるプログラムではcompilerがentry pointを生成するため、両方は同時に使えません。

## 関数をまたぐ値

scalar、名前付きproduct type、固定長配列は、すべて値として関数をまたぎます。

```primer
type Point { x: i64, y: i64, }

fn move_x(point: Point, amount: i64) -> Point {
    return Point { x: point.x + amount, y: point.y, };
}

original: Point = Point { x: 2, y: 3, };
moved: Point = move_x(original, 5);
print(original.x); // 2
print(moved.x);    // 7
```

`move_x`が受け取る`point`は、呼び出した側の`original`とは別の値です。関数から返るproduct typeや配列も、呼び出した側の新しい値になります。関数の内外で、同じ変更可能な場所を知らないうちに共有することはありません。

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

以下では、複数の値をまとめたproduct typeと固定長配列を「集約値」と呼びます。

- Cは型付き関数とprototypeを生成し、Cの値として受け渡す
- LLVM IRは型付きparameterと戻り値を生成し、受け取った値を観測しやすいlocal slotへ保存する
- QBE IRは集約引数の場所を受け取り、関数の開始時に自身のstack領域へコピーする。集約戻り値には、呼び出し側が用意した戻り先を隠れた先頭引数として使う
- WebAssembly TextはQBEと同じ形をlinear memory上のaddressで表す
- Windows x86-64はscalar引数をWindows x64 ABIの汎用registerまたはXMM registerへloweringする。集約引数の場所は引数位置に対応する汎用registerで渡し、集約戻り値の戻り先は内部規約として`RAX`で渡す
- Primer bytecodeとVMは呼び出しごとに独立したframeを作り、集約値を複製して渡す

Primer IRはABI register、stack offset、隠れた戻り先を決めません。これらはbackend loweringの判断であり、成果物で観測します。QBE、WebAssembly、Windows x86-64が内部でaddressを渡しても、それは値をコピーするための実装です。Primerの参照型や外部ABIを定義するものではありません。

## 現在の制約

parameterは最大4個です。scalar、名前付きproduct type、固定長配列をparameterと戻り値に使えます。

再帰は直接呼び出しと間接呼び出しの両方を診断します。現在のWebAssembly backendではproduct型の一時memoryを呼び出しごとに分離していないためです。再帰を許可すると、一部の出力経路だけで値が壊れる可能性があります。

再帰を追加するときは、少なくとも次を同時に満たします。

- すべてのbackendで呼び出しframeが独立する
- product型のlocalとtemporaryも呼び出しごとに独立する
- 観測結果からcall stackと出自を区別できる
- stackまたはmemory消費へ明示的な上限を設けられる

## セキュリティ境界

`FunctionId`、`BindingId`、命令番号は、一回のcompile結果の関係を読むための識別子です。関数を差し替えるhandle、実行中のframeを変更する参照、compiler内部へ書き戻す権限ではありません。

生成物に現れる集約値のaddressも同様です。Primerのプログラムはそのaddressを値として取り出したり、後で使うために保持したりできません。受け取った関数は最初に自身の領域へ値をコピーするため、呼び出し側の値を書き換える経路にもなりません。

将来、外部関数やpluginを追加する場合も、観測APIと実行・変更権限は別に設計します。観測機能を有効にしただけで呼び出し先や生成結果が変わってはいけません。
