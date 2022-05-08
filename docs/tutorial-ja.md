# Chumsky: チュートリアル

*本チュートリアルでは`master`ブランチで更新されており，最新の安定版リリースではないことに注意してください: 詳細は異なる可能性があります*

本チュートリアルでは，プログラミング言語'Foo'のためのパーサ（とインタプリタ！）を開発します．

Fooは小さな言語ですが，私たちが楽しむには充分です．[チューリング完全](https://ja.wikipedia.org/wiki/チューリング完全)ではありませんが，Chumskyでパースに取り組む十分に複雑です．'本物の'プログラミング言語で見るようなたくさんの要素を含んでいます．以下は，Fooでかかれたサンプルコードです．

```
let seven = 7;
fn add x y = x + y;
add(2, 3) * -seven
```

インタプリタ全体のソースコードは，メインリポジトリの`examples/foo.rs`にあります．

## セットアップ

`cargo new --bin foo`で新しいプロジェクトを作成し，依存関係として最新版のChumskyを追加してから，`main.rs`に以下を記述してください．

```rust
use chumsky::prelude::*;

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    println!("{}", src);
}
```

このコードの目的はただ一つ，コマンドラインの第一引数をパスとして扱い，対応するファイルを読み，端末に内容を表示することです．このチュートリアルではIOエラーのハンドリングについてあまり気にしないこととするので，`.unwrap()`で充分です．

`test.foo`という名前のファイルを生成し，`cargo run -- test.foo`を実行してください（`--`はcargo自身ではなく，cargoに対して残りの引数を渡すように伝えます）．`test.foo`の内容があれば，コンソールに表示されるでしょう．

次に，Fooで書かれたプログラムを表現するデータ型を作成しましょう．Fooで書かれた全てのプログラムは式です．これを`Expr`と呼ぶことにします．

```rust
#[derive(Debug)]
enum Expr {
    Num(f64),
    Var(String),

    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Call(String, Vec<Expr>),
    Let {
        name: String,
        rhs: Box<Expr>,
        then: Box<Expr>,
    },
    Fn {
        name: String,
        args: Vec<String>,
        body: Box<Expr>,
        then: Box<Expr>,
    },
}
```

これはFooの[抽象構文木](https://ja.wikipedia.org/wiki/抽象構文木)（AST）です．これはあらゆるFooプログラムを表現し，かつFooプログラム自身を再帰的に定義します（`Box`は型が無限に大きくなることを回避するために使用されます）．それぞれの式は部分式を含む場合があります．

Fooのパーサを作成する関数も作ります．このパーサは`char`ストリームを入力を受け取り，`Expr`を生成するので，`I`（入力）と`O`（出力）の型パラメタとしてこれらの型を使います．

```rust
fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    // 後で記述します
}
```

関連型`Error`はChumskyが使うエラー型をカスタマイズできるようにします．今のところは，必要なことが全てできる組み込みエラー型`Simple<I>`に専念することにします．

`main`で`println!`を下記のよう変更します．

```rust
println!("{:?}", parser().parse(src));
```

## 桁をパースする

Chumskyは'パーサ・コンビネータ'ライブラリです．多くの小さなパーサを組み合わせてパーサを作成することができます．最小のパーサは'プリミティブ'と呼ばれ，[`primitive`](https://docs.rs/chumsky/latest/chumsky/primitive/index.html)モジュールに格納されています．

まずはFooの文法で最も単純な要素である'数字'のパースから始めたいと思います．

```rust
// In `parser`...
filter(|c: &char| c.is_ascii_digit())
```

`filter`プリミティブは一つの入力を読み，条件をパスした場合にそれを受け入れます．この場合，その条件は単に文字が数字であることをチェックします．

このコードを今コンパイルすると，エラーが発生します．なぜでしょうか？

パーサは`Expr`を生成すると約束しましたが，`filter`プリミティブは見つけた入力のみを出力します．このままでは`char`から`Expr`のパーサではなく，`char`から`char`へのパーサです！

これを解決するためには，パーサ・コンビネータの'コンビネータ'部分を分解する必要があります．Chumskyの`map`メソッドを使用してパーサの出力を`Expr`に変換することとします．これは`Iterator`の同名メソッドに似ています．

```rust
filter(|c: &char| c.is_ascii_digit())
    .map(|c| Expr::Num(c.to_digit(10).unwrap() as f64))
```

ここでは，`char`の桁を`f64`に変換し（アンラップしても問題ありません．`map`はパースに成功した出力にのみ適用されます），それを`Expr::Num(_)`でラップしてFooの式に変換します．

このコードを実行してみてください．`test.foo`に桁を入力すると，インタプリタが下記のようなASTを生成します．

```
Ok(Num(5.0))
```

## 数字をパースする

もう少し冒険してみると，複数桁の入力をしても期待通りの動作にならないことにすぐに気が付くでしょう．`42`と入力すると`Num(4.0)`とASTが生成されるだけです．

これは`filter`が*単一の*入力のみを受け付けるためです．しかし新しい別の疑問が生じます．なぜ我々のインタプリタはパースしなかった末尾の桁で文句を言わ*ない*のでしょうか．

その答えはChumskyのパーサが*怠惰*だからです．可能な限り入力の全てを消費した後に停止するでしょう．末尾の入力が残っている場合，無視されます．

これは明らかに常に望ましいわけではありません．ユーザがファイルの末尾にランダムで無意味なものを置く場合，それについてのエラーを生成できてほしいです！さらに悪いことに，その'無意味なもの'は，ユーザがプログラムの一部として意図した入力であっても，文法エラーを含んでいるため，適切にパースされません．どのようにして入力の全てをパーサが消費するようにできるでしょうか？

これをするには2つの新しいパーサを使います．`then_ignore`コンビネータと`end`プリミティブです．

```rust
filter(|c: &char| c.is_ascii_digit())
    .map(|c| Expr::Num(c.to_digit(10).unwrap() as f64))
    .then_ignore(end())
```

`then_ignore`コンビネータは1つ目のパターンの後に2つ目をパースしますが，1つ目の出力を選んで2つ目を無視します．

`end`プリミティブは，入力の終わりに遭遇した場合でのみ成功します．

これらを組み合わせると，長い入力に対してエラーがえられます．残念ながら，これは別の問題（得にUnix系プラットフォームで作業している場合）を明かにします．桁の前後にあるあらゆる空白がパーサを混乱させ，エラーを引き起こします．

桁パーサの後に`padded_by`（与えられたパターンの前後を無視する）の呼び出しを追加し，あらゆる空白文字をフィルタリングすることで，空白文字を処理することができます．

```rust
filter(|c: &char| c.is_ascii_digit())
    .map(|c| Expr::Num(c.to_digit(10).unwrap() as f64))
    .padded_by(filter(|c: &char| c.is_whitespace()).repeated())
    .then_ignore(end())
```

この例はChumskyのパーサに関していくつか重要なことを教えてくれます．

1. パーサは怠惰である．つまり，末尾の入力は無視されます．

2. 空白文字は自動的に無視されない．Chumskyは汎用のパーシングライブラリであり，かついくつかの言語は空白文字の構造について非常に気を遣うため，Chumskyもそうしています．

## 整理と近道

この時点で，物事は少し混乱しているように見え始めています．1桁の数字を適切にパースするために，4行のコードを書くことになってしまいました．少し整理しましょう．いくつかの無駄を省くために，Chumskyに付属するテキストベースのパーサプリミティブも活用します．

```rust
let int = text::int(10)
    .map(|s: String| Expr::Num(s.parse().unwrap()))
    .padded();

int.then_ignore(end())
```

より良くなりました．カスタム桁パーサを，任意の正の整数をパースする組み込みパーサと入れ替えました．

## 簡単な式を評価する

ここからはパーサから離れ，ASTを評価できる関数を作成します．これはインタプリタの'心臓部'であり，プログラムを実際に計算するものです．

```rust
fn eval(expr: &Expr) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a)?),
        Expr::Add(a, b) => Ok(eval(a)? + eval(b)?),
        Expr::Sub(a, b) => Ok(eval(a)? - eval(b)?),
        Expr::Mul(a, b) => Ok(eval(a)? * eval(b)?),
        Expr::Div(a, b) => Ok(eval(a)? / eval(b)?),
        _ => todo!(), // We'll handle other cases later
    }
}
```

この関数は一見すると恐ろしいかもしれませんが，それほど多くのことはしません．最終的な結果を得られるまで，ただ再帰的に自身を呼び出し，ASTの各ノードを評価し，演算子で結果を結合します．どんな実行時エラーも単に`?`を使ってスタックに戻されます．

`main`関数も少し変更し，`eval`にASTを渡せるようにします．

```rust
fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    match parser().parse(src) {
        Ok(ast) => match eval(&ast) {
            Ok(output) => println!("{}", output),
            Err(eval_err) => println!("Evaluation error: {}", eval_err),
        },
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }
}
```

これは大きな変更に見えますが，以前のコードを拡張し，パースが成功する場合に`eval`にASTを渡すようにしただけです．失敗した場合，パーサが生成したエラーを表示するだけです．今のところ，どの演算子も評価時にエラーを生成できませんが，将来的に変更するので，準備として処理できることを確認します．

## 単項演算子をパースする

パーサに戻って，単項演算子を処理しましょう．現在，単項演算子は負の演算子`-`だけです．我々は`-`の後に続く任意の数をパースしようとしています．より正式には下記の通りです．

```
expr = op* + int
```

我々はまた，後に明らかとなるように，`int`パーサに新しく'atom'と名前をつけます．

```rust
let int = text::int(10)
    .map(|s: String| Expr::Num(s.parse().unwrap()))
    .padded();

let atom = int;

let op = |c| just(c).padded();

let unary = op('-')
    .repeated()
    .then(atom)
    .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

unary.then_ignore(end())
```

ここでいくつかの新しいコンビネータを紹介します．

- `repeated`は与えられたパターンを任意の回数（ゼロも含む！）パースし，その出力を`Vec`に集約します

- `then`はあるパターンをパースし，その直後に別のパターンをパースして，両方の出力をタプルのペアに集約します

- `foldr`は`(Vec<T>, U)`形式の出力を受け取り，与えられた関数を`Vec<T>`の各要素に繰り返し適用することで，単一の`U`に畳み込みます．

最後のコンビネータはもう少し検討の価値があります．我々は単一のアトム（今のところただの数です）に続く，負の演算子`-`の*任意の数*をパースしようとしています．これは下記のような出力を得られます．

```rust
(['-', '-', '-'], Num(42.0))
```

`foldr`関数は下記のように，複数の要素を単一の要素に'畳み込む'関数を繰り返し適用します．

```
['-',   '-',   '-'],   Num(42.0)
  |      |      |          |
  |      |       \        /
  |      |     Neg(Num(42.0))
  |      |         |
  |       \       /
  |  Neg(Neg(Num(42.0)))
  |          |
   \        /
Neg(Neg(Neg(Num(42.0))))
```

これは命令型プログラミングに慣れている人々にとって想像することが少し難しいかもしれませんが，関数型プログラマにとっては自然に理解できるはずです．`foldr`は`reduce`と同じ意味です！

インタプリタを試してみてください．以前と同じように入力できますが，`-17`のような値も入力できます．負の演算子を複数回適用することもできます．`--9`はコマンドラインで`9`という値が出力されます．

これは面白いことです．最終的にインタプリタは便利な（ある種の）計算を行うようになったのです！

## 二項演算子をパースする

この調子で二項演算子に移りましょう．伝統的に，これらはパーサにとってかなりの問題となります．`3 + 4 * 2`のような式をパースするために，乗算が[加算よりも熱心に結合する](https://ja.wikipedia.org/wiki/演算子の優先順位)ことを理解する必要があり，よって初めに適用されます．それゆえに，この式の結果は`11`であり，`14`ではありません．

パーサはこれらのケースに対処するために様々な戦略を採用しますが，Chumskyにとっては単純なことです．最も熱心に結合する（最も'優先順位'の高い）演算子はパースするときに最初に考慮されるものでなければなりません．

注目すべき点は，加算演算子（`+`と`-`）は一般的に*同じ*優先順位をもつものとしてみなされることです．同じことが乗算演算子（`*`と`/`）にも適用されます．このため，各グループを単一のパターンとして扱います．

各ステージで，単純なパターンを探します．単項式と，それに続く演算子と単項式の組み合わせが続きます．より正式には下記のとおりです．

```
expr = unary + (op + unary)*
```

パーサを拡張してみましょう．

```rust
let int = text::int(10)
    .map(|s: String| Expr::Num(s.parse().unwrap()))
    .padded();

let atom = int;

let op = |c| just(c).padded();

let unary = op('-')
    .repeated()
    .then(atom)
    .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

let product = unary.clone()
    .then(op('*').to(Expr::Mul as fn(_, _) -> _)
        .or(op('/').to(Expr::Div as fn(_, _) -> _))
        .then(unary)
        .repeated())
    .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

let sum = product.clone()
    .then(op('+').to(Expr::Add as fn(_, _) -> _)
        .or(op('-').to(Expr::Sub as fn(_, _) -> _))
        .then(product)
        .repeated())
    .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

sum.then_ignore(end())
```

`Expr::Mul as fn(_, _) -> _`の文法は少し見慣れ無いかもしれませんが，心配しないでください！Rustでは，[タプルの列挙型のバリアントは暗黙的に関数となります](https://stackoverflow.com/questions/54802045/what-is-this-strange-syntax-where-an-enum-variant-is-used-as-a-function)．ここでやっていることは，Rustがそれぞれのバリアントを`as`キャストを使って同じ型をもつように扱い，残りを型推論に任せていることの確認です．これらの関数はパーサの内部を通過し，最終的に`foldl`呼び出しの`op`に格納されます．

その他の3つのコンビネータをここで紹介します．

- `or`はパターンのパースを試み，失敗した場合は別のパターンを試行します

- `to`は`map`に似ていますが，出力をマッピングするのではなく，新しい値で出力を上書きします．この場合は，各単項演算子を，その演算子と関連するASTノードを生成する関数に，変換するために使用します．

- `foldl`は前節の`foldr`にとても似ていますが，`(Vec<_>, _)`を操作するのではなく，`(_, Vec<_>)`を操作して関数で値を結合するため，逆方向へ進みます．

インタプリタを試してみましょう．インタプリタが任意の構成で組み合わせた単項演算子と二項演算子を正確に処理できることが分かるはずです．電卓として使えますね！

## 括弧をパースする

新しい挑戦者が近づいてきました．*ネストされた式*です．時にはデフォルトの演算子優先順位のルールを完全に上書きしたいですね．`(3 + 4) * 2`のように括弧で式をネストすることでできます．これをどのように処理するのでしょうか？

いくつか前の節の`atom`パターンの生成は偶然ではありません．括弧はどの演算子よりも優先されるので，単一の値のように括弧で囲われた式を扱わなければなりません．単一の値のように振る舞うこれを，慣例で'アトム'と呼びます．

パーサ全体をクロージャにまとめ，パーサが自身を定義できるようにします．

```rust
recursive(|expr| {
    let int = text::int(10)
        .map(|s: String| Expr::Num(s.parse().unwrap()))
        .padded();

    let atom = int
        .or(expr.delimited_by(just('('), just(')'))).padded();

    let op = |c| just(c).padded();

    let unary = op('-')
        .repeated()
        .then(atom)
        .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

    let product = unary.clone()
        .then(op('*').to(Expr::Mul as fn(_, _) -> _)
            .or(op('/').to(Expr::Div as fn(_, _) -> _))
            .then(unary)
            .repeated())
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    let sum = product.clone()
        .then(op('+').to(Expr::Add as fn(_, _) -> _)
            .or(op('-').to(Expr::Sub as fn(_, _) -> _))
            .then(product)
            .repeated())
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    sum
})
    .then_ignore(end())
```

いくつか注意すべき点があります．

1. `recursive`を使うと，クロージャのスコープ内でパーサのコピーを作成し，パーサを再帰的に定義できます

2. `atom`の定義内で`expr`の再帰的な定義を使用できます．新しい`delimited_by`コンビネータを使用し，括弧の中にネストできるようにします．

3. `then_ignore(end())`の呼び出しは`recursive`の呼び出しの内部に入れ込みられ*ません*．これは一番外側の式で入力の最後をパースしたいためです．ネストの各レベルではパースしません．

インタプリタを動かしてみてください．驚くほど多くのケースをエレガントに処理できることが分かるはずです．下記のケースが正しく動作することを確かめてください．

| Expression    | Expected result |
|---------------|-----------------|
| `3 * 4 + 2`   | `14`            |
| `3 * (4 + 2)` | `18`            |
| `-4 + 2`      | `-2`            |
| `-(4 + 2)`    | `-6`            |

## letをパースする

次のステップは`let`の処理です．Rustやその他の命令型言語とは異なり，Fooの`let`は，下記の形式の式であり，文ではありません（Fooは文を持ちません）．

```
let <ident> = <expr>; <expr>
```

`let`を式の最外レベルにのみ表示したいので，オリジナルの再帰的な式の定義からは除外します．しかし`let`を連鎖できるようにしたいので，`let`自身の再帰的な定義とします．`fn`構文も追加する予定なので，これを`decl`('declaration'; 定義)と呼びます．

```rust
let ident = text::ident()
    .padded();

let expr = recursive(|expr| {
    let int = text::int(10)
        .map(|s: String| Expr::Num(s.parse().unwrap()))
        .padded();

    let atom = int
        .or(expr.delimited_by(just('('), just(')')))
        .or(ident.map(Expr::Var));

    let op = |c| just(c).padded();

    let unary = op('-')
        .repeated()
        .then(atom)
        .foldr(|_op, rhs| Expr::Neg(Box::new(rhs)));

    let product = unary.clone()
        .then(op('*').to(Expr::Mul as fn(_, _) -> _)
            .or(op('/').to(Expr::Div as fn(_, _) -> _))
            .then(unary)
            .repeated())
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    let sum = product.clone()
        .then(op('+').to(Expr::Add as fn(_, _) -> _)
            .or(op('-').to(Expr::Sub as fn(_, _) -> _))
            .then(product)
            .repeated())
        .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)));

    sum
});

let decl = recursive(|decl| {
    let r#let = text::keyword("let")
        .ignore_then(ident)
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .then(decl)
        .map(|((name, rhs), then)| Expr::Let {
            name,
            rhs: Box::new(rhs),
            then: Box::new(then),
        });

    r#let
        // Must be later in the chain than `r#let` to avoid ambiguity
        .or(expr)
        .padded()
});

decl
    .then_ignore(end())
```

`keyword`は単に完全な識別子を探すパーサです（つまり，キーワードで始まるだけの識別子にはマッチしません）．

それ以外は，これまで見たことが無いものは`r#let`の定義にはなにもありません．似たコンビネータが，異なる方法で連結されています．文法の一部が存在することを検証した後に，気にしない箇所を選択的に無視し，その後，気にする要素を使用して`Expr::Let`のASTノードを作成します．

もう一つ注意しなければならないことは，`ident`の定義は`"let"`をパースするということです．パーサが誤って`"let"`が変数だと判断しないように，`r#let`を`expr`の早期に，または，チェインよりも早い段階で配置し，正しい解釈を優先させます．前節で述べた通り，Chumskyは，最初にパースが成功したものを選択することで，曖昧さを単純に処理します．よって正しい順序で宣言することの確認がしばしば重要になりえます．

インタプリタを実行すると，下記の入力を受け付けます．

```
let five = 5;
five * 3
```

残念ながら，`Expr::Var`と`Expr::Let`をまだ処理していないため，関数`eval`はパニックとなります．今からやりましょう．


```rust
fn eval<'a>(expr: &'a Expr, vars: &mut Vec<(&'a String, f64)>) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a, vars)?),
        Expr::Add(a, b) => Ok(eval(a, vars)? + eval(b, vars)?),
        Expr::Sub(a, b) => Ok(eval(a, vars)? - eval(b, vars)?),
        Expr::Mul(a, b) => Ok(eval(a, vars)? * eval(b, vars)?),
        Expr::Div(a, b) => Ok(eval(a, vars)? / eval(b, vars)?),
        Expr::Var(name) => if let Some((_, val)) = vars.iter().rev().find(|(var, _)| *var == name) {
            Ok(*val)
        } else {
            Err(format!("Cannot find variable `{}` in scope", name))
        },
        Expr::Let { name, rhs, then } => {
            let rhs = eval(rhs, vars)?;
            vars.push((name, rhs));
            let output = eval(then, vars);
            vars.pop();
            output
        },
        _ => todo!(),
    }
}
```

うーん．少し複雑になりましたね．でも大丈夫です，重要な変更は3つだけです．

1. 以前に定義した変数を追跡し続ける必要があるため，それらを記憶するために`Vec`を使います．`eval`が再帰的な関数であるため，全ての再帰呼び出しに渡す必要があります．

2. `Expr::Let`に到達したとき，初めに右項（`rhs`）を評価します．一度評価したら，`var`スタックにプッシュして末尾の`then`式（つまり，セミコロンの後に現れる残りのコード全て）を評価します．後でポップすることは*技術的に*必要ではありません．Fooではネストされた宣言を許可していないためですが，良い慣行であり，ネストを追加しようと思えばそうできるので，とりあえずポップします．

3. `Expr::Var`（つまり行中の変数）に到達したとき，スタックを*逆方向に*検索して（Fooでは[変数シャドウイング](https://en.wikipedia.org/wiki/Variable_shadowing)を禁止し，同じ名前で最近宣言された変数を見つけたいため）変数の値を探します．その名前の変数が見つからなかった場合，実行時エラーが発生しスタックに伝播されます．

明らかに，`eval`のシグネチャは変更されているので，`main`で呼び出されている呼び出しを次のように更新します．

```rust
eval(&ast, &mut Vec::new())
```

インタプリタをテストして確認しましょう．`let`宣言で実験してみて，壊れていないかを確かめます．特に，次のプログラムが`8`を生成することを確認し，変数シャドウイングをテストすることには価値があります．

Make sure to test the interpreter. Try experimenting with `let` declarations to make sure things aren't broken. In particular, it's worth testing variable shadowing by ensuring that the following program produces `8`:

```
let x = 5;
let x = 3 + x;
x
```

## 関数をパースする

Fooの実装はほぼ完成です．あとは一つだけ，*関数*が残されています．

驚くべきことに，関数のパースは簡単な部類です．変更が必要なのは，`r#fn`を追加して`decl`を定義することだけです．`r#let`の既存の定義に非常に似ています．

```rust
let decl = recursive(|decl| {
    let r#let = text::keyword("let")
        .ignore_then(ident)
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .then(decl.clone())
        .map(|((name, rhs), then)| Expr::Let {
            name,
            rhs: Box::new(rhs),
            then: Box::new(then),
        });

    let r#fn = text::keyword("fn")
        .ignore_then(ident)
        .then(ident.repeated())
        .then_ignore(just('='))
        .then(expr.clone())
        .then_ignore(just(';'))
        .then(decl)
        .map(|(((name, args), body), then)| Expr::Fn {
            name,
            args,
            body: Box::new(body),
            then: Box::new(then),
        });

    r#let
        .or(r#fn)
        .or(expr)
        .padded()
});
```

ここで新しいことはありません，全て理解したことです．

明らかに関数*呼び出し*のサポートも追加する必要がありますので，`atom`を修正します．

```rust
let call = ident
    .then(expr.clone()
        .separated_by(just(','))
        .allow_trailing() // Foo is Rust-like, so allow trailing commas to appear in arg lists
        .delimited_by(just('('), just(')')))
    .map(|(f, args)| Expr::Call(f, args));

let atom = int
    .or(expr.delimited_by(just('('), just(')')))
    .or(call)
    .or(ident.map(Expr::Var));
```

ここで唯一の新しいコンビネータは`seperated_by`で，これは`repeated`のように振る舞いますが，各要素にセパレータ・パターンを要求します．`allowed_trailing`と呼ばれるメソッドを持っており，要素の末尾にあるセパレータをパースできます．

次に，関数`eval`を修正して関数スタックをサポートします．

```rust
fn eval<'a>(
    expr: &'a Expr,
    vars: &mut Vec<(&'a String, f64)>,
    funcs: &mut Vec<(&'a String, &'a [String], &'a Expr)>,
) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a, vars, funcs)?),
        Expr::Add(a, b) => Ok(eval(a, vars, funcs)? + eval(b, vars, funcs)?),
        Expr::Sub(a, b) => Ok(eval(a, vars, funcs)? - eval(b, vars, funcs)?),
        Expr::Mul(a, b) => Ok(eval(a, vars, funcs)? * eval(b, vars, funcs)?),
        Expr::Div(a, b) => Ok(eval(a, vars, funcs)? / eval(b, vars, funcs)?),
        Expr::Var(name) => if let Some((_, val)) = vars.iter().rev().find(|(var, _)| *var == name) {
            Ok(*val)
        } else {
            Err(format!("Cannot find variable `{}` in scope", name))
        },
        Expr::Let { name, rhs, then } => {
            let rhs = eval(rhs, vars, funcs)?;
            vars.push((name, rhs));
            let output = eval(then, vars, funcs);
            vars.pop();
            output
        },
        Expr::Call(name, args) => if let Some((_, arg_names, body)) = funcs
            .iter()
            .rev()
            .find(|(var, _, _)| *var == name)
            .copied()
        {
            if arg_names.len() == args.len() {
                let mut args = args
                    .iter()
                    .map(|arg| eval(arg, vars, funcs))
                    .zip(arg_names.iter())
                    .map(|(val, name)| Ok((name, val?)))
                    .collect::<Result<_, String>>()?;
                vars.append(&mut args);
                let output = eval(body, vars, funcs);
                vars.truncate(vars.len() - args.len());
                output
            } else {
                Err(format!(
                    "Wrong number of arguments for function `{}`: expected {}, found {}",
                    name,
                    arg_names.len(),
                    args.len(),
                ))
            }
        } else {
            Err(format!("Cannot find function `{}` in scope", name))
        },
        Expr::Fn { name, args, body, then } => {
            funcs.push((name, args, body));
            let output = eval(then, vars, funcs);
            funcs.pop();
            output
        },
    }
}
```

また大きな変更です！しかしよく調べてみると，先ほど`let`宣言のサポートを追加したときの変更とよく似ています．`Expr::Fn`に到達する度に，`funcs`スタックに関数をプッシュして継続します．`Expr::Call`に到達する度に，変数と同じようにスタックを逆方向に検索し，関数の本体を実行します（引数を評価してプッシュすることを確かめてください！）．

前と同じく，`main`の`eval`呼び出しを変更する必要があります．

```rust
eval(&ast, &mut Vec::new(), &mut Vec::new())
```

インタプリタをテストしてください．何ができるかを見てみましょう！以下はサンプルプログラムです．

```
let five = 5;
let eight = 3 + five;
fn add x y = x + y;
add(five, eight)
```

## おわりに

ここまででChumskyのAPIを探検するのは終わりです．Chumskyができることをほんの一部だけ触ってみましたが，今ではさらに助けが必要な場合に，リポジトリにある例とAPIドキュメントの例に頼る必要があるでしょう．それにも関わらず，パーサ・コンビネータを使ってパーサを開発するということは面白い進出でした．

何はともあれ，あなたは今，こじんまりとした小さな計算機言語で遊べるようになりました．

興味深いことに，Fooの`eval`関数には微妙なバグがあり，関数呼び出しで予期しないスコープの挙動が発生します．これを読者の方への課題として残しておきます．

## 拡張タスク

- 興味深い関数スコープのバグを見つけ，どのように修正するか検討してください


- トークンのレキシングを別のコンパイルステージに分けて，パーサに`.padded()`を不要としてください

- より多くの演算子を追加してください

- 三項演算子`if <expr> then <expr> else <expr>`を追加してください

- 異なる型の値を，`f64`から`enum`に変換することで，追加してください

- Add values of different types by turning `f64` into an `enum`

- 言語内にラムダを追加してください

- エラーメッセージをより有用な方法でフォーマットしてください．もしかすると，元のコードへの参照を提供することでしょう．
