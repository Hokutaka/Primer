# Secret値の設計

[English](secrets.en.md)

**状態: Draft**

この文書は、Primerで秘密の値を扱うための`Secret`の目的、保証、限界、観測との関係を整理します。`Secret<T>`は概念を説明するための仮表記であり、最終的な構文やRust上の実装を確定するものではありません。

Primerは多くの変換を観測できることを目指します。しかし、観測範囲を広げても、password、token、個人情報、model weightなどの秘密を誤って表示または保存してはいけません。

`Secret`は、観測を無効にする全体設定ではありません。値ごとに設定する、観測と外部出力を越えさせない安全境界です。

全体の観測方針は[可観測性の契約](observability.ja.md)、設計の進め方は[コンパイラ進化計画](evolution-plan.ja.md)、具体的な将来用途は[設計判断のための利用シナリオ](use-case-analysis.ja.md)で整理します。

## 基本的な考え方

通常の値は、利用者が明示的に要求した観測へ含められます。

```text
value: i64 = 42
observation: 42
```

秘密の値は、広い観測を要求した場合でも中身を含めません。

```text
value: Secret<i64> = ...
observation: <secret>
```

Primerは`Secret`について次を目指します。

1. 秘密の値を観測、診断、debug表示へ含めない
2. 秘密から計算した値も、原則として秘密にする
3. 秘密を通常の出力、保存、外部送信へ暗黙に渡さない
4. 秘密を公開する操作を明示し、監査できるようにする
5. loweringや最適化によって秘密の印が失われないようにする

## `Secret`で表せるもの

`Secret`はpassword専用ではありません。任意の値を秘密として扱えます。

```text
Secret<String>
Secret<Bytes>
Secret<Token>
Secret<Model>
Secret<Tensor>
Secret<UserDefinedType>
```

`Password`などの領域固有型は、ユーザー定義型やlibrary型として`Secret`と組み合わせられます。

```text
Password
  value: Secret<Bytes>
```

役割は次のように分かれます。

| 概念 | 役割 |
| --- | --- |
| `Password` | passwordの作成、検査、hash化などの使い方を表す |
| `Secret<T>` | `T`の中身を観測と通常の外部出力から守る |

ユーザーが型を定義できることと、秘密指定を自由に解除できることは別です。`Secret`の解除は通常のfield accessや型変換では行えません。

## 見えるものと隠すもの

初期の設計候補では、秘密の存在と安全な変換は観測できますが、中身は観測できません。

| 情報 | 初期状態 | 理由 |
| --- | --- | --- |
| 実際の値 | 隠す | 秘密そのものだから |
| 値の文字列表現 | 隠す | 値の漏えいになるから |
| 値の長さ、shape、範囲 | 原則として隠すか個別判断 | 値を推測できる場合があるから |
| `Secret`であること | 見せる | 安全境界を監査するため |
| 内側の型`T` | 原則として見せる | 型検査と変換を説明するため |
| 出自と変換経路 | 値を含まない範囲で見せる | 秘密がどこへ流れたか監査するため |
| 公開操作が行われた事実 | 見せる | 秘密の越境を監査するため |
| 公開された実際の値 | 観測記録には含めない | 公開操作の監査と値の記録を分けるため |

型、長さ、shape、アクセス位置、実行時間も秘密になる用途があります。その場合は、値だけを隠す`Secret`より強い方針が必要です。最初から「存在を含むすべてが隠れる」とは主張しません。

## 秘密の伝播

秘密の値を使った計算結果は、原則として秘密になります。

```text
secret: Secret<i64>

secret + 1              -> Secret<i64>
secret == candidate     -> Secret<Bool>
hash(secret)             -> Secret<Hash>
make_record(secret)      -> Secret<Record> または秘密fieldを持つRecord
```

この規則は、秘密から計算した値を通常値として観測する抜け道を防ぎます。

どこまで伝播させるかは、次の二種類に分かれます。

### 明示的なデータの流れ

秘密の値を演算、引数、field、戻り値として直接使用する流れです。型とIRで比較的明確に追跡できます。

初期実装では、少なくともこの流れを追跡する必要があります。

### 制御による暗黙の流れ

秘密を直接出力しなくても、分岐結果から秘密が漏れる場合があります。

```text
if secret_flag {
    print(1);
} else {
    print(0);
}
```

表示される`1`または`0`から`secret_flag`が分かります。これは値を隠すだけでは防げません。

制御による流れには、次の候補があります。

| 方針 | 利点 | 制約 |
| --- | --- | --- |
| `Secret<Bool>`を通常の分岐条件に使えない | 単純で漏えいを防ぎやすい | 秘密に基づく処理を書きにくい |
| 秘密条件の内側で作られた値と副作用を秘密として扱う | より多くの処理を表現できる | 型とeffectの追跡が複雑になる |
| constant-timeな秘密選択を別操作として提供する | timing leakを抑えた選択を表現できる | backendごとの保証と検証が必要になる |

第一候補は、安全な制御モデルが設計されるまで`Secret<Bool>`を通常の公開分岐条件として使わせないことです。秘密条件を必要とする機能は、制御構文、effect、backend保証と一緒に設計します。

## 明示的な公開

秘密を通常値へ変える操作を、明示的な公開（declassification）と呼びます。

```text
secret_result: Secret<Bool>
public_result: Bool = declassify(secret_result)
```

`declassify`は通常のcastではありません。秘密の安全境界を越える操作です。

公開操作には、少なくとも次が必要です。

- ソース上で明示される
- 対象となる値が明確である
- どの位置で行われたか診断と観測から確認できる
- 公開理由または用途を将来関連付けられる
- 外部の観測利用者が勝手に実行できない
- 暗黙の最適化で追加または削除されない

観測記録には、公開操作の種類、位置、対象の出自を含められます。ただし、公開された実際の値を自動的に保存しません。

公開を許可する主体を、プログラム作成者、実行者、policyのどこに置くかは未決定です。構文と権限モデルを設計するときに決めます。

秘密境界を越える操作には、元の内容をそのまま通常値にする`declassify`と、信頼された変換によって公開可能な別の表現を作る操作があります。この二つは同じ公開操作として扱いません。後者の条件は「暗号化との境界」で整理します。

## 観測境界での扱い

秘密は、文字列を描画するときだけ伏せればよいものではありません。観測スナップショットを作る時点で中身を含めない必要があります。

```text
compiler or VM state
      ↓
observation capture boundary
      ↓
redacted immutable snapshot
      ↓
renderer / file / external tool
```

rendererだけで`<secret>`へ置き換えると、rendererへ渡る前の観測データに秘密が残ります。保存、通信、crash dump、別のrendererから漏れる可能性があります。

次の出力先はすべて秘密の安全境界を守ります。

- 診断
- debug表示
- スナップショット
- 変換記録
- Emission Mapの付随情報
- Primer VMのtrace
- CLI出力
- 将来の公開観測スキーマ
- Tint*、Whitebase、エージェントへ渡すデータ

秘密の出自や変換を表示する場合も、値、長さ、内容を推測できる文字列を含めません。固定されたredaction表現を使用します。

## 通常の出力と外部送信

`Secret`は観測だけでなく、通常のプログラム出力にも適用します。

初期の設計候補では、次を暗黙に許可しません。

- `print(secret)`
- secret値の通常ファイルへのserialize
- secret値のnetwork送信
- secret値を通常の戻り値として外部境界へ渡す
- secret値を非secretなaggregateへ格納する
- secret値をformat文字列へ埋め込む

必要な処理は、秘密を扱えることが明示されたAPI、または明示的な公開を通します。

保存先や送信先が暗号化されていることと、`Secret`の型規則は別です。暗号化APIへsecret値を渡す場合も、そのAPIが秘密を扱える境界として定義されている必要があります。

## 暗号化との境界

`Secret`と暗号化は、別の問題を解決します。

- `Secret`は、その値を見せてよいかをコンパイラが追跡し、観測や通常出力への漏えいを防ぎます。
- 暗号化は、鍵を使って値を別の表現へ変換します。暗号方式、鍵管理、protocolなどはlibraryやruntimeが担当します。

概念上の暗号化APIは、次のように表せます。

```text
plaintext: Secret<Bytes>
key: Secret<Key>
nonce: Bytes
ciphertext: Bytes = trusted_encrypt(plaintext, key, nonce)
```

平文と鍵は`Secret`のままです。暗号文を通常値として扱えるのは、`trusted_encrypt`が秘密を扱うために承認されたAPIであり、その契約が出力を公開可能と定めている場合だけです。

これは`declassify`とは異なります。

- `declassify(secret)`は、元の内容そのものを公開します。
- `trusted_encrypt(secret, ...)`は、元の内容を公開せず、契約に従って別の表現を作ります。

通常のユーザー関数が、secret値を受け取って勝手に通常値を返せてはいけません。信頼された変換には、少なくとも次が必要です。

- 秘密を受け取れることと、出力の秘密区分がsignatureまたは契約に明記される
- compilerまたはpolicyによって、その契約を使う権限が与えられる
- 呼び出し位置、変換の種類、入力の出自を、実際の値なしで監査できる
- backendとruntimeが必要な境界を実装できない場合は拒否する
- 通常の関数定義やcastでは、secretの印を取り除けない

hash化も、自動的に安全な公開変換とはみなしません。決まった入力から同じhashが得られる場合や、候補を総当たりできる場合は、元の秘密を推測できるためです。redaction、集約、匿名化なども、それぞれの契約とpolicyで公開可能性を判断します。

暗号方式の選択、認証付き暗号、nonce生成、鍵の保存と更新、通信protocolは、Primerのcompiler coreより上位の責務です。Primerは暗号を独自実装せず、承認されたAPIがどこで使われ、秘密がどこから流れたかを説明できるようにします。

FFIや外部の暗号libraryも、秘密を扱える信頼境界として明示される必要があります。未対応または契約を確認できない境界へsecret値を渡す処理は拒否します。

`Secret`を付けても、runtime上の平文や鍵が自動的にメモリ内で暗号化されるわけではありません。必要な場合は、秘密用memory、zeroization、hardware支援などを別の保証として追加します。

## コンパイルとlowering

`Secret`の印は、型検査だけで消してはいけません。

```text
Source type
  -> Primer IR
  -> Control-flow IR
  -> Backend IR
  -> runtime value or storage
```

各段階で次を確認します。

- secret値の出自が保持されている
- 非secretな観測データへ値がコピーされていない
- secret値が公開出力へ到達していない
- 公開操作が明示的に残っている
- 最適化後も安全境界が維持されている
- backendが必要な保証へ対応している

backend内部で型表現が消える場合も、観測のredactionと安全性検証に必要な情報は副次データとして保持します。

未対応のbackendは、`Secret`を無視して通常値としてloweringしてはいけません。明示的な未対応診断を返します。

## ユーザー定義型との関係

ユーザー定義型は、秘密の意味を持つ型を作るために使えます。

```text
Password
ApiToken
PrivateKey
PersonalRecord
ModelWeights
```

しかし、通常のユーザー定義型だけでは、Primerの観測機構が中身を読み取らない保証を作れません。`Secret`は型、qualifier、effectなど、コンパイラが安全境界として理解する仕組みである必要があります。

最終的な表現は未決定です。

```text
Secret<Password>
secret Password
Password marked as secret
```

どの構文を選んでも、通常の型定義から秘密指定を偽装または解除できないことが必要です。

## `Secret`が保証しないもの

`Secret`は暗号化そのものではなく、すべての攻撃を防ぐ仕組みでもありません。

初期の`Secret`だけでは、次を保証できません。

- メモリ上の値が暗号化されること
- 使用後のメモリが必ずzeroizeされること
- timing、cache、分岐、アクセスpatternから情報が漏れないこと
- OS、debugger、別process、管理者から値を隠すこと
- 悪意のあるbackendや外部toolchainから値を隠すこと
- ソースへ直接書いた秘密literalをソース閲覧者から隠すこと
- binaryへ埋め込んだ秘密定数を解析から守ること
- 外部libraryが受け取った値を安全に扱うこと

constant-time処理、暗号化、zeroization、秘密管理、sandboxなどは、必要に応じて別の保証として設計します。

特にpasswordや鍵をソースへ直接書いて`Secret`で包んでも、安全にはなりません。秘密はruntimeの安全な入力境界から受け取る必要があります。

## セキュリティの段階

何を隠すかによって必要な仕組みが異なります。

| 段階 | 隠す対象 | 必要な仕組み |
| --- | --- | --- |
| 値の秘密 | 値と直接導かれた内容 | `Secret`、redaction、公開制御 |
| metadataの秘密 | 型、長さ、shape、名前、出自 | metadata policy、より強いredaction |
| 挙動の秘密 | 分岐、時間、アクセスpattern、通信量 | information-flow制御、constant-time、oblivious処理 |

最初の`Secret`は値の秘密を中心にします。metadataと挙動まで隠す必要がある用途では、追加の保証が必要であることを明示します。

## 想定用途

- password、token、秘密鍵を診断やtraceへ出さない
- MLのmodel weight、入力、gradientを観測から除外する
- 個人情報を含むfieldを安全に扱う
- 広いコンパイラ観測を有効にしたまま、秘密値だけを伏せる
- 秘密がどの処理へ流れ、どこで公開されたかを監査する
- エージェントへ観測データを渡す前に秘密を除去する

## 実装前に決めること

1. `Secret<T>`を通常のgeneric type、型qualifier、effectのどれとして表すか
2. 内側の型、長さ、shape、名前をどこまで見せるか
3. 明示的なデータの流れをどのIR段階で追跡するか
4. `Secret<Bool>`による制御の流れをどう扱うか
5. 公開操作の構文、権限、理由をどう表すか
6. 直接の公開と、暗号化などの信頼された変換をどう区別するか
7. 信頼された変換を誰が承認し、出力の秘密区分をどう宣言するか
8. 関数parameter、戻り値、field、containerで秘密をどう伝播するか
9. 外部関数とFFIへsecret値を渡す条件
10. secret値を扱える標準APIの条件
11. backendが満たすべき最低保証
12. runtimeメモリのzeroizationをどこまで保証するか
13. panic、crash、core dumpでの扱い
14. observation snapshotでの固定されたredaction形式

## 完了条件

`Secret`の最初の実装は、少なくとも次を満たしたときに完了とします。

- secret値を通常値と型で区別できる
- secret値から直接得た結果へ秘密が伝播する
- 診断、debug、snapshot、traceが実際の値を含まない
- 通常の`print`、serialize、外部出力がsecret値を拒否する
- 明示的な公開または承認された変換だけが秘密境界を越えられる
- 通常の関数やcastではsecretの印を取り除けない
- 信頼された変換が出力の秘密区分を明示する
- 公開操作を値なしで監査できる
- 未対応backendが明示的な診断を返す
- 最適化前後で秘密境界が維持される
- 正常系、拒否される操作、redaction、backend未対応のテストがある
- 保証しないside channelと外部境界が文書化されている

`Secret`は、Primerの可観測性を弱める例外ではありません。安全に観測できる範囲を型として明示し、秘密を漏らさずに変換経路を説明するための仕組みです。
