# Aplite Storage
Some data structures needed for Aplite

## Tree
A super simple tree implementation, in which each `parent`, `first_child`, and `next_siblings` are stored separately in a `Vec`.

## EntityManager
Unique Id generator with index and version stored within.
Kinda inspired by how [`slotmap`](https://github.com/orlp/slotmap) works.
Hopefully this improve the reusability of generated integer for an entity.

## Map
Non hashing HashMap, just feed the index of the `Entity (u64)` directly.

## Storage
Inspired by how [`slotmap`](https://github.com/orlp/slotmap) works, but using enum instead of union.

## Performance
On 1000000 iteration of insert & get:
- INSERT: `map`: 19.530042ms | `std`: 104.050333ms | `index_map`: 9.42825ms
- GET: `map`: 3.712916ms | `std`: 47.293583ms | `index_map`: 0ns
