# Chumsky

[![crates.io](https://img.shields.io/crates/v/chumsky.svg)](https://crates.io/crates/chumsky)
[![crates.io](https://docs.rs/chumsky/badge.svg)](https://docs.rs/chumsky)
[![License](https://img.shields.io/crates/l/chumsky.svg)](https://github.com/zesterer/chumsky)
[![actions-badge](https://github.com/zesterer/chumsky/workflows/Rust/badge.svg?branch=master)](https://github.com/zesterer/chumsky/actions)

強力なエラー回復機能を備えた，人間向けのパーサライブラリ

<a href = "https://www.github.com/zesterer/tao">
    <img src="https://raw.githubusercontent.com/zesterer/chumsky/master/misc/example.png" alt="Example usage with my own language, Tao"/>
</a>

*注: エラー診断レンダリングは[Ariadne](https://github.com/zesterer/ariadne)によって実行されます*

## 目次

- [機能](#features)
- [Brainfuckパーサの例](#example-brainfuck-parser)
- [チュートリアル](#tutorial)
- [パーサコンビネータとは*なにか*](#what-is-a-parser-combinator)
- [*なぜ*パーサコンビネータを使うのか](#why-use-parser-combinators)
- [分類](#classification)
- [エラー回復](#error-recovery)
- [性能](#performance)
- [計画されている機能](#planned-features)
- [哲学](#philosophy)
- [注釈](#notes)
- [ライセンス](#license)

## 特徴

- 多くのコンビネータ
- 入力，出力，エラー及びスパン型にまたがるジェネリック
- 強力なエラー回復戦略
- ASTへのインラインマッピング
- `u8`及び`char`向けのテキスト固有パーサ
- 再帰パーサ
- バックトラッキングを完全にサポートし，あらゆる文脈自由文法をパース可能
- 入れ子の入力をパースし，区切り文字の解析から字句のステージへ移動できます（Rustのように！）
- ビルトインパーサデバッギング

## [Brainfuck](https://ja.wikipedia.org/wiki/Brainfuck)パーサの例

完全なインタプリタは[`examples/brainfuck.rs`](https://github.com/zesterer/chumsky/blob/master/examples/brainfuck.rs)を確認してください(`cargo run --example brainfuck -- examples/sample.bf`)．

```rust
use chumsky::prelude::*;

#[derive(Clone)]
enum Instr {
    Left, Right,
    Incr, Decr,
    Read, Write,
    Loop(Vec<Self>),
}

fn parser() -> impl Parser<char, Vec<Instr>, Error = Simple<char>> {
    recursive(|bf| choice((
        just('<').to(Instr::Left),
        just('>').to(Instr::Right),
        just('+').to(Instr::Incr),
        just('-').to(Instr::Decr),
        just(',').to(Instr::Read),
        just('.').to(Instr::Write),
        bf.delimited_by(just('['), just(']')).map(Instr::Loop),
    ))
        .repeated())
}
```

他の例は下記のとおりです:

- [JSONパーサ](https://github.com/zesterer/chumsky/blob/master/examples/json.rs) (`cargo run --example json --
  examples/sample.json`)
- [簡易Rust-y言語のインタプリタ](https://github.com/zesterer/chumsky/blob/master/examples/nano_rust.rs)(`cargo run --example nano_rust -- examples/sample.nrs`)

## チュートリアル

Chumskyは[チュートリアル](https://github.com/zesterer/chumsky/blob/master/tutorial.md)で，どのように単項演算子，二項演算子，演算子優先順位，関数，let宣言及び呼び出しを持つ単純で動的な言語のためのパーサとインタプリタを書くかを教えてくれます．

## パーサコンビネータとは*なにか*

パーサコンビネータとは，他のパーサの観点からパーサを定義することで，パーサを実装するための技法です．その結果で得られるパーサは[再帰下降](https://ja.wikipedia.org/wiki/再帰下降構文解析)戦略を使ってトークンのストリームを出力に変換します．パーサを定義するためにパーサコンビネータを使うことは，Rustの[`イテレータ`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)トレイトを使いイテレーションアルゴリズムを定義することとほぼ同じです．`イテレータ`の型指向APIは手作業でイテレーションロジックを記述するよりも間違いを犯しにくく，かつ複雑なイテレーションロジックを簡単に記述することができる．同じことがパーサコンビネータにも言えます．

## *なぜ*パーサコンビネータを使うのか

優れたエラー回復機能を持つパーサを書くことは概念的に難しく，時間がかかります．そのためには再帰下降アルゴリズムの複雑さを理解し，その上で回復戦略を実装する必要があります．プログラミング言語を開発している場合，その過程であなたの考えが変わることはほとんど確実で，パーサのリファクタリングに時間と苦痛を伴うことになります．パーサコンビネータは，構文を素早く反復できる人間工学的なAPIを提供することでこの問題を解決します．

パーサコンビネータはまた，既存のパーサが存在しないドメイン固有言語にも最適です．このような状況で信頼性があり，フォールトトレラントなパーサを書くことは，降下パーサコンビネータライブラリの助けを借りると，数日かかる作業を半日の作業にします．

## 分類

Chumskyのパーサは[再帰下降](https://ja.wikipedia.org/wiki/再帰下降構文解析)パーサであり，全ての既知の文脈自由文法を含む[Parsing Expression Grammers (PEGs)](https://ja.wikipedia.org/wiki/Parsing_Expression_Grammar)のパースができます．Chumskyをより拡張し，限定的な文脈依存文法を受け入れることが理論的に可能ですが，これはほとんど必要ありません．

## エラー回復

Chumskyはエラー回復をサポートします．つまり，構文エラーに遭遇した場合，エラーを報告します．また，パースし続けられる状態に回復することで，複数のエラーを一度に発生させます．さらに，将来のコンパイルステージが消費するために，入力から部分的な[AST](https://ja.wikipedia.org/wiki/抽象構文木)を生成することができます．

しかし，エラー回復のための銀の弾丸はありません．定義によれば，パーサへの入力が無効である場合，パーサは入力の意味について経験則に基づく判断しかできません．異なる回復戦略は，異なる言語や言語内の異なるパターンに対してより良いでしょう．

Chumskyは様々な回復戦略（それぞれ`Strategy`トレイトを実装している）を提供しますが，どの戦略を，どこに，どの順番で適用するかが，Chumskyの生成できるエラーの質に大きく影響すると理解することが重要です．また，有用なASTを回復できる度合いにも大きく影響します．可能であれば，入力の大部分を無闇にスキップするのではなく，より「具体的な」回復戦略を初めに試してください．

異なる状況とパーサの異なるレベルで，異なる戦略を適用して実験し，満足のいく設定を見つけることを推奨します．もし提供されているエラー回復戦略が捕捉したい特定のパターンをカバーしない場合，Chumskyの内部を掘ったり独自の戦略を実装することも可能です．有用な戦略を思いついた場合，[メインリポジトリ](https://github.com/zesterer/chumsky/)に対してPRを開いてください！

## 性能

Chumskyは性能よりも，高品質なエラーと人間工学に重点を置いています．とはいえ，Chumskyが他のコンパイラに追いつけることが重要です．どれくらい正確にChumskyが動作するかは，解析対象，パーサを構築する方法，パーサに最初にマッチさせるパターン，エラー型の複雑さ，ASTの構築に関するものなど全てに依存するため，残念ながら，適切なベンチマークを作成することは*非常に*困難です．ですが，本リポジトリが含む[JSONベンチマーク](https://github.com/zesterer/chumsky/blob/master/benches/json.rs)を私のRyzen 7 3700xで実行したところ，次のような結果が得られました．

```ignore
test chumsky ... bench:   4,782,390 ns/iter (+/- 997,208)
test pom     ... bench:  12,793,490 ns/iter (+/- 1,954,583)
```

参考として，同じような設計のパーサ・コンビネータである[`pom`](https://github.com/J-F-Liu/pom)の結果も載せました．パースされるファイルは典型的なJSONファイルを広く表しており，3018行あります．1秒あたり630000行強のJSONを変換しています．

明らかに，最適化された手書きのパーサよりも少し遅いです．ですが，それで良いです！Chumskyの目標は*十分に速い*ことです．もしあなたの言語で十分なコードを書いて，パースの性能が問題になり始めたら，あなたは既に十分な時間とリソースを費やしており，手書きのパーサが最善の選択でしょう．

## 計画されている機能

- エラー回復とエラー生成をスキップする，最適化された'ハッピーパス'パーサモード
- 出力を生成しないが入力の妥当性を検証する，割り当てを行わないことが保証される，さら高速な'検証'パーサモード

## 哲学

Chumskyはこうでなければならない

- パーサが何をしているかを正確に理解していなくても，使いやすい
- 型指向であり，コンパイル時にアンチパターンからユーザを遠ざける
- デフォルトで文脈自由解析のための，成熟した'バッテリ内蔵の'ソリューションであること．もしも`パーサ`か`戦略`のいずれかを手作業で実装しなければならない場合，それは修正すべき問題である．
- '十分に高速'でありながら，より高速ではないこと（つまり，エラー品質と性能はトレードオフの関係がある場合，Chumskyは常に前者の選択肢をとる）
- モジュール形式でかつ拡張性があり，ユーザが独自のパーサ，回復戦略，エラー型，スパンを実装できること．及び，入力トークンと出力ASTの両方に対して汎用的であること．

## 注釈

このような無茶な名前を選択したNoam氏にお詫びいたします．

## ライセンス

ChumskyはMITライセンスで提供されています（メインリポジトリの`LICENSE`を参照）．
