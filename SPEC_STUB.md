仕様のドラフト：
* `<program>` := `<stmt>*`
* `<stmt>` := コメント | node\_def | `import`
* `<node_def>` := `<ident> = <node_path> | <expression> | <class>`: ノードやオブジェクトをDSL上で連結するために定義する。
* `<node_path>`: ノードのDSLパス。Logixノードと一対一で対応する。`Flow`もこれで指定できる [例1]。
* \[例1\]: `logix.event.one_per_frame -> logix.extension.plusplus.display`
* `<expression>`: 式。ただし、糖衣構文である。変数に格納されるかどうかは未規定。多分そのまま繋げる。
  * `Flow > If`は式ではないが、`Operator > ?:`は`if`式として実装されててほしい。
  * `if`式に改行が出現する場合は`end if`を末尾におかないと改行が出現した時点でエラー (`dangling-if-else`問題を防ぐため)

```rb
# OK
foo = if v then 2 else 3

# OK (冗長)
bar = if v then 2 else 3 end if

# Error
baz = if v then
2
else
3

# OK
qux = if v then
  2
else
  3
end if
```

もし`if`のいずれかの節が空か、`then`節しか存在しない場合か、すべてのブランチを考慮した結果__least-upper-boundが計算できない__ (例: まあなんか適当) __または lubが`Object` になる  (例: `string` と `int` の lubはおそらく`Object`) 場合__ (これはコンパイルエラーにしてもいいかもしれない。Logixはunion型を持たない)、その`if`は`if`文となる。この`if`文は`Flow > If`に変換されるべきである。
例：
```rb
t = if true then end if
```
* RefIDはCの`void *`並サポート
  * 壊れやすいことの体現
* `logix:dummy`型を受け取るノードはジェネリックなコネクションを貼れるようにする
* 絶対に成功しないキャスト (例: `logix:User` から `logix:int`) はコンパイルエラーになるべき
* `while`文と`for`文も必要だが私は眠いのでまた明日
  * 条件が`true`であるべきではない (簡単に無限ループを引き起こせるので)
* 属性: `#[attr]` or `#[attr(foo = "bar")]`
  * 共通言語基盤のアセンブリの属性を保存するときやnullability (`@NotNull`) を表示するときに活用することを想定している
  * いくつかの属性はコンパイラマジックによりコンパイラの動作を変える
* `a -> b` と記述して初めてノードが連結される。連結でなく`#[no_remove]`もないノードはコンパイル時に除去されてもよい。
  * ノードが連結しているかどうかはコンパイル時に再帰的に探索される。探索の開始点は入力を持たないノードとする。
* `a -> b -> c`と記述した場合は`a -> b`と`b -> c`と同じ。4つ以上もも同様。
* 当然型が合わないと連結できない
* 「デフォルトの値」はDSLに存在しない。すべて明示的に指定する必要がある。
* `null`は`null`と書く
* キャストは明示的に行われる（構文未定）
* 文字列リテラル：`"foobar"`
* だるいのでソースコードはUTF-8 without BOMのみ
  * logix同様絵文字も安全に入れることができる
* 型推論
  * 型推論はHM型推論を使ってパワフルにしたい
    * これはlogixが本質的に有向グラフであり、`a -> b -> c`とし、`b`がジェネリックなノードだったときに、`a`から後ろ向きに推論する必要があるから
  * `1` <- デフォルトで `int`
  * `1.0` <- `float`
  * その他いろんなリテラル (例：pack xyzの糖衣構文)
* 演算子
  * 同じ型同士
  * `string`に`/`はエラー

TODO
* `Flow > Write`系ノードの扱いについて書く
* DSL上のループとLogix上のループのモデリングをはっきりさせる
  * そもそも`Flow > While` と `Flow > For` をはっきりわかってない
