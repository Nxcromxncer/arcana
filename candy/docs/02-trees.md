# Candy 02 — `trees`

Tree-shaped data structures for Arcana: a positional complete tree, an
unbalanced BST, two self-balancing search trees (AVL and red-black), a
binary heap, a trie, two range-sum trees (segment and Fenwick), a B-tree,
a B+ tree, a suffix-array text index and a randomized treap.

```arc
import candy.trees

spell main() {
    bind t = candy.trees.bst()
    t["insert"](5)
    t["insert"](2)
    t["insert"](8)
    cast t["to_list"]()                ? [2, 5, 8]
    cast t["remove"](2)                ? On
    cast t["to_list"]()                ? [5, 8]
}
```

## Importing

```arc
import candy.trees                  ? full name
import candy.trees as tr             ? aliased
from candy.trees import bst     ? named imports
import trees                         ? when this file sits next to yours
```

The full import table (project root, `ARCANA_PATH`, aliased and `from`
imports, the `candy` aggregator) is in [01-structures.md](01-structures.md);
every spelling that works for `candy.structures` works for `candy.trees`.

## Conventions shared by every Candy tree

1. **Trees are closure objects.** A constructor such as `bst()` returns
   a map of spells sharing private state; call methods by indexing:
   `t["insert"](7)`. There are no classes in Arcana 1.0.
2. **Every instance has a `"kind"` tag** — `"candy/bst"`, `"candy/b-tree"`,
   ... — so programs can tell structures apart.
3. **Misuse raises catchable `AssertionError` (E2101)** with a message that
   starts with `Candy:`.
4. **Ordered keys must be comparable.** BST, AVL, RedBlackTree, BinaryHeap,
   BTree, BPlusTree and Treap order their keys: the keys must be all numbers
   or all texts. The first key that breaks this rule (or is not a number or
   text at all) raises `AssertionError: Candy: keys must be all numbers or
   all texts` — checked when a key is inserted, and when a later
   contains/remove/min/max call would compare against a different type.
5. **No deep recursion.** Arcana caps call depth at 1000, so every operation
   that can walk a long chain (plain BST walks, heap sifts, trie walks,
   suffix searches, segment/Fenwick loops) is iterative; only guaranteed
   O(log n) recursion is used (AVL/RB/treap rebalancing, B-tree descent).
6. **Snapshots are fresh.** `to_list()` always returns a new list, never
   internal state.

---

## BinaryTree

`binary_tree()` — a **complete** binary tree filled in level order
(array-backed: children of slot `i` live at `2i+1` and `2i+2`). It is
positional, *not* a search tree: `insert(v)` appends at the next level-order
slot and there is deliberately no remove. `height()` is 0 when empty and 1
for a single node.

| Method | Returns | Notes |
|---|---|---|
| `insert(v)` | null | appends at the next level-order slot |
| `contains(v)` | boolean | linear scan; values may be any type |
| `size()` | integer | |
| `height()` | integer | empty = 0, single node = 1 |
| `is_empty()` | boolean | |
| `to_list()` | list | level order, compact (no trailing nulls) |
| `level_order()` | list | same as `to_list()` |
| `inorder()` | list | iterative, over the implicit array |
| `preorder()` | list | iterative |
| `postorder()` | list | iterative |
| `clear()` | null | |

```arc
bind bt = binary_tree()
for i in [1, 2, 3, 4, 5, 6, 7] { bt["insert"](i) }
cast bt["to_list"]()     ? [1, 2, 3, 4, 5, 6, 7]
cast bt["inorder"]()     ? [4, 2, 5, 1, 6, 3, 7]
cast bt["height"]()      ? 3
```

## BST

`bst()` — unbalanced binary search tree. `insert`, `remove`,
`contains` and the traversals are **iterative**, so even a degenerate chain
of thousands of nodes never hits the recursion limit. The two-child remove
case uses the inorder successor.

| Method | Returns | Notes |
|---|---|---|
| `insert(v)` | boolean | `On` if added, `Off` if duplicate |
| `remove(v)` | boolean | `On` if removed, `Off` if absent |
| `contains(v)` | boolean | |
| `min()` / `max()` | value | AssertionError when empty |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `height()` | integer | empty = 0; a chain of n has height n |
| `inorder()` | list | sorted |
| `preorder()` | list | |
| `postorder()` | list | |
| `to_list()` | list | = `inorder()` |
| `clear()` | null | |

```arc
bind b = bst()
for v in [5, 2, 8, 2] { cast b["insert"](v) }   ? On On On Off
cast b["to_list"]()      ? [2, 5, 8]
cast b["remove"](2)      ? On
cast b["min"]()          ? 5
```

## AVL

`avl()` — a self-balancing BST. Node heights are maintained exactly, so
`height()` is exact and every operation is O(log n) even on sorted input.

| Method | Returns | Notes |
|---|---|---|
| `insert(v)` | boolean | `On` if added, `Off` if duplicate |
| `remove(v)` | boolean | `On` if removed, `Off` if absent |
| `contains(v)` | boolean | |
| `min()` / `max()` | value | AssertionError when empty |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `height()` | integer | exact; ≤ 1.45 · log₂(n + 2) |
| `is_balanced()` | boolean | re-verifies every node's stored height and balance |
| `inorder()` | list | sorted |
| `preorder()` | list | |
| `postorder()` | list | |
| `to_list()` | list | = `inorder()` |
| `clear()` | null | |

```arc
bind a = avl()
for i in range(0, 1000) { a["insert"](i) }   ? sorted input, no problem
cast a["height"]()       ? 10 (the worst case for 1000 keys is 14)
cast a["is_balanced"]()  ? On
```

## RedBlackTree

`red_black_tree()` — a left-leaning red-black tree with both insert and
remove (null links count as black). Supports the full ordered-set API and
`is_valid()`, which re-checks the three classic invariants: black root, no
red parent with a red child, equal black height on every root-to-null path.

| Method | Returns | Notes |
|---|---|---|
| `insert(v)` | boolean | `On` if added, `Off` if duplicate |
| `remove(v)` | boolean | `On` if removed, `Off` if absent |
| `contains(v)` | boolean | |
| `min()` / `max()` | value | AssertionError when empty |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `height()` | integer | ≤ 2 · log₂(n + 1) |
| `inorder()` | list | sorted |
| `to_list()` | list | = `inorder()` |
| `is_valid()` | boolean | structural self-check, used by tests |
| `clear()` | null | |

```arc
bind r = red_black_tree()
for v in [9, 4, 16, 1] { r["insert"](v) }
cast r["remove"](4)      ? On
cast r["is_valid"]()     ? On
cast r["to_list"]()      ? [1, 9, 16]
```

## BinaryHeap

`binary_heap(descending = Off)` — a binary **min**-heap (max-heap with
`On`). `binary_heap_from(items, descending = Off)` heapifies an
existing list bottom-up in O(n) instead of pushing one by one. Values must
be all numbers or all texts (see conventions).

| Method | Returns | Notes |
|---|---|---|
| `push(v)` | null | |
| `pop()` | value | the extreme; AssertionError when empty |
| `peek()` | value | without removing; AssertionError when empty |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `to_list()` | list | snapshot of the internal array in **heap order — not sorted** |
| `clear()` | null | |

Both constructors return the same method set.

```arc
bind h = binary_heap_from([5, 3, 8, 1])
cast h["pop"]()          ? 1
cast h["peek"]()         ? 3
bind m = binary_heap(On)
m["push"](2)
m["push"](7)
cast m["pop"]()          ? 7
```

## Trie

`trie()` — prefix tree over **texts**. Words live on nodes marked
`end`; remove() prunes branches that became useless. The empty text is a
legal word. `words_with_prefix` and `to_list` return their words sorted.

| Method | Returns | Notes |
|---|---|---|
| `insert(word)` | null | duplicates do not grow the trie |
| `remove(word)` | boolean | `On` if removed, `Off` if absent; prunes dead branches |
| `contains(word)` | boolean | whole-word test |
| `starts_with(prefix)` | boolean | `On` even when no word ends there yet |
| `words_with_prefix(prefix)` | list | sorted |
| `size()` | integer | number of stored words |
| `is_empty()` | boolean | |
| `to_list()` | list | all words, sorted |
| `longest_common_prefix()` | text | `""` when the trie is empty or branches |
| `clear()` | null | |

```arc
bind t = trie()
for w in ["cat", "car", "card"] { t["insert"](w) }
cast t["starts_with"]("ca")          ? On
cast t["words_with_prefix"]("car")   ? ["car", "card"]
cast t["remove"]("car")              ? On
cast t["contains"]("card")           ? On
```

## SegmentTree

`segment_tree(values)` — range-**sum** tree over a numeric list,
iterative bottom-up on a size-2n array. The empty list is allowed (all
queries then return 0). `query(start, end)` sums the half-open range
`[start, end)`.

| Method | Returns | Notes |
|---|---|---|
| `size()` | integer | |
| `get(i)` | number | AssertionError on a bad index |
| `update(i, v)` | null | point assign; recomputes ancestors |
| `query(start, end)` | number | sum over `[start, end)`; AssertionError unless `0 <= start <= end <= size` |
| `to_list()` | list | the current values |
| `clear()` | null | |

```arc
bind s = segment_tree([2, 1, 5, 0, 3])
cast s["query"](1, 4)    ? 6
s["update"](3, 10)
cast s["query"](1, 4)    ? 16
```

## FenwickTree

`fenwick_tree(source)` — binary indexed tree for prefix sums. Pass an
integer `n` for n zeros, or a numeric list for an O(n) build. The API is
0-based over 1-based storage; `prefix(i)` sums `[0, i)` and `range_sum(s,
e)` sums `[s, e)`.

| Method | Returns | Notes |
|---|---|---|
| `add(i, delta)` | null | adds delta to element `i` |
| `prefix(i)` | number | sum of `[0, i)` |
| `range_sum(start, end)` | number | sum of `[start, end)` |
| `size()` | integer | |
| `to_list()` | list | the current values |
| `clear()` | null | |

Index/range violations raise AssertionError.

```arc
bind f = fenwick_tree([2, 1, 5])
cast f["prefix"](3)      ? 8
f["add"](1, 10)
cast f["range_sum"](0, 2)  ? 13
```

## BTree

`b_tree(min_degree = 2)` — the CLRS B-tree; `t = min_degree` must be an
integer ≥ 2. Every non-root node holds `t-1..2t-1` keys. insert() splits
full nodes on the way down; remove() borrows from siblings or merges so no
node is left underfull.

| Method | Returns | Notes |
|---|---|---|
| `insert(key)` | boolean | `On` if added, `Off` if duplicate |
| `contains(key)` | boolean | |
| `remove(key)` | boolean | `On` if removed, `Off` if absent |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `height()` | integer | empty = 0 |
| `to_list()` | list | sorted |
| `is_valid()` | boolean | checks key counts, sortedness, equal leaf depth |
| `clear()` | null | |

```arc
bind b = b_tree(2)
for v in [10, 20, 5, 6, 12] { b["insert"](v) }
cast b["to_list"]()      ? [5, 6, 10, 12, 20]
cast b["remove"](10)     ? On
cast b["is_valid"]()     ? On
```

## BPlusTree

`bplus_tree(min_degree = 2)` — a B+ tree: internal nodes hold only
separator keys, **all data lives in linked leaves**, so range scans walk a
leaf chain. Inserting an existing key updates its value. `range_query(s,
e)` returns the values of keys with `s <= key < e` (half-open).

| Method | Returns | Notes |
|---|---|---|
| `insert(key, value)` | null | updates the value when the key exists |
| `get(key, default = null)` | value | `default` when the key is absent |
| `has(key)` | boolean | |
| `remove(key)` | boolean | `On` if removed, `Off` if absent |
| `range_query(start, end)` | list | values with `start <= key < end` |
| `to_list()` | list | `[key, value]` pairs sorted by key |
| `items()` | list | same as `to_list()` |
| `size()` | integer | number of distinct keys |
| `is_empty()` | boolean | |
| `height()` | integer | empty = 0 |
| `is_valid()` | boolean | checks counts, separators, leaf chain, depth |
| `clear()` | null | |

```arc
bind bp = bplus_tree(2)
for k in range(0, 5) { bp["insert"](k, k * 10) }
cast bp["range_query"](1, 4)   ? [10, 20, 30]
cast bp["get"](2)              ? 20
cast bp["remove"](2)           ? On
```

## SuffixArray

`suffix_array(text)` — **suffix-array based** (with binary searches
over the sorted suffixes); it answers the same queries a suffix tree would.
The text must have length ≥ 1. `occurrences` counts the positions where a
pattern starts; `longest_repeating_substring` uses the maximum LCP between
adjacent sorted suffixes (ties break to the first in sorted order).

| Method | Returns | Notes |
|---|---|---|
| `contains(pattern)` | boolean | substring test via binary search |
| `contains_suffix(s)` | boolean | whole-suffix test |
| `occurrences(pattern)` | integer | number of starting positions |
| `longest_repeating_substring()` | text | `""` when nothing repeats |
| `size()` | integer | length of the text |
| `suffixes()` | list | the raw suffix array (suffix start indexes) |
| `clear()` | null | |

Empty or non-text patterns raise AssertionError (`Candy: empty pattern`).

```arc
bind sa = suffix_array("banana")
cast sa["occurrences"]("ana")            ? 2
cast sa["contains_suffix"]("ana")        ? On
cast sa["longest_repeating_substring"]() ? ana
```

## Treap

`treap(seed = null)` — a randomized search tree kept balanced by
random priorities via split/merge (expected O(log n) per operation). If
`seed` is an integer, `random_seed(seed)` runs at construction — note this
reseeds Arcana's **global** random generator.

| Method | Returns | Notes |
|---|---|---|
| `insert(key)` | boolean | `On` if added, `Off` if duplicate |
| `remove(key)` | boolean | `On` if removed, `Off` if absent |
| `contains(key)` | boolean | |
| `min()` / `max()` | value | AssertionError when empty |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `height()` | integer | empty = 0; expected O(log n) |
| `to_list()` | list | sorted |
| `clear()` | null | |

```arc
bind tp = treap(42)
tp["insert"](3)
tp["insert"](1)
tp["insert"](2)
cast tp["to_list"]()     ? [1, 2, 3]
```

## Complexity summary

| Structure | insert | remove | contains/query | notes |
|---|---|---|---|---|
| BinaryTree | O(1) append | — (positional) | O(n) scan | traversals O(n); height O(log n) |
| BST | O(h), O(n) worst | O(h) | O(h) | iterative; h can be n on sorted input |
| AVL | O(log n) | O(log n) | O(log n) | `is_balanced` O(n) |
| RedBlackTree | O(log n) | O(log n) | O(log n) | `is_valid` O(n) |
| BinaryHeap | push O(log n) | pop O(log n) | peek O(1) | `from`-build O(n); `to_list` O(n) |
| Trie | O(m) | O(m) | O(m) | m = word length; `words_with_prefix` O(m + k²) for k results |
| SegmentTree | build O(n) | update O(log n) | query O(log n) | `get` O(1) |
| FenwickTree | build O(n) | add O(log n) | prefix/range O(log n) | `to_list` O(n log n) |
| BTree | O(log n) | O(log n) | O(log n) | `is_valid` O(n) |
| BPlusTree | O(log n) | O(log n) | O(log n) | `range_query` O(log n + output) |
| SuffixArray | O(n³) build worst | — | O(m · log n) | build insertion-sorts the suffixes; `contains_suffix` O(n·m); LRS O(n²) |
| Treap | O(log n) expected | O(log n) expected | O(log n) expected | priorities from the global random generator |

## Error behaviour

Every misuse raises catchable `E2101` (`AssertionError`) with a `Candy: ...`
message:

* `min` / `max` on an empty BST, AVL, RedBlackTree or Treap;
* `pop` / `peek` on an empty BinaryHeap;
* the first key that breaks the all-numbers-or-all-texts rule
  (`Candy: keys must be all numbers or all texts`) on BST, AVL,
  RedBlackTree, BinaryHeap, BTree, BPlusTree or Treap — on insert, or on a
  later `contains` / `remove` / `min` / `max` that would compare across
  types;
* `b_tree` / `bplus_tree` with a `min_degree` below 2 or of
  non-integer type;
* `binary_heap_from` with a non-list argument;
* segment tree: `get` / `update` with an out-of-range or non-integer index,
  `query` with a range violating `0 <= start <= end <= size`, and non-number
  values in `values` or `update`;
* fenwick tree: `add` / `prefix` / `range_sum` with out-of-range or
  non-integer indexes, a negative size, non-number values, or a source that
  is neither an integer nor a list;
* trie: non-text words or prefixes;
* suffix array: a text of length 0, and empty (`Candy: empty pattern`) or
  non-text patterns in `contains` / `contains_suffix` / `occurrences`.

Arity and type mistakes in *your* call (`t["insert"]()` with no argument,
`t["size"](1)`) are ordinary Arcana errors (`E3004`/`E3003`) and are not
catchable — they are programmer errors, exactly like the core language
treats them.
