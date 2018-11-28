# Quoridor Judge Server
## 使い方
```
###################
#        P        #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#        E        #
###################
```

## 入力形式
```
[先攻の駒のx座標] [先攻の駒のy座標] [後攻の駒のx座標] [後攻の駒のy座標] [先攻の壁の残り枚数] [後攻の壁の残り枚数]
w_00 w_01 ... w_0(W-1)
w_10 w_11 ... w_1(W-1)
. . .
w_(H-1)0 w_(H-1)1 ... w_(H-1)(W-1)
```

`w_ij`: 座標(j,i)の壁のフラグ

## 出力形式
### 移動する場合
```
x y
```
左上が(0,0)，右下が(W-1,H-1)
### 板を置く場合
```
x y dir
```
`dir`: 'H' or 'V'
例えば
`1 1 H`としたときには
```
###################
#        P        #
# * * * * * * * * #
#                 #
# *-*-* * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#                 #
# * * * * * * * * #
#        E        #
###################
```
となる．
