# logix-as-a-text
重要: このプロジェクトは凍結されています。後継言語は[origlang](https://github.com/KisaragiEffective/origlang)を、コーディングツールは[MirageX][https://github.com/rheniumNV/mirage-x-template]を参照してください。

## なにこれ
* [LogiX](https://wiki.neos.com/wiki/LogiX)をテキスト管理できるようにするプロジェクト

## 動機
* 特に大きいプロジェクトだと限界があるよね
* だからツールチェイン作りたいよね

## 言語設計
### コンセプト
* **LogiXをテキストで管理できるようにする**
* 既存の言語の良い点及び「良い慣習」、「広く受け入れられている慣習」を取り入れる
  * 例: 算術加算を行う演算子を`+`にする、演算子を中置記法で固定するなど

### 目指すところ
* 既存の言語の改善できる点を改善すること
* 厳密に定義された規格を作り上げること
* LogiXのローレイヤーを抽象化する
  * 例: `Attach Audio Clip`と`Attach Mesh`と`Attach Sprite`と`Attach Texture 2D`は出力こそ異なるものの、やりたいことは完全に同一なので、
* 既存のアセットとの簡単な相互IOをできるようにする
* LogiXのリファクタリングをより簡単にする

### 目指さないところ
* (暗黙のモノパック)

## 構文

(※構想は構文段階であることに注意されたい)

### Hello World
プログラマのみなさんにとってはおなじみのHello Worldは簡単にできる。

```text
"Hello, World!" -> display
```

これは `Hello, World!` という値を持つ `Input > string` ノードを作成し、`Display` ノードにそのまま出力する。

画像:
(Now printing...)
