# Primerコンパイラの設計

[English](architecture.en.md)

Primerは、コンパイラによる変換を観測可能にするための静的型付き実験言語です。コンパイラの構造と変換境界を明示します。

可観測性についてPrimerが守る境界は、[可観測性の契約](observability.ja.md)で定めます。生成物に関する用語と条件は、[出力経路とターゲット](targets.ja.md)で定めます。

## 設計原則

Primerは、洗練された実装と可観測性の両立を目指します。実装や変換が高度になっても、その境界と結果を観測可能に保ちます。

具体的には、次の原則に従います。

- 型に関する決定を目に見える予測可能なものにします。
- バックエンドに依存しない意味は、バックエンドへのloweringより前に解決します。
- バックエンド固有の決定は、明示的なlowering境界の内側で行います。
- emitterはPrimerの意味を再解釈せず、バックエンドIRを出力形式へ変換します。
- 最適化を行う場合は、実験対象となる明示的で観測可能なパスとして導入します。
- 生成する観測結果に、タイムスタンプやランダムな識別子などの偶発的な非決定性を含めません。

Primer v0.1は、数値変換を暗黙に挿入したり、観測に有用な変換を隠したりしません。

## コンパイラ構成

コンパイラの処理経路は次のとおりです。

```text
Primer Source
      ↓
Lexer / Parser
      ↓
AST
      ↓
Primer IR Builder
  - semantic validation
  - type resolution
  - contextual float resolution
      ↓
Primer IR
  - typed
  - backend independent
      │
      ├── Observation 1: emit-ir / .pir
      │
      ↓
Backend Lowering
      ↓
Backend-specific Rust IR
      ↓
Emitter
      ↓
Backend Artifact
      │
      └── Observation 2
```

この構成で中心となる境界はPrimer IRです。

フロントエンドはPrimerプログラムの意味を決定します。バックエンドは、すでに解決された意味を対象の表現へ変換する方法を決定します。

### 構造上の不変条件

次の規則をコンパイラ設計の一部とします。

1. バックエンドのコンパイルは、ASTではなくPrimer IRから開始します。
2. 意味検証と型の解決は、バックエンドへのloweringより前に行います。
3. バックエンドのlowererは、Primer IRと自身のバックエンドIRの両方を参照できます。
4. バックエンドのemitterは、Primer IR、AST、意味解析の状態に依存してはいけません。
5. バックエンド固有のRust IRは、内部実装の境界です。
6. 公開する観測結果は、Primer IRのテキストと出力されたバックエンド成果物です。
7. 最適化を暗黙に行いません。将来最適化を導入する場合は、明示的で観測可能なパスにします。
8. Primer IRは束縛へcompilation-localな決定的IDを付け、名前がshadowingされても参照先を明示します。
9. 構造化された`if`、`while`、`break`、`continue`はPrimer IRに保持し、branch、合流点、後方経路、loop exitはBytecodeおよび各バックエンドIRへのloweringで導入します。

概念上、すべてのバックエンドは同じ構造に従います。

```text
Primer IR
    ↓
backend::lower()
    ↓
Backend-specific Rust IR
    ↓
backend::emit()
    ↓
Artifact
```

Rustモジュールの物理的な配置はバックエンドごとに異なる場合がありますが、構造上の境界は同じです。

## Primer IR

Primer IRは、構文解析、意味検証、型の解決を終えたあとに生成される、型付きでバックエンドに依存しない表現です。

すべてのPrimer IR式は、具体的な型に解決されています。

```text
i64
f32
f64
```

`infer`はPrimer IRが生成される前に解決されるため、実行時の型やバックエンドの型としては現れません。

接尾辞のない浮動小数点リテラルも、バックエンドへのloweringより前に解決されます。各バックエンドが文脈に基づく型推論を繰り返す必要はありません。

Primer IRは、汎用的なマシンIRになることや、早い段階でSSA形式を課すことを目的としません。フロントエンドとバックエンドの境界を明確に保てる範囲で、Primerの意味を直接表します。

## バックエンドへのlowering

各バックエンドは、出力を生成する前にPrimer IRをバックエンド固有のRust表現へloweringします。

現在の出力経路と実装上の境界は次のとおりです。

| 出力経路 | バックエンド内部表現 | 出力される成果物 |
| --- | --- | --- |
| C | C IR | `.c` |
| LLVM | LLVM IR表現 | `.ll` |
| QBE | QBE IR表現 | `.ssa` |
| WebAssembly | WAT指向の命令IR | `.wat` |
| Windows x86-64直接アセンブリ | アセンブリIR | `.s` |
| Primer bytecode | `BytecodeProgram` | `.pbc` |

バックエンドIRには、Primer IRに含めるべきでない決定を表現できます。

例を示します。

- LLVMとQBEの一時値およびバックエンド命令
- WATのスタックマシン命令およびローカル変数
- C固有の型および出力形式
- x86-64のスタックスロット、フレームサイズ、定数、レジスタ、ABI操作
- bytecodeのスロットおよびVM命令

これらの表現は現在、公開されたシリアライズ形式ではなく、安定したCLI契約にも含まれません。

### ソース位置とbytecode命令の出自

Primer IRの文と式は、対応するソースコードのUTF-8バイト範囲を保持します。範囲は開始位置を含み、終了位置を含みません。行番号と列番号は表示時にこの範囲から求めます。

各bytecode命令は命令本体とは別に、次のいずれかの出自を保持します。

- `Source(span)`: Primer IRの文または式から生成された命令
- `Synthetic`: ソースコード上に直接対応する箇所を持たない、コンパイラ生成の命令

`Synthetic`は追跡情報の欠落を意味しません。コンパイラ生成であることを明示する値です。

VMは実行エラーをbytecode命令番号で報告します。`run_vm`はその番号から命令の出自を解決し、利用可能な場合はソース位置も実行エラーへ関連付けます。この出自情報は現在の内部表現であり、`emit-bytecode`のテキスト形式には含めません。

## 観測境界

Primerは二つの主要な観測境界を公開します。

### 観測1: 解決済みのPrimerの意味

```text
primer emit-ir <file> [-o <output.pir>]
```

`.pir`は、フロントエンドによる意味と型の解決後、バックエンドへのlowering前に生成されます。

この観測結果は、次の問いに答えるためのものです。

> Primerは、このソースプログラムをどのような意味として解釈したか。

`emit-ir`が成功した場合、次のことが保証されます。

- 構文解析に成功しています。
- 意味検証に成功しています。
- 式の型が解決されています。
- 文脈に基づく浮動小数点型が解決されています。
- 結果はバックエンドに依存しません。

バックエンドの値割り当て、ABI、スタックマシン、対象命令に関する決定は、この観測結果に含めません。

### 観測2: 出力成果物

各出力経路のemitコマンドは、バックエンドへのloweringと出力を終えた結果を公開します。

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
```

これらの観測結果は、次の問いに答えるためのものです。

> 選択した出力経路とターゲットは、解決済みのPrimerプログラムをどのように表現したか。

既存の`emit-*`コマンドが観測APIです。同じ機能を重複させる別の`observe`コマンドは、現在のPrimerには必要ありません。

### 内部バックエンドIRは観測契約ではない

バックエンド固有のRust IRは観測1と観測2の間にありますが、内部表現のままにします。

バックエンドIRを公開すると、実装の詳細が互換性の要件になります。将来の実験で有用になる可能性はありますが、v0.1の観測契約には含めません。

バックエンドIRの明示的な観測点は、具体的な必要性が生じた場合にのみ追加します。

## コード生成

Primerは、最適化そのものではなく、観測できない暗黙のソースレベル最適化を避けます。

バックエンドへのloweringでは、対象表現に必要な機械的変換に加えて高度な変換も行えます。ただし、観測に有用な構造が失われる場合は、その変換を明示的で観測可能なパスとして扱います。

正当なloweringの例を示します。

- 型付きLLVM命令またはQBE命令の選択
- Primer式からWebAssemblyスタック命令への変換
- Primerの値からCの型および式への対応付け
- Direct ASMのスタックスロット割り当ておよび定数の実体化
- Primerの演算からbytecode命令へのlowering

将来最適化を導入する場合は、出力処理の内側に隠さず、名前と境界を持つ明示的なパスにします。

## ツールの責務

Primer、Tint*、Whitebaseは異なる責務を持ちます。

### Primer

Primerはコンパイラによる変換と出力を担当します。

```text
Parse
  ↓
Resolve
  ↓
Primer IR
  ↓
Lower
  ↓
Backend IR
  ↓
Emit
```

Primerは、観測可能なコンパイラ成果物を定義し生成します。

### Tint*

Tint*は、Primerのための視覚的な開発・観察環境です。

コンパイラの意味を重複して実装せず、Primerが公開するCLIの観測結果を利用します。ソースと生成された表現を対話的に調べ、比較しやすくすることがTint*の役割です。

### Whitebase

Whitebaseは、出力された成果物を実験への入力として利用します。

外部の選択を記録しながら、経路選択、ビルド、実行、計測、比較を行うことがWhitebaseの役割です。

```text
Primer source
      ↓
Primer
      ↓
Observation artifact
      ↓
Whitebase
  - select build route
  - invoke external tools
  - run
  - measure
  - compare
```

WhitebaseはPrimer内部のRust IRに依存せず、Primerを外部ツールの境界として扱います。

## 再現性

観測成果物は、直接比較できるときに最も有用です。

同じPrimerのバージョン、ソース入力、出力経路、ターゲット、ターゲット機能、明示的なオプションに対して、Primerは可能な限り決定的なテキスト観測結果を生成します。

タイムスタンプ、ランダムな識別子、環境依存のメタデータなどの偶発的な値は、それ自体が実験対象でない限りPrimerの観測結果に含めません。

外部ツールチェーンの出力はこの保証の対象外です。Whitebaseなどの利用側が記録します。

## 対象外と今後の検討事項

現在の設計では、次のものを必要としません。

- バックエンド固有のRust IRを公開するシリアライズ形式
- 既存の`emit-*`コマンドと重複する汎用的な`observe`コマンド
- すべてのバックエンドが共有する汎用SSA表現
- 暗黙の最適化
- Primer内部でのビルド処理の統括
- Primer内部でのベンチマークまたは性能測定

将来の検討事項には次のものがあります。

- 追加の観測境界を持つ明示的な最適化パイプライン
- 具体的な用途が生じた場合の、任意のバックエンドIR観測
- ソース、Primer IR、出力成果物、メタデータをまとめるObservation Bundle
- バックエンド側の処理を再現する実験で必要になった場合の、シリアライズされたPrimer IRの入力

これらの機能は、次の中心原則を保てる場合にのみ追加します。

> Primerは、変換をより観測しやすく、より説明しやすいものにします。
