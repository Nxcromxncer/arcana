# Candy

A data-structures and algorithms library for **Arcana**, named for code that is
small, sweet and self-contained. Tier I covers the four core families:

| Module | Grimoire | Provides |
|---|---|---|
| 01 `structures` | `candy/structures.arc` | Stack, Queue, Deque, CircularQueue, PriorityQueue, MonotonicStack, MonotonicQueue, MinStack, MaxStack |
| 02 `trees` | `candy/trees.arc` | Binary Tree, BST, AVL, Red-Black, Binary Heap, Trie, Segment Tree, Fenwick Tree, B-Tree, B+ Tree, Suffix Array (suffix-tree queries), Treap |
| 03 `collections` | `candy/collections.arc` | Dynamic Array, Singly/Doubly Linked List, HashMap, HashSet, Ordered Map/Set, DSU, Bloom Filter, LRU/LFU Cache, Skip List |
| 04 `graphs` | `candy/graphs.arc` | Directed/undirected weighted graph (adjacency list/matrix/edge list views), BFS, DFS, paths, Dijkstra, connected components, cycle detection, bipartite checking, topological sort, SCC |

Every module is pure Arcana — no new interpreter features, no Rust changes —
and runs on the plain 1.0 toolchain (`arcana run` / `arcana check`).

## Using Candy

The `candy.arc` grimoire sits next to the `candy/` folder and loads everything:

```arc
import candy

spell main() {
    bind s = candy.structures.stack()
    s["push"](21)
    bind t = candy.trees.avl()
    for x in [5, 3, 8, 1] { t["insert"](x) }
    cast candy.graphs.is_bipartite(candy.graphs.graph())
    cast s["pop"]() + t["min"]()
}
```

Or import only what you need:

```arc
import candy.structures            ? structures.stack()
import candy.trees as tr           ? tr.bst()
from candy.collections import hash_map
from candy.graphs import graph, bfs, dijkstra
```

Resolution rules: a path like `candy.trees` maps to the file `candy/trees.arc`
looked up next to the importing file, then in every folder on `ARCANA_PATH`.
If your own file lives *inside* the `candy/` folder, import the module by its
bare name (`import trees`).

## How structures look

Arcana 1.0 has no user-defined types, so every Candy structure is a **closure
object**: a constructor spell returns a map of spells sharing private state.
Each instance carries a `"kind"` tag, and misuse raises a catchable
`AssertionError` whose message starts with `Candy:`:

```arc
bind q = priority_queue()
q["push"]("email", 3)
q["push"]("alarm", 1)
cast q["pop"]()                    ? alarm

try {
    bind s = stack()
    s["pop"]()
} catch e {
    cast e["message"]              ? Candy: pop from an empty stack
}
```

The full conventions (mutate-vs-return, comparability, O(1) guarantees) are in
[`docs/01-structures.md`](docs/01-structures.md).

## Testing

Each module ships a self-contained test grimoire:

```
arcana run candy/test_structures.arc     ? ALL STRUCTURES TESTS PASSED
arcana run candy/test_trees.arc          ? ALL TREES TESTS PASSED
arcana run candy/test_collections.arc    ? ALL COLLECTIONS TESTS PASSED
arcana run candy/test_graphs.arc         ? ALL GRAPHS TESTS PASSED
```

Every constructor and method is documented in [`docs/`](docs/README.md), with a
method table and runnable examples per structure.
