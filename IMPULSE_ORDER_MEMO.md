# Impulseの実行順序
## If-Else
### LaaD
```scala
declare m: Impulse
declare expr: bool
declare impulse1: Impulse
declare impulse2: Impulse

m -> if (expr) {
    impulse1
} else {
    impulse2
}
```

### LogiX
```text
+---+      +----+     +----------+
| m |=====>|    |====>| impulse1 |
+---+      |    |     +----------+
           | If |
+------+   |    |     +----------+
| expr |==>|    |====>| impulse2 |
+------+   +----+     +----------+
```

## If-Elseif-Else
### LaaD
```scala
m -> if (expr) {
    impulse1
} else if (expr2) {
    impulse2
} else {
    impulse3
}
```

### LogiX
```text
+---+      +----+     +----------+
| m |=====>|    |====>| impulse1 |
+---+      |    |     +----------+
           | If |
+------+   |    |     +----+     +----------+
| expr |==>|    |====>|    |====>| impulse2 |
+------+   +----+     |    |     +----------+
                      | If |
+-------+             |    |     +----------+
| expr2 |============>|    |====>| impulse3 |
+-------+             +----+     +----------+
```

## While
### LaaD
```scala
t -> {
  start
  while(cond) {
    loop
  }
  end
}
```

### LogiX
```text
+---+      +-------+     +-------+
| t |=====>|       |====>| start |
+---+      |       |     +-------+
           |       |
           |       |     +------+
           | While |====>| loop |
           |       |     +------+
           |       |     
+------+   |       |     +-----+
| cond |==>|       |====>| end |
+------+   +-------+     +-----+
```

## Range-for
### Sugared LaaD
```scala
t -> {
  start
  for (i in 0..5) {
    impulse
  }
  end
}
```

### Desugared LaaD
注：`__`から始まる変数名は説明のために導入したものである。

```scala
__while_node = logix.flow.while
t -> start -> __while_node.fire
__cond -> __while_node.condition
__i_init = logix.flow.write
__while_node.start -> __i_init.fire
0 -> __i_init.fire
i = Variable<i32>
__i_init.variable -> i
__while_node.do -> impulse
impulse -> __i_increment.fire
__i_increment = logix.flow.write
__i_increment.variable -> __i_plus_1
__i_plus_1 = logix.math.plus1<i32>
i.value < 5 -> __cond
__while_node.after -> end
```

## Non-range for
### Sugared LaaD
```scala
t -> {
  for (start, cond, end) {
    impulse
  }
}
```

### Desugared LaaD
```scala
t -> start
__for_node = logix.flow.while
start -> __for_node.fire
cond -> __for_node.cond
__for_node.do -> impulse
impulse -> end
```

### メモ: Java
```java
t();
for(start(); cond(); end()) {
    impulse();
}
```

### プログラムの命令中の順序について
* 単純な変数から変数への接続は他の変数から変数への接続と入れ替わっても良い (MAY)
	* `a -> b`と`c -> d`がある場合、互いに他方の接続の宣言と入れ替わったかのように解釈されても良い (MAY)
	* 接続自体は純粋であるため
* `a -> b`と書いたとき、`a`は`b`が評価される前に評価されなければならない (MUST)
	* このとき、happens-before関係が生まれる
* オプティマイザは演算子の評価順を意味論が変えない範囲で自由に入れ替えることができる (MAY)
	* 例：`expensive_check && cheap_check`を`cheap_check && expensive_check`と解釈しても良い (MAY)
	* 例：`1 - 2`を`2 - 1`と解釈してはならない (MUST NOT)
* 
