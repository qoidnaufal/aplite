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
On 1000000 repeated insert & get:
- INSERT: `map`: 19.487625ms | `std hashmap`: 102.835ms | `storage`: 9.076708ms
- GET: `map`: 2.628375ms | `std hashmap`: 45.996208ms | `storage`: 0ns | `storage unsafe`: 0ns
