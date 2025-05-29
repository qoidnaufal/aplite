# Aplite Storage
Some data structures needed for Aplite

## VecMap
My own implementation based on [`slotmap`](https://github.com/orlp/slotmap).
Performance wise, it's kinda ok and a little bit better than [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html).

### Comparison with HashMap
with hashmap:<br>
time needed to update: 5.042µs<br>
time needed to update: 6.542µs<br>
time needed to update: 5.167µs<br>
time needed to update: 5.875µs<br>
time needed to update: 6.416µs<br>
time needed to update: 6µs<br>
time needed to update: 8.708µs<br>
time needed to update: 7.25µs<br>
time needed to update: 5.5µs<br>
time needed to update: 5.625µs<br>
time needed to update: 5.209µs<br>
time needed to update: 8.75µs<br>
time needed to update: 8.209µs<br>
time needed to update: 7.625µs<br>
time needed to update: 7.291µs<br>
time needed to update: 6.875µs<br>
time needed to update: 7.666µs<br>
time needed to update: 8.917µs<br>

with vecmap:<br>
time needed to update: 3.375µs<br>
time needed to update: 4.208µs<br>
time needed to update: 4.333µs<br>
time needed to update: 3.709µs<br>
time needed to update: 4µs<br>
time needed to update: 4.584µs<br>
time needed to update: 2.625µs<br>
time needed to update: 2.209µs<br>
time needed to update: 4.25µs<br>
time needed to update: 4.917µs<br>
time needed to update: 5.166µs<br>
time needed to update: 5.375µs<br>
time needed to update: 4.458µs<br>
time needed to update: 4.292µs<br>
time needed to update: 4.875µs<br>
time needed to update: 4.625µs<br>
time needed to update: 5.208µs<br>
time needed to update: 5µs<br>

### DerivedMap
Todo

## Tree
Todo
