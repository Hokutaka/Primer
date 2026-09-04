# Primer言語リファレンス

[English](language.en.md)

この文書では、Primer v0.1の構文と意味を定義します。

## 文法

```text
program     := item* EOF

item        := type_definition
             | function_definition
             | statement

type_definition :=
    "type" IDENT "{" field_definition ("," field_definition)* ","? "}"

field_definition := IDENT ":" type_ref ("=" expression)?

function_definition :=
    "fn" IDENT "(" parameters? ")" "->" return_type block

parameters  := parameter ("," parameter)*

parameter   := IDENT ":" type_ref

return_type := type_ref | "void"

statement   := binding
             | assignment
             | "print" "(" expression ")" ";"
             | IDENT "(" arguments? ")" ";"
             | "return" expression? ";"
             | if_statement
             | while_statement
             | for_statement
             | "break" ";"
             | "continue" ";"

if_statement := "if" expression block ("else" block)?

while_statement := "while" expression block

for_statement :=
    "for" "(" (binding_clause | assignment_clause) ";"
    expression ";" assignment_clause ")" block

block       := "{" statement* "}"

binding     := "mut"? IDENT ":" type_spec "=" expression ";"

binding_clause := "mut"? IDENT ":" type_spec "=" expression

assignment  := assignment_clause ";"

assignment_clause := IDENT "=" expression

type_spec   := "i64"
             | "f32"
             | "f64"
             | "bool"
             | fixed_array_type
             | IDENT
             | "infer"

type_ref    := "i64" | "f32" | "f64" | "bool" | fixed_array_type | IDENT

fixed_array_type := "[" ("i64" | "f32" | "f64" | "bool") ";" INTEGER "]"

expression  := equality

equality    := comparison (("==" | "!=") comparison)*

comparison  := additive (("<" | "<=" | ">" | ">=") additive)*

additive    := multiply (("+" | "-") multiply)*

multiply    := unary (("*" | "/") unary)*

unary       := ("-" | "!") unary
             | postfix

postfix     := primary (("." IDENT) | ("[" expression "]"))*

primary     := "true"
             | "false"
             | INTEGER
             | FLOAT
             | "[" expression ("," expression)* ","? "]"
             | IDENT
             | IDENT "(" arguments? ")"
             | IDENT "{" field_value ("," field_value)* ","? "}"
             | "(" expression ")"

arguments   := expression ("," expression)*

field_value := IDENT ":" expression
```

変数束縛は標準では不変です。`mut`を付けた束縛だけが再代入できます。式から参照できるのは、その式より前に宣言された変数束縛だけです。

型の指定は常に必要です。

```primer
count: i64 = 42;
single: f32 = 0.1 + 0.2;
double: f64 = 0.1 + 0.2;
value: infer = count * 2;
```

`infer`は型推論を明示的に要求します。`infer`自体が実行時の型になることはありません。

## 名前付きproduct type

`type`は、複数のfieldを一つの値へまとめる名前付きproduct typeを定義します。型は名前で区別されるため、同じfieldを持つ二つの型も別の型です。

```primer
type Point {
    x: f64 = 0.0,
    y: f64,
}

point: Point = Point {
    y: 2.0,
};

print(point.x);
```

fieldの型は定義時に必ず指定し、`infer`は使用できません。既定値のないfieldは値を作るときに必要です。fieldは名前で指定するため、記述順は型定義と異なっても構いません。末尾のカンマも使用できます。

明示したfieldの式はソースに書いた順番で評価します。その後、省略したfieldの既定値を型定義順で評価します。Primer IRでは、この順番と、値が明示されたか既定値から来たかを観測できます。

field accessには`.`を使い、`segment.start.x`のように入れ子にできます。fieldは直接変更できません。変更するときは新しい値を作り、`mut`な束縛へ全体を再代入します。

```primer
mut point: Point = Point { x: 1.0, y: 2.0, };
point = Point { x: 3.0, y: point.y, };
```

積値を別の束縛へ入れた後で元の束縛へ再代入しても、先の値は変化しません。これは言語上の値としての規則です。各backendが物理的にどう配置・コピーするかは、生成物で観測できます。

`if`と`while`の条件の直後にある`{`はブロックの開始として読みます。構築式から得たfieldを条件にする場合は、構築式を丸括弧で囲みます。

```primer
type Flags { enabled: bool, }

if (Flags { enabled: true, }).enabled {
    print(true);
}
```

空のproduct type、空の構築式、無限サイズになる値による再帰型、積値どうしの比較、積値そのものの`print`は現在サポートしません。

詳細な設計と各backendの表現は[名前付きproduct typeの設計](../design/product-types.ja.md)で説明します。

## 固定長配列

固定長配列は、同じ型の箱を決まった数だけ横に並べた値です。`[i64; 4]`は「`i64`の箱が4個ある配列」という意味です。

```primer
values: [i64; 4] = [2, 4, 6, 8];
print(values[2]);
```

長さは型の一部です。したがって、`[i64; 3]`と`[i64; 4]`は別の型です。配列リテラルの要素数と宣言した長さが違う場合や、要素の型がそろわない場合はエラーになります。空の配列は、要素型を決められないため現在は使えません。

添字は`i64`で、先頭は`0`です。上の例の`values[2]`は3番目の値`6`を読み取ります。負の添字や長さ以上の添字は、Primer VMと生成されたプログラムのどちらでも実行時に停止します。境界検査は各backendの生成物にも残るため、どこで安全を確かめているか観測できます。

配列は一つの値としてコピーされます。コピーした後で元の`mut`束縛へ新しい配列を再代入しても、先に作ったコピーは変わりません。

```primer
mut first: [i64; 2] = [10, 20];
second: [i64; 2] = first;
first = [30, 40];
print(second[0]); // 10
```

現在の要素型は`bool`、`i64`、`f32`、`f64`です。配列の入れ子、product typeのfield、関数のparameterや戻り値にはまだ配列を使えません。`values[0] = 1;`のような要素への直接代入も未対応です。これらは黙って別の意味にせず、診断として報告します。

詳しい設計と各backendでの境界検査は[固定長配列の設計](../design/fixed-arrays.ja.md)で説明します。

## 関数とentry point

`fn`は、名前を付けた処理を定義します。parameterと戻り値の型は必ず書きます。

```primer
fn add(left: i64, right: i64) -> i64 {
    return left + right;
}

answer: i64 = add(20, 22);
```

値を返す関数では`return expression;`を明示します。blockの最後に式を書いても、暗黙の戻り値にはなりません。どの経路でも値を返すと確認できない場合はエラーになります。

値を返さない関数は`-> void`と書きます。blockの最後まで進むか、`return;`で途中終了できます。値を返す関数の呼び出し結果は式として使い、`void`関数は文として呼び出します。

```primer
fn show(value: i64) -> void {
    print(value);
}

show(answer);
```

関数名はファイル全体で解決するため、定義より前から呼び出せます。parameterと関数内の束縛は、その関数の外から見えません。関数からトップレベルの実行時束縛を読むこともできません。

トップレベルに実行文がある場合、compilerがentry pointを生成します。代わりに`fn main() -> void`を明示できますが、明示的な`main`とトップレベル実行文は同時に書けません。`main`はparameterを受け取れません。

現在の関数シグネチャは`bool`、`i64`、`f32`、`f64`に限り、parameterは最大4個です。名前付きproduct typeや固定長配列の受け渡し、再帰、command-line argumentはまだサポートしません。未対応の書き方は黙って別の意味にせず、診断します。

Primer IRとbytecodeでは、関数ID、parameterの束縛ID、呼び出し、戻り値を観測できます。各backendの成果物では、これらが関数シンボル、引数、ローカル領域、ABI上のレジスタやメモリへ変わった結果を観測できます。詳細は[関数の設計](../design/functions.ja.md)を参照してください。

## 可変な束縛と再代入

変更が必要な束縛は、宣言の先頭へ`mut`を付けます。

```primer
mut count: i64 = 40;
count = count + 2;
print(count);
```

`mut`は型ではなく、名前`count`へ再代入できることを指定します。再代入では`: type_spec`を書きません。この違いにより、新しい宣言と既存の束縛への代入を区別します。

`mut`のない束縛へ再代入すると型検査エラーになります。

```primer
count: i64 = 40;
count = 42; // エラー
```

再代入する値の型は、宣言時に解決された型と一致する必要があります。`infer`を使用した場合も、型を推論するのは宣言時だけです。

```primer
mut value: infer = 1; // i64へ解決
value = 2;            // OK
value = 0.5;          // エラー
```

Primer IRでは、最初の初期化と再代入を別の文として保持します。bytecodeでも初期化の`store`と再代入の`assign`を区別します。

## 条件分岐、繰り返し、ブロックスコープ

`if`は`bool`の条件に従って文を実行します。`else`は省略できます。

```primer
if value < 10 {
    print(value);
} else {
    print(10);
}
```

条件が`bool`でなければ型検査エラーになります。`if`は現在、値を作る式ではありません。

`while`は条件が`true`である間、本文を繰り返します。条件は本文を実行する前に毎回評価されるため、最初から`false`なら本文は一度も実行されません。

```primer
mut count: i64 = 0;

while count < 3 {
    print(count);
    count = count + 1;
}
```

`while`の条件にも`bool`が必要です。`while`も値を作る式ではありません。

`for`は、開始文、`bool`の継続条件、更新文、本文を一つにまとめます。開始文には新しい束縛か、既存の`mut`な束縛への再代入を書けます。

```primer
mut sum: i64 = 0;

for (mut i: i64 = 0; i < 6; i = i + 1) {
    sum = sum + i;
}
```

開始文は最初に一度だけ実行します。各回の本文を実行する前に継続条件を評価し、本文を最後まで実行した後に更新文を実行してから、もう一度継続条件を評価します。現在の構文では、開始文、継続条件、更新文をすべて省略できません。

開始文で宣言した束縛は、継続条件、更新文、本文から参照できますが、`for`の後からは参照できません。開始文が既存の束縛への再代入なら、その束縛は`for`の後にも残ります。本文は、その内側に別のブロックスコープを作ります。

`break;`は最も内側のループを終了します。`while`内の`continue;`は条件評価へ直接進みます。`for`内では更新を実行してから条件評価へ進みます。どちらもループの外では使用できません。

```primer
while value < 10 {
    value = value + 1;

    if value < 3 {
        continue;
    }

    if value > 5 {
        break;
    }
}
```

現在は外側のループを名前で指定するラベル付きの`break`や`continue`を持ちません。

波括弧で囲まれた各ブロックは新しいスコープを作ります。ブロック内の束縛は外側から見えません。内側から外側の束縛を参照でき、`mut`なら再代入もできます。

内側のブロックでは、外側と同じ名前を別の束縛として宣言できます。

```primer
mut value: i64 = 1;

if true {
    value = 2;          // 外側のvalueを更新
    value: bool = true; // 内側だけの別のvalue
    print(value);       // boolのvalue
}

print(value);           // i64のvalue
```

Primer IRは各束縛へ決定的なIDを付け、同じ名前でもどの宣言を参照したか区別します。構造化された`if`、`while`、`for`、`break`、`continue`はPrimer IRに残ります。`for`は初期化、条件、本文、更新を別々に保持します。Bytecodeと各バックエンドIRへのlowering時に、構造化されたループは条件、本文、必要な更新、終了の経路へ変換されます。`break`と`continue`は、対象ループ内の正しい経路へのジャンプへ変換されます。

## 型

Primer v0.1には、一つの真偽値型、三つの数値型、固定長配列、ユーザーが定義する名前付きproduct typeがあります。

```text
bool
i64
f32
f64
固定長配列
名前付きproduct type
```

バックエンドへのlowering時に、各バックエンドがこれらの型を自身の表現へ対応付けます。

たとえば、Cバックエンドでは次のように対応付けます。

```text
Primer    C
bool      bool
i64       int64_t
f32       float
f64       double
```

## 真偽値と比較

`bool`には`true`と`false`の二つの値があります。`!`は値を反転します。

```primer
enabled: bool = true;
disabled: bool = !enabled;
```

`==`と`!=`は、同じ型の値どうしを比較できます。数値型では、さらに`<`、`<=`、`>`、`>=`を使用できます。比較結果の型は常に`bool`です。

```primer
same: bool = enabled == true;
small: bool = 1 + 2 < 4;
different: bool = 0.1f32 != 0.2f32;
```

`bool`の大小比較と算術演算はできません。また、比較でも暗黙の数値変換は行いません。

## 数値リテラル

整数リテラルの型は`i64`です。

```primer
x: i64 = 42;
```

接尾辞のない浮動小数点リテラルは、明示的な浮動小数点型を文脈から得られる場合、その型として扱います。

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
```

この違いは、バックエンドへのloweringより前にPrimer IRで解決されます。

たとえば、Cバックエンドは次のように出力できます。

```c
float primer_a = (0.1f + 0.2f);
double primer_b = (0.1 + 0.2);
```

期待される浮動小数点型がない場合、接尾辞のない浮動小数点リテラルは`f64`になります。

```primer
x: infer = 0.1 + 0.2;
```

この場合、`x`は`f64`として推論されます。

リテラルの接尾辞で型を明示できます。

```primer
a: infer = 0.1f32 + 0.2f32;
b: infer = 0.1f64 + 0.2f64;
```

浮動小数点リテラルでは科学的記数法も使用できます。

```primer
x: f64 = 1.5e-3;
```

## 型検査

現在の算術演算では、左右のオペランドが同じ型である必要があります。

```text
i64 op i64 -> i64
f32 op f32 -> f32
f64 op f64 -> f64
```

Primer v0.1は暗黙の数値変換を行いません。

たとえば、次の式は左右のオペランドが`i64`と`f64`であるため型エラーになります。

```primer
x: infer = 1 + 0.1;
```

変数束縛に明示した型は、解決された式の型と一致するか検査されます。

```primer
x: f32 = 0.1 + 0.2;
```

この例では、変数束縛に指定した`f32`が、接尾辞のない浮動小数点リテラルに期待される型を与えます。そのため式全体が`f32`として扱われます。

この決定はPrimer IRに記録され、各バックエンドで繰り返し計算されることはありません。

比較演算でも左右のオペランドは同じ型である必要があります。比較する値の型と、結果の`bool`はPrimer IRで別々に観測できます。

## 出力

`print(expression);`は、現在の真偽値型と数値型を受け付けます。名前付きproduct typeはfieldを、固定長配列は要素を指定して出力します。

Primerは、観測対象となる浮動小数点数の挙動が見えるだけの精度を保って出力します。

現在の出力形式は次のとおりです。

```text
bool   `true`または`false`
i64    整数として出力
f32    有効数字9桁
f64    有効数字17桁
```

たとえば、次の`f32`の計算では、短く丸められた小数ではなく浮動小数点数による近似値が見える場合があります。

```primer
x: f32 = 0.1 + 0.2;
print(x);
```

各バックエンドは、自身の対象環境の規約に従って実装しながら、この観測可能な出力の振る舞いを保つ責任を持ちます。
