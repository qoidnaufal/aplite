# Aplite Storage
Some data structures needed for Aplite

## VecMap
My own implementation based on [`slotmap`](https://github.com/orlp/slotmap).
Performance wise, it's kinda ok and a little bit better than [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html).

## DerivedMap
Also my own implementation based on slotmap.

### Comparison against HashMap
For now, it seems there's no significant advantage from using `VecMap`.
Graph management is also easier and more natural with `HashMap`.
I'll just leave it here for comparison based on updating signal value.

with `HashMap`:
- [X] average: 50.571µs
- [X] hi: 76.916µs
- [X] lo: 27.333µs
- [X] update amount: 100

with `VecMap` & `DerivedMap`:
- [X] average: 49.093µs
- [X] hi: 86.584µs
- [X] lo: 23.917µs
- [X] update amount: 100

## Tree
Todo
