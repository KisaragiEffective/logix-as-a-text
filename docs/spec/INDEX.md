# Logix as a text
## 用語定義
1. LaaD: Logix as a DSL。
2. LNJ: Logix native JSON。LogixをJSONとしてエクスポートしたときの表現形式。
3. LZBS: lzma+bson。
4. CLI: 共通言語基盤 (Common Language Interface)

## 仕様の一覧
1. Logix as a DSL
2. LNJ to LaaD stub
3. LNJ-LZBS converter

## LNJ-LZBS converter
* LZBSはlzmaを解凍してBSONをJSONにするとLNJになる。
* LNJはJSONをBSONにしてlzmaで圧縮するとLZBSになる。

## L4B2LNJ
* LiteDBにはlz4bsonという別の形式で保存されている (独立したファイルとして見えるようになっている)。

