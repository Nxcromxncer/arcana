# Candy 03 — `collections`

Hash maps, hash sets, caches, a dynamic array, linked lists, a disjoint set
union, a bloom filter and a skip list for Arcana — everything built by hand on
top of lists and maps, in the same closure-object style as
[`structures`](01-structures.md).

```arc
import candy.collections

spell main() {
    bind m = candy.collections.hash_map()
    m["set"]("gem", 3)
    m["set"]("coin", 7)
    cast m["get"]("gem")               ? 3

    bind cache = candy.collections.lru_cache(2)
    cache["put"]("a", 1)
    cache["put"]("b", 2)
    cache["put"]("c", 3)               ? silently evicts "a"
    cast cache["keys"]()               ? ["c", "b"]
}
```

## Importing

| Your file sits...         | Import                        |
|---------------------------|-------------------------------|
| in the `arcana` project root | `import candy.collections` |
| next to a `candy/` folder | `import candy.collections`    |
| next to the module itself | `import collections`          |

```arc
import candy.collections            ? full name
import candy.collections as coll    ? aliased
from candy.collections import hash_map, lru_cache
import collections                  ? when this file sits next to yours
```

The full import table (aliases, `ARCANA_PATH`, the `import candy` aggregator)
is in [01-structures.md](01-structures.md#importing) and applies unchanged to
this module.

## Conventions shared by every Candy structure

1. **Structures are closure objects.** A constructor such as
   `hash_map()` returns a map of spells sharing private state. Call
   methods by indexing: `m["set"]("gem", 3)`. There are no classes in Arcana.
2. **Every instance has a `"kind"` tag** — e.g. `"candy/hash-map"` — so
   programs can tell structures apart.
3. **Misuse raises catchable `AssertionError` (E2101)** with a message that
   starts with `Candy:` (a non-text key, an out-of-range index, a bad
   capacity, ...). Catch with `try`/`catch` and inspect `e["message"]`.
4. **Queries return values, mutators return `null`** — with the documented
   exceptions that carry information: `pop`, `remove_at`, `remove`, `add`,
   `delete`, `union`, ... `to_list()`, `keys()`, `items()` always return a
   fresh list, never internal state.
5. **Hash-key types**: anything used as a hash or map key (hash map keys, hash
   set values, bloom filter items) must be text or integer, because Arcana
   maps and the hash helpers only support those.

---

## DynamicArray

`dynamic_array()` — a hand-rolled growable array: a backing list of
`capacity` slots plus a count. Grows ×2 when full, shrinks ×½ when a quarter
full (never below 8). `capacity()` exposes the machinery, which makes it a
nice tool for teaching amortised analysis.

| Method | Returns | Notes |
|---|---|---|
| `append(v)` | null | amortised O(1) |
| `pop()` | the last value | AssertionError when empty |
| `get(i)` | the value at `i` | 0-based; AssertionError out of range |
| `set(i, v)` | null | overwrites; AssertionError out of range |
| `insert(i, v)` | null | `0 ≤ i ≤ size`; shifts the tail right |
| `remove_at(i)` | the removed value | shifts the tail left |
| `remove(v)` | boolean | On if the first occurrence was removed |
| `contains(v)` | boolean | O(n) scan of live items |
| `index_of(v)` | integer or null | first index; null when absent |
| `size()` | integer | |
| `capacity()` | integer | length of the backing list |
| `is_empty()` | boolean | |
| `to_list()` | list | snapshot, live items only |
| `clear()` | null | resets the capacity to 8 |

```arc
bind a = dynamic_array()
a["append"](1)
a["append"](2)
a["insert"](1, 99)
cast a["to_list"]()      ? [1, 99, 2]
cast a["pop"]()          ? 2
cast a["capacity"]()     ? 8
```

## SinglyLinkedList

`singly_linked_list()` — nodes `{"v": v, "next": ...}` with head and tail
pointers. `push_back` is O(1) via the tail; `pop_back` is O(n) because a
singly linked list must find the second-to-last node.

| Method | Returns | Notes |
|---|---|---|
| `push_front(v)` | null | O(1) |
| `push_back(v)` | null | O(1) via the tail pointer |
| `pop_front()` | value | AssertionError when empty |
| `pop_back()` | value | O(n); AssertionError when empty |
| `front()` | value | AssertionError when empty |
| `back()` | value | AssertionError when empty |
| `get(i)` | value | O(i) walk; AssertionError out of range |
| `insert_at(i, v)` | null | `0 ≤ i ≤ size` |
| `remove_at(i)` | the removed value | |
| `remove(v)` | boolean | first occurrence |
| `contains(v)` | boolean | |
| `index_of(v)` | integer or null | |
| `reverse()` | null | in place, O(n) |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `to_list()` | list | front … back |
| `clear()` | null | |

```arc
bind l = singly_linked_list()
l["push_back"](1)
l["push_front"](0)
l["reverse"]()
cast l["to_list"]()      ? [1, 0]
```

## DoublyLinkedList

`doubly_linked_list()` — nodes `{"v", "prev", "next"}` with head and
tail. Same API as the singly linked list, but `pop_back` is O(1), plus one
extra method.

| Method | Returns | Notes |
|---|---|---|
| `push_front(v)` | null | |
| `push_back(v)` | null | |
| `pop_front()` | value | AssertionError when empty |
| `pop_back()` | value | O(1), unlike the singly list |
| `front()` | value | |
| `back()` | value | |
| `get(i)` | value | O(i) walk from the head |
| `insert_at(i, v)` | null | `0 ≤ i ≤ size` |
| `remove_at(i)` | the removed value | |
| `remove(v)` | boolean | first occurrence; O(1) unlink after the search |
| `contains(v)` | boolean | |
| `index_of(v)` | integer or null | |
| `reverse()` | null | in place, O(n), keeps both directions intact |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `to_list()` | list | front … back |
| `to_list_reversed()` | list | back … front, O(n) |
| `clear()` | null | |

```arc
bind l = doubly_linked_list()
for i in range(0, 4) { l["push_back"](i) }
cast l["to_list"]()           ? [0, 1, 2, 3]
cast l["to_list_reversed"]()  ? [3, 2, 1, 0]
```

## HashMap

`hash_map()` — a **real bucketed hash table** (separate chaining), not a
wrapper over an Arcana map. Integers are mixed through big multiplications,
texts get a polynomial rolling hash over code points; buckets double and
rehash past a 0.75 load factor. A parallel key list preserves insertion order,
so `keys()`, `values()` and `items()` come out in insertion order — which also
means `delete` is O(size) (the order list is rebuilt).

| Method | Returns | Notes |
|---|---|---|
| `set(k, v)` | null | insert or update |
| `get(k, default = null)` | value | `default` when the key is absent |
| `has(k)` | boolean | |
| `delete(k)` | boolean | On if the key was present |
| `size()` | integer | |
| `keys()` | list | insertion order |
| `values()` | list | insertion order |
| `items()` | list of `[k, v]` | insertion order |
| `clear()` | null | resets to 8 buckets |

```arc
bind m = hash_map()
m["set"]("gem", 3)
m["set"](7, "seven")         ? integer and text keys coexist
cast m["keys"]()             ? ["gem", 7]
cast m["get"]("gem", 0)      ? 3
cast m["delete"]("gem")      ? On
```

## HashSet

`hash_set()` — the same bucketing machinery with values only (they are
the keys). `to_list()` makes **no order promise**; sort it if you need one.

| Method | Returns | Notes |
|---|---|---|
| `add(v)` | boolean | On when newly added, Off when already present |
| `has(v)` | boolean | |
| `delete(v)` | boolean | |
| `size()` | integer | |
| `to_list()` | list | unordered |
| `clear()` | null | |

```arc
bind seen = hash_set()
cast seen["add"]("gem")      ? On
cast seen["add"]("gem")      ? Off
cast seen["has"]("gem")      ? On
```

## OrderedMap

`ordered_map()` — a thin, documented wrapper over Arcana's own
insertion-ordered map. Cheaper constants than `HashMap` for small maps; the
trade-off is that `delete` is O(size), because Arcana maps have no in-place
key deletion.

| Method | Returns | Notes |
|---|---|---|
| `set(k, v)` | null | insert or update |
| `get(k, default = null)` | value | |
| `has(k)` | boolean | |
| `delete(k)` | boolean | O(size): rebuilds the map without the key |
| `keys()` | list | insertion order |
| `values()` | list | insertion order |
| `items()` | list of `[k, v]` | insertion order |
| `size()` | integer | |
| `clear()` | null | |

## OrderedSet

`ordered_set()` — an insertion-ordered set over an Arcana map.

| Method | Returns | Notes |
|---|---|---|
| `add(v)` | boolean | On when newly added |
| `has(v)` | boolean | |
| `delete(v)` | boolean | O(size) rebuild |
| `to_list()` | list | insertion order |
| `size()` | integer | |
| `clear()` | null | |

```arc
bind s = ordered_set()
s["add"](3)
s["add"](1)
s["add"](2)
cast s["to_list"]()          ? [3, 1, 2] — insertion, not sorted, order
```

## DSU

`dsu(n)` — disjoint set union (union-find) over the integer elements
`0..n-1`; `n` must be an integer ≥ 0. Union by size with full path
compression, so `find`/`union` run in amortised O(α(n)) — effectively O(1).

| Method | Returns | Notes |
|---|---|---|
| `find(x)` | integer | root of `x`; AssertionError out of range |
| `union(x, y)` | boolean | On when two components merged, Off when already together |
| `connected(x, y)` | boolean | |
| `count()` | integer | number of components |
| `component_size(x)` | integer | size of `x`'s component |
| `size()` | integer | `n` |
| `components()` | list of lists | each ascending; groups ordered by smallest member |

```arc
bind d = dsu(5)
d["union"](0, 2)
d["union"](2, 4)
cast d["connected"](0, 4)    ? On
cast d["count"]()            ? 3
cast d["components"]()       ? [[0, 2, 4], [1], [3]]
```

## BloomFilter

`bloom_filter(expected = 1000, rate = 0.01)` — a probabilistic set sized
for `expected` items at target rate `rate`: bits
`m = ceil(-expected · ln(rate) / ln(2)²)` and `k = max(1, round(m/expected · ln(2)))`
hash functions, combined by double hashing `h_i = (h1 + i·h2 + i²) mod m`.
**False positives are possible; false negatives never are.**

| Method | Returns | Notes |
|---|---|---|
| `add(item)` | null | text or integer items only |
| `might_contain(item)` | boolean | On means "possibly present" |
| `bit_count()` | integer | number of set bits |
| `capacity()` | integer | `m`, the number of bits |
| `hash_count()` | integer | `k`, hashes per item |
| `estimated_fp()` | decimal | `1 - (1 - 1/m)^(k·adds)`: the expected fraction of set bits; raise it to the `k`-th power for the classic false-positive probability |
| `adds()` | integer | how many `add()` calls so far |
| `clear()` | null | resets the bits and the counter |

```arc
bind bf = bloom_filter(1000, 0.01)
bf["add"]("gem")
cast bf["might_contain"]("gem")    ? On — never a false negative
cast bf["might_contain"]("ghost")  ? usually Off, sometimes On
cast bf["hash_count"]()            ? 7
```

## LRUCache

`lru_cache(cap)` — a least-recently-used cache with a **true O(1)
core**: a slot array of `[key, value, prev_idx, next_idx]` nodes forms a
doubly linked recency chain (`head` = most recent, `tail` = victim), a free
list recycles slots, and an Arcana map `key -> slot` gives O(1) lookup.
Evicted/deleted keys leave stale map entries (Arcana maps cannot delete in
place); they are detected lazily and the map is compacted from the live chain
amortised O(1).

| Method | Returns | Notes |
|---|---|---|
| `get(k, default = null)` | value | **touches**: moves `k` to most-recent |
| `put(k, v)` | null | touches or inserts; silently evicts the LRU item when `size` would exceed `cap` |
| `has(k)` | boolean | does **not** touch recency |
| `delete(k)` | boolean | |
| `size()` | integer | |
| `capacity()` | integer | |
| `keys()` | list | MRU-first |
| `to_list()` | list of `[k, v]` | MRU-first |
| `clear()` | null | |

```arc
bind c = lru_cache(2)
c["put"]("a", 1)
c["put"]("b", 2)
c["get"]("a")            ? touch "a"
c["put"]("c", 3)         ? evicts "b"
cast c["keys"]()         ? ["c", "a"]
```

## LFUCache

`lfu_cache(cap)` — a least-frequently-used cache with the same API and
the same slot-array storage. Each key tracks a frequency and a last-use stamp
(one global counter); eviction scans all live slots for the lowest frequency,
ties broken by the oldest stamp. The scan makes **`get`/`put` O(capacity)
worst case** — the honest price of exact LFU without per-frequency bucket
lists. `put` on an existing key refreshes the value and bumps the frequency;
`get` bumps it too.

| Method | Returns | Notes |
|---|---|---|
| `get(k, default = null)` | value | touches: frequency +1, new stamp, most-recent |
| `put(k, v)` | null | touches or inserts; evicts (lowest frequency, oldest stamp) when `size` would exceed `cap` |
| `has(k)` | boolean | does not touch frequency or recency |
| `delete(k)` | boolean | |
| `size()` | integer | |
| `capacity()` | integer | |
| `keys()` | list | MRU-first |
| `to_list()` | list of `[k, v]` | MRU-first |
| `clear()` | null | |

```arc
bind c = lfu_cache(3)
c["put"]("a", 1)
c["put"]("b", 2)
c["put"]("c", 3)
c["get"]("a")
c["get"]("a")
c["put"]("d", 4)         ? evicts "b": frequency 1, oldest stamp
cast c["has"]("b")       ? Off
```

## SkipList

`skip_list(max_level = 16, p = 0.5)` — a probabilistic **sorted** map.
Nodes are `{"key", "val", "forward"}` with one forward pointer per level off a
head sentinel; a node's level is `1 +` the number of consecutive
`random() < p` draws, capped at `max_level` (assert 1–32; assert
`0 < p < 1`). Keys must be comparable: all numbers or all texts — the first
offending call raises AssertionError.

| Method | Returns | Notes |
|---|---|---|
| `add(key, value = null)` | boolean | On when the key is new; does **not** overwrite an existing value (use `set`) |
| `set(key, value)` | null | insert or overwrite |
| `get(key, default = null)` | value | |
| `has(key)` | boolean | |
| `delete(key)` | boolean | |
| `size()` | integer | |
| `first()` | key | smallest; AssertionError when empty |
| `last()` | key | largest; AssertionError when empty |
| `range(start, end)` | list of keys | `start ≤ key < end`, ascending |
| `to_list()` | list | sorted keys |
| `items()` | list of `[k, v]` | sorted by key |
| `clear()` | null | |

```arc
bind s = skip_list()
s["add"]("pear", 1)
s["add"]("apple", 2)
s["add"]("fig", 3)
cast s["to_list"]()          ? ["apple", "fig", "pear"]
cast s["range"]("b", "q")    ? ["fig", "pear"]
cast s["last"]()             ? pear
```

## Complexity summary

"Expected" = probabilistic (hashing, skip list levels); "amortised" = spread
over many operations.

| Structure | Main operations | Notes |
|---|---|---|
| DynamicArray | `append`/`pop` amortised O(1); `get`/`set` O(1); `insert`/`remove_at` O(n) | doubles when full, halves at ¼ full |
| SinglyLinkedList | `push_front`/`push_back`/`pop_front` O(1); `pop_back` O(n); `get`/`insert_at`/`remove_at` O(i); `reverse` O(n) | |
| DoublyLinkedList | as the singly list, but `pop_back` O(1) | |
| HashMap | `set`/`get`/`has` expected O(1) amortised; `delete` O(n) | delete rebuilds the insertion-order list |
| HashSet | `add`/`has`/`delete` expected O(1) amortised | |
| OrderedMap | `set`/`get`/`has` O(1) expected; `delete` O(n) | wrapper over an Arcana map |
| OrderedSet | `add`/`has` O(1) expected; `delete` O(n) | |
| DSU | `find`/`union`/`connected`/`component_size` amortised O(α(n)); `components` O(n·α(n)) | union by size + path compression |
| BloomFilter | `add`/`might_contain` O(k); `bit_count`/`clear` O(m) | k is 7 for the default sizes |
| LRUCache | `get`/`put`/`has`/`delete` amortised O(1); `keys`/`to_list` O(capacity) | slot array + recency chain |
| LFUCache | `get`/`put` O(capacity) worst case; `has`/`delete` O(1); `keys`/`to_list` O(capacity) | exact LFU via an eviction scan |
| SkipList | `add`/`set`/`get`/`has`/`delete`/`last` expected O(log n); `first` O(1); `range` O(log n + hits) | probabilistic levels |

## Error behaviour

Every misuse raises catchable `E2101` (`AssertionError`) with a `Candy: ...`
message:

* dynamic array: `pop` on empty; `get`/`set`/`remove_at` with an index outside
  `0..size-1` (negatives included); `insert` outside `0..size`; a non-integer
  index;
* linked lists: `pop_front`/`pop_back`/`front`/`back` on empty; `get`/
  `remove_at` out of range; `insert_at` outside `0..size`; a non-integer index;
* hash map / ordered map: `set`/`get`/`has`/`delete` with a key that is not
  text or integer;
* hash set / ordered set: `add`/`has`/`delete` with a value that is not text
  or integer;
* DSU: `dsu` with a non-integer or negative `n`; `find`/`union`/
  `connected`/`component_size` with a non-integer or out-of-range element;
* bloom filter: `expected` not an integer or below 1; `rate` not a number in
  `(0, 1)`; `add`/`might_contain` with a non-text/integer item;
* LRU / LFU cache: a capacity that is not an integer ≥ 1;
* skip list: `max_level` not an integer in 1–32; `p` not a number in `(0, 1)`;
  `first`/`last` of an empty list; a key that is neither number nor text, or a
  key whose family (numbers vs texts) differs from earlier keys.

Arity and type mistakes in *your* call (`m["get"]()` with no argument,
`a["size"](1)`) are ordinary Arcana errors (`E3004`/`E3003`) and are not
catchable — they are programmer errors, exactly like the core language treats
them.
