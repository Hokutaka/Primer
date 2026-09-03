# 名前付きproduct typeの設計

[English](product-types.en.md)

**状態: 実装済み**

この文書は、Primerへ最初に導入したユーザー定義型として、名前付きproduct typeの意味、構文、可観測性、実装境界を整理します。

言語として利用するための規則は[言語リファレンス](../reference/language.ja.md)で定義します。この文書では、その規則を選んだ理由と生成物への変換も説明します。

全体のコンパイラ構成は[コンパイラ設計](architecture.ja.md)、将来の開発順序は[コンパイラ進化計画](evolution-plan.ja.md)、秘密を含む値は[Secret値の設計](secrets.ja.md)で扱います。

## 目的

product typeは、複数の値を名前付きのfieldへまとめ、一つの意味を持つ値として扱うための型です。

```primer
type Point {
    x: f64,
    y: f64,
}
```

Primerでは、単に値をまとめられることに加えて、次を観測できる必要があります。

- ソース上でどの型とfieldが定義されたか
- 構築時にどのfieldが明示され、どのfieldへ既定値が使われたか
- field accessがどの型とfieldへ解決されたか
- aggregateがbackendでどのようなメモリ、local、レジスタ、命令へ分解されたか
- 型としての抽象化がどの変換段階で失われたか

## 合意した基本方針

最初のproduct typeは次の性質を持ちます。

- 型宣言には一般的な入口として`type`を使う
- 型は名前によって区別するnominal typeとする
- fieldは名前と確定した型を持つ
- 型定義とaggregate literalのfieldはカンマで区切る
- 末尾のカンマを許可する
- fieldは値の作成後に直接書き換えない
- `mut`はaggregate内部ではなく、束縛全体の再代入を許可する
- aggregateは言語の意味として値として扱う
- 物理的なコピー、共有、分解方法は言語仕様で固定しない
- 型名は同じファイルのtop-level全体から参照できる
- 型名と値名は別のnamespaceで管理し、field名は型ごとに管理する
- backendに依存しない型とfieldの意味はPrimer IRより前に解決する
- aggregate literalの明示値はソース記述順、省略されたfieldの既定値はその後に型定義順で評価する
- memory layoutとABIはbackend lowering以降で決定する

## 構文

```text
type_definition :=
    "type" IDENT "{"
        field_definition ("," field_definition)* ","?
    "}"

field_definition :=
    IDENT ":" type ("=" expression)?

aggregate_literal :=
    IDENT "{"
        field_value ("," field_value)* ","?
    "}"

field_value :=
    IDENT ":" expression

field_access :=
    postfix "." IDENT
```

現在の実装では、fieldを一つも持たないproduct typeと空のaggregate literalを許可しません。型定義の`{}`を指して、少なくとも一つのfieldが必要だと診断します。

空の型には、値を持たない目印や状態を表す用途があります。しかし、Cには標準的な空structがなく、backendごとにゼロサイズ値の物理表現が異なります。必要な用途と観測方法を別に設計してから追加します。将来、空の型を許可する変更は、現在有効なプログラムの意味を変えません。

## 型定義

fieldの型は型定義で指定します。

```primer
type Point {
    x: f64,
    y: f64,
}
```

値を作るたびにfieldの型を繰り返しません。

```primer
point: Point = Point {
    x: 1.0,
    y: 2.0,
};
```

型定義内のfieldには、組み込み型だけでなくユーザー定義型も指定できます。

```primer
type Line {
    start: Point,
    end: Point,
}
```

fieldの型には`infer`を使用しません。型定義は値の使用場所に関係なく、確定した形を持つ必要があります。

```primer
type Point {
    x: infer, // エラー
}
```

一方、aggregate literal自体から束縛の型を推論できます。

```primer
point: infer = Point {
    x: 1.0,
    y: 2.0,
};
```

## 型のidentityと名前解決

同じfieldを持つ型でも、名前が異なれば別の型です。

```primer
type Point {
    x: f64,
    y: f64,
}

type Velocity {
    x: f64,
    y: f64,
}
```

`Point`と`Velocity`の間に暗黙の変換はありません。

型定義はtop-level itemとして扱います。実行されるstatementへ型定義を混ぜません。ASTは将来の関数定義も扱えるように、概念上は次の形を持ちます。

```text
Program
  Item::TypeDefinition
  Item::Statement
```

型名は宣言順に依存せず、同じファイルのtop-level全体から参照できます。

```primer
type Line {
    start: Point,
    end: Point,
}

type Point {
    x: f64,
    y: f64,
}
```

コンパイラは次の順に解決します。

1. top-levelの型名をすべて登録する
2. 各fieldの型を解決する
3. 値として無限サイズになる循環を検査する
4. 既定値を型検査する
5. 実行されるstatementを型検査する

名前は、役割ごとに別のnamespaceで管理します。namespaceとは、名前を登録して探すための「名前の箱」です。

- 型名は型namespaceへ登録する
- 束縛と将来の関数名は値namespaceへ登録する
- field名は、そのfieldを持つ型の中で管理する
- 同じscopeの同じnamespaceに、同名の定義を複数置くことはできない
- 型名と値名では、同じ綴りを使用できる

コンパイラは名前が書かれた場所から、どのnamespaceを探すか判断します。型指定の`Point`とaggregate literalを始める`Point`は型namespaceを探し、式の中の`point`は値namespaceを探します。

```primer
type Point {
    x: f64,
}

point: Point = Point {
    x: 1.0,
};
```

型と値に同じ綴りを使うこともできますが、読む人が区別しやすい名前を選ぶことを推奨します。言語仕様として大文字・小文字による役割の制限は設けません。

名前参照は、意味解析で種類と識別子へ解決します。概念上は次の情報になります。

```text
type-ref Point -> TypeId 0
value-ref point -> BindingId 3
field-ref x -> FieldId 0
```

必要なnamespaceに名前がなく、別のnamespaceに同名の定義がある場合は、その違いを診断します。たとえば値として定義された`Point`を型の場所で使った場合、単に「見つからない」とせず、値は存在するが型が必要だと伝えます。

## 再帰的な型

値を直接含み続けるためサイズを決定できない型は、意味解析で診断します。

```primer
type A {
    b: B,
}

type B {
    a: A,
}
```

将来、固定サイズの参照が導入された場合、参照を通した再帰型を許可するかは別に設計します。現在の判断は、将来の参照や再帰型を言語から排除するものではありません。

## 既定値

型の作者は、fieldへ明示的な既定値を定義できます。

```primer
type Options {
    retries: i64 = 3,
    verbose: bool = false,
    timeout: f64,
}
```

既定値のないfieldはaggregate literalで必ず指定します。

```primer
options: Options = Options {
    timeout: 10.0,
};
```

この構築では`retries`と`verbose`へ既定値を使用します。型ごとの暗黙のゼロ値は導入しません。

既定値には次の規則を適用します。

- fieldの型と一致する必要がある
- aggregateを構築するたびに適用する
- 明示された値があれば、そのfieldの既定値を使用しない
- 既定値の使用をPrimer IRで構造化して記録する
- 最初の実装では、実行時の束縛や同じaggregateの別fieldへ依存しない式を扱う

最後の項目は最初の実装範囲です。将来扱える式を永久に制限する決定ではありません。

## aggregate literal

fieldは名前で指定するため、aggregate literalでの記述順は型定義順と一致しなくても構いません。

```primer
point: Point = Point {
    y: 2.0,
    x: 1.0,
};
```

次を意味解析エラーにします。

- 存在しない型
- 存在しないfield
- 同じfieldの重複
- 既定値を持たないfieldの不足
- fieldの型と値の型の不一致

aggregate literalで明示されたfieldの式は、ソースに書かれた順番で評価します。その後、省略されたfieldの既定値を型定義に書かれた順番で評価します。明示されたfieldの既定値は評価しません。

評価結果は、名前解決済みの`FieldId`によって対応するfieldへ関連付けます。そのため、評価順、Primer IRでfieldを決定的に表示する順序、backendが決める物理的な配置順は別の情報です。

Primer IRは、実際の評価順と`FieldId`への対応を構造化して保持します。fieldの一覧は型定義順で決定的に表示できますが、その表示順へ式の評価を並べ替えてはいけません。これにより、将来、関数呼び出しや実行時エラーを含む式が追加されても、挙動が偶然のbackend実装へ依存しません。

## field access

field accessには`.`を使用し、入れ子にできます。

```primer
print(point.x);
print(line.start.y);
```

field accessは、意味解析で型とfieldへ解決します。文字列としてfield名をbackendへ渡し、backend側で再解決することはしません。

`if`と`while`では、条件の直後の`{`を本文の開始として読みます。構築直後のfieldを条件に使う場合は、構築式を丸括弧で囲みます。

```primer
if (Flags { enabled: true, }).enabled {
    print(true);
}
```

## 不変性と再代入

aggregateのfieldは作成後に直接書き換えません。

```primer
point.x = 3.0; // エラー
```

`mut`な束縛は、aggregate全体を新しい値へ再代入できます。

```primer
mut point: Point = Point {
    x: 1.0,
    y: 2.0,
};

point = Point {
    x: 3.0,
    y: point.y,
};
```

`mut`は保存されたaggregate自体へ外部から変更可能な権限を与える指定ではありません。現在の束縛へ別の同型の値を入れられることを示します。

## 値としての意味

aggregateを別の束縛へ渡しても、プログラムから見える共有された可変状態を作りません。

```primer
mut a: Point = Point {
    x: 1.0,
    y: 2.0,
};

b: Point = a;

a = Point {
    x: 3.0,
    y: 2.0,
};

print(b.x); // 1.0
```

この意味を保てる限り、backendやruntimeは値を物理的にコピー、共有、分解できます。copy、move、borrow、参照同一性の具体的な仕組みは、必要な型と操作を設計するときに決めます。

## Primer IR

Primer IRは、少なくとも次の意味を保持します。

```text
TypeId
FieldId
TypeDefinition
FieldDefinition
Type::Named(TypeId)
Construct
FieldAccess
FieldValueOrigin
```

`TypeId`は型のidentityを、`FieldId`は型内で解決されたfieldを表します。どちらもcompilation-localで決定的な識別子とします。

テキスト表現は次の形で、実際の綴りをsnapshotで固定します。

```text
type %Point@0 {
  field %x@0: f64 = 0.0f64
  field %y@1: f64 = 0.0f64
}

%point@0: %Point@0 = construct %Point@0 {
  field %x@0 = 10.0f64 [explicit]
  field %y@1 = 0.0f64 [default]
}

print.f64 field %point@0.%x@0
```

`explicit`と`default`は表示上の注釈だけでなく、内部の構造化された情報として保持します。既定値を使用したfieldは、その既定値を定義したソース範囲とも関連付けます。

## backend lowering

Primer IRでは、型名、field名、型、構築、field accessを保持します。次はbackend lowering以降で決定します。

- aggregate全体のサイズ
- fieldのoffsetとalignment
- padding
- メモリ、local、レジスタへの配置
- 値のコピーまたは共有方法
- ABI上の受け渡し方法

現在は、C struct、LLVMの名前付きaggregate、QBEの`alloc8`領域、WATのlinear memory、x86-64のstack、Primer VMの構造化された値へ変換します。QBEでは`blit`、WATとx86-64ではfieldごとのload/storeにより値をコピーします。その違いは出力成果物で観測できます。

WAT、QBE、x86-64の現在の内部layoutでは、各scalar fieldへ8バイトの場所を割り当てます。これは外部ABIや将来のlayoutを固定する言語仕様ではありません。

Primerソースから物理layoutを固定する構文と、外部ABI互換性は今回の設計範囲に含めません。

## Secretとの関係

通常のユーザー定義型は、それだけで秘密を隠す安全境界にはなりません。

将来`Secret`をaggregateまたはfieldと組み合わせる場合、`Secret`のredactionと伝播規則が通常の観測規則より優先されます。`default`や`explicit`という出自は観測できても、秘密の値そのものを表示してはいけません。

```text
field token = <secret> [default]
```

aggregateを実装することと、`Secret`の最終的な構文や解除方法を決めることは別の作業です。

## 現在の実装範囲

現在、次の範囲を実装しています。

- `type`によるtop-levelの名前付きproduct type
- nominalな型identity
- 組み込み型およびユーザー定義型を持つfield
- 明示値と既定値によるaggregate構築
- field access
- aggregate全体の束縛と再代入
- 型名のfile-wideな解決
- 値として無限サイズになる型循環の診断
- Primer IRでの型、field、構築、field access、値の出自
- bytecode、VM、すべてのbackendへのlowering
- 正常系、診断、各観測成果物のsnapshot
- 日本語と英語の仕様同期

`check`、Primer IR、bytecode、VM、C、LLVM、WAT、QBE、Windows x86-64のすべてで同じ言語上の意味を扱います。正常系、診断、八つの観測成果物をtestで固定しています。

## 後続で検討する機能

次は今回の仕様から排除せず、別の設計判断として後続へ分けます。

- `with`による一部fieldを置き換えた新しい値の作成
- aggregate全体の`==`と`!=`
- `print(aggregate)`と安定したformat
- type aliasとnewtype
- tuple、array、sum type、generic type
- field visibilityとmodule境界
- copy、move、borrow、reference
- 参照を通した再帰型
- custom layoutと外部ABI
- `Secret`の具体的な型表現と伝播

## 検証

積型の観測fixtureでは、既定値、値コピー、束縛全体の再代入、入れ子のfield accessを同じソースから各成果物へ変換します。C、LLVM、Windows x86-64については、生成物を`clang`でも検査できます。WATとQBEの実行系がない環境では、構造化IRとsnapshotで生成結果を検査します。
