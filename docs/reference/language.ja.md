# Primer言語リファレンス

[English](language.en.md)

この文書では、Primer v0.1の構文と意味を定義します。

## 文法

```text
program     := statement* EOF

statement   := binding
             | assignment
             | "print" "(" expression ")" ";"
             | if_statement
             | while_statement
             | for_statement
             | "break" ";"
             | "continue" ";"

if_statement := "if" expression block ("else" block)?

while_statement := "while" expression block

for_statement := "for" binding expression ";" assignment_clause block

block       := "{" statement* "}"

binding     := "mut"? IDENT ":" type_spec "=" expression ";"

assignment  := assignment_clause ";"

assignment_clause := IDENT "=" expression

type_spec   := "i64"
             | "f32"
             | "f64"
             | "bool"
             | "infer"

expression  := equality

equality    := comparison (("==" | "!=") comparison)*

comparison  := additive (("<" | "<=" | ">" | ">=") additive)*

additive    := multiply (("+" | "-") multiply)*

multiply    := unary (("*" | "/") unary)*

unary       := ("-" | "!") unary
             | primary

primary     := "true"
             | "false"
             | INTEGER
             | FLOAT
             | IDENT
             | "(" expression ")"
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

`for`は、束縛、`bool`の条件、再代入、本文を一つにまとめます。

```primer
mut sum: i64 = 0;

for mut i: i64 = 0; i < 6; i = i + 1 {
    sum = sum + i;
}
```

初期化は最初に一度だけ実行します。各回の本文を実行する前に条件を評価し、本文を最後まで実行した後に更新を行ってから、もう一度条件を評価します。現在の構文では、初期化、条件、更新をすべて省略できません。

初期化で宣言した束縛は、条件、更新、本文から参照できますが、`for`の後からは参照できません。本文は、その内側に別のブロックスコープを作ります。

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

Primer v0.1には、一つの真偽値型と三つの数値型があります。

```text
bool
i64
f32
f64
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

`print(expression);`は、現在のすべての具体的な型を受け付けます。

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
