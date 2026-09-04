# 固定長配列の設計

[English](fixed-arrays.en.md)

Status: Implemented

この文書は、Primerの固定長配列について、設計判断と観測できる情報を記録します。現在の構文は[言語リファレンス](../reference/language.ja.md)で定義します。

## まず何ができるか

```primer
values: [i64; 4] = [2, 4, 6, 8];
print(values[2]);
```

`[i64; 4]`は「`i64`を4個持つ配列」です。`values[2]`は、先頭を0番として3番目の値を読みます。

現在の固定長配列でできることは次のとおりです。

- 同じ型の値を、決まった個数まとめる
- 固定長配列をproduct typeのfieldにする
- 固定長配列を直接入れ子にする
- `i64`の添字で一つの要素を読む
- 配列全体を別の束縛へコピーする
- `mut`な束縛へ、同じ型の配列全体を再代入する
- ループと組み合わせて集計や線形探索を書く

要素一つだけの代入、動的な長さ、関数をまたぐ配列は現在の範囲に含めません。

## 設計判断

### 長さは型の一部

`[i64; 3]`と`[i64; 4]`は別の型です。これにより、配列が必要とする場所の大きさをcompile時に決められます。また、長さが違う配列を誤って代入したとき、実行前に見つけられます。

### 配列は値

配列を別の束縛へ入れると、配列全体がコピーされます。二つの束縛が、外から見えない同じ可変領域を共有することはありません。

この規則は名前付きproduct typeと同じです。内部でどのような命令やmemory copyになるかはbackendごとに違いますが、Primerの意味は変わりません。

### 固定サイズの値は組み合わせられる

名前付きproduct typeを配列の要素にでき、固定長配列をproduct typeのfieldにできます。

```primer
type Point {
    x: i64,
    y: i64,
}

type Path {
    points: [Point; 4],
}
```

この組み合わせでも、配列とproduct typeは独立した値としてコピーされます。`type Node { children: [Node; 1], }`のように、配列を挟んでも値の大きさが無限になる型はフロントエンドで診断します。

固定長配列は直接入れ子にもできます。

```primer
matrix: [[i64; 3]; 2] = [[1, 2, 3], [4, 5, 6]];
print(matrix[1][2]);
```

内側の配列も一つの値です。配列全体のコピーや再代入では、すべての階層がコピーされます。`matrix[row][column]`では、外側と内側の添字をそれぞれ検査します。

### 添字は必ず検査する

有効な添字は`0`から`length - 1`までです。負数や`length`以上の値は範囲外です。

Primer VMだけでなく、C、LLVM IR、QBE IR、WebAssembly Text、Windows x86-64 assemblyの各経路も境界検査を生成します。最適化のためにこの検査を黙って消すことは、現在の言語の意味を変えるため認めません。

## 観測できるもの

| 段階 | 残る情報 |
| --- | --- |
| AST | 要素型の構文、長さ、各要素、添字式、span |
| Primer IR | 解決済みの`[element; length]`、`array[...]`、`index(...)` |
| Bytecode | `array.new element length`、`array.get element length`、命令の出自 |
| Primer VM | 配列値、要素型、長さ、範囲外になった添字、失敗した命令位置 |
| Backend IR | 再帰的な型、配置、コピー、各添字の検査、要素addressの計算、loadまたはaggregate copy |
| 生成物 | backend固有の配列表現と、実際に実行される境界検査 |

配列の長さは型情報として残し、実行時に別の隠れたmetadataから推測しません。これにより、どの段階でも「何個の配列を扱っているか」を追えます。

## Backendでの表現

| Backend | 配列 | 境界検査 |
| --- | --- | --- |
| C | 要素のC配列を持つ専用`struct` | 型と長さごとの`primer_array_get_*` |
| LLVM IR | `[N x element]` | 型と長さごとの内部関数、違反時は`llvm.trap` |
| QBE IR | scalarは8 byte単位、product typeはfieldから求めたstrideのstack領域 | 比較と分岐、違反時は`abort` |
| WebAssembly Text | scalarは8 byte単位、product typeはfieldから求めたstrideのlinear memory | `i64.lt_s` / `i64.ge_s`、違反時は`unreachable` |
| Windows x86-64 | scalarは1 slot、product typeはfieldから求めた複数のstack slot | 負数と上限の比較、違反時は`ud2` |
| Primer bytecode | 型付きの配列値 | `array.get`をVMが検査 |

scalarの大きさが4 byteでも、QBE、WebAssembly、Windows x86-64では現在8 byte単位の場所を使います。product typeや配列の要素は、その値全体が必要とする場所をstrideにします。これは単純で観測しやすい現在のlayoutであり、Primerの型の意味ではなくbackend loweringの判断です。

## セキュリティ境界

観測用に見える配列型、長さ、binding ID、命令番号、memory addressは、実行中の配列を書き換える権限ではありません。観測と外部からの干渉は分けます。

境界検査は、不正なmemory accessを許す前に実行を止めます。ただし、配列は秘密を隠す型ではありません。生成物や観測結果から要素が見える可能性があります。秘密値を隠す仕組みは、現在の言語仕様にはありません。

## 現在の制限

- 要素型は`bool`、`i64`、`f32`、`f64`、名前付きproduct typeまたは固定長配列
- 長さは0より大きい整数
- 空の配列リテラルは未対応
- 要素への直接代入は未対応
- 関数のparameterと戻り値に配列は未対応
- 配列全体の比較と`print`は未対応

未対応の形は、backendごとに違う動作へ落とさず、フロントエンドで診断します。
