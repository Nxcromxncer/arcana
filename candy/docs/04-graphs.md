# Candy 04 — `graphs`

Labelled graphs and the classic algorithms over them for Arcana: breadth-first
and depth-first search, shortest paths (BFS and Dijkstra), connected
components, cycle detection, bipartiteness, topological sorting and strongly
connected components.

```arc
import candy.graphs

spell main() {
    bind g = candy.graphs.graph()
    g["add_edge"]("paris", "lyon", 4)
    g["add_edge"]("lyon", "nice", 2)
    cast candy.graphs.bfs(g, "paris")              ? ["paris", "lyon", "nice"]
    cast candy.graphs.shortest_path(g, "paris", "nice")   ? [6, ["paris", "lyon", "nice"]]
}
```

## Importing

| Your file sits...             | Import                  |
|-------------------------------|-------------------------|
| in the `arcana` project root  | `import candy.graphs`   |
| next to a `candy/` folder     | `import candy.graphs`   |
| next to the module itself     | `import graphs`         |

```arc
import candy.graphs                ? full name
import candy.graphs as gr          ? aliased
from candy.graphs import graph, bfs
import graphs                      ? when this file sits next to yours
```

See [01-structures.md](01-structures.md) for the full import table and the
rules behind it.

## Conventions shared with every Candy structure

1. **The graph is a closure object.** `graph()` returns a map of spells
   sharing private state; call methods by indexing: `g["add_edge"]("a", "b")`.
2. **The instance has a `"kind"` tag**: `"candy/graph"` when undirected,
   `"candy/digraph"` when `graph` was called with `On`.
3. **Vertex labels are texts or integers** (anything a map key can be). Mixed
   labels are fine; `1` and `"1"` are different vertices.
4. **Insertion order is preserved everywhere**: `vertices()` returns labels in
   the order they were added, `neighbors(v)` in the order edges to them were
   added, and every traversal uses that order.
5. **Storage is the adjacency list itself**: a map `label -> (map label ->
   weight)`, so undirected edges are stored in both endpoints' maps.
6. **Misuse raises catchable `AssertionError` (E2101)** with a message that
   starts with `Candy:` — see [Error behaviour](#error-behaviour).
7. **Every traversal is iterative** (explicit stack/queue lists), because
   Arcana limits recursion to about 1000 frames. A 2000-vertex chain is
   routine work for this module.

---

## graph

`graph(directed = Off)` — a graph whose edges carry numeric weights
(default 1). With `directed = On` you get a digraph: arcs have exactly one
direction. Self-loops are allowed; re-adding an edge replaces its weight.

| Method | Returns | Notes |
|---|---|---|
| `add_vertex(v)` | boolean | `On` if new, `Off` if already present |
| `add_edge(u, v, w = 1)` | null | adds missing endpoints automatically |
| `remove_edge(u, v)` | boolean | `On`/`Off`; AssertionError when `u` or `v` is unknown; undirected removes both directions |
| `remove_vertex(v)` | boolean | `On`/`Off` (Off when absent); drops incident edges |
| `has_vertex(v)` | boolean | |
| `has_edge(u, v)` | boolean | `Off` when either endpoint is unknown |
| `weight(u, v)` | number or null | null when the edge is absent; AssertionError when `u` or `v` is unknown |
| `vertices()` | list | insertion order |
| `neighbors(v)` | list | out-neighbors in insertion order; AssertionError when unknown |
| `in_degree(v)` | integer | undirected: equals `out_degree` and `degree` |
| `out_degree(v)` | integer | undirected: equals `in_degree` and `degree` |
| `degree(v)` | integer | undirected: the degree; directed: `in_degree + out_degree` (a self-loop counts twice) |
| `size()` | integer | number of vertices |
| `edge_count()` | integer | undirected counts each edge once (self-loops once) |
| `edges()` | list of `[u, v, w]` | directed: every arc; undirected: each edge once, reported at the endpoint that comes first in vertex order (self-loops at their own vertex) |
| `adjacency_list()` | map `label -> list` | fresh copy, neighbor labels in order |
| `adjacency_matrix()` | map | `{"labels": [...], "matrix": [[weight-or-null]]}` — row `i`, column `j` |
| `reverse()` | new graph | directed: every arc flipped; undirected: an exact structural copy (same vertex and neighbor order) |
| `is_directed()` | boolean | |
| `clear()` | null | removes everything |

```arc
bind g = graph()
g["add_edge"]("a", "b")
g["add_edge"]("b", "c", 7)
cast g["vertices"]()          ? ["a", "b", "c"]
cast g["edges"]()             ? [["a", "b", 1], ["b", "c", 7]]
cast g["neighbors"]("b")      ? ["a", "c"]
cast g["weight"]("b", "c")    ? 7
```

Degrees count a self-loop once in an undirected graph (it appears once in the
vertex's own neighbor map) and once each in `in_degree`/`out_degree` of a
digraph.

## bfs

`bfs(graph, start)` — breadth-first visit order: `start` first, then its
neighbors in insertion order, and so on. Only `start`'s component is visited.

| Parameter | Meaning |
|---|---|
| `graph` | a graph made by `graph` |
| `start` | label to start from; must exist |

```arc
bind g = graph()
g["add_edge"]("a", "b")
g["add_edge"]("a", "c")
g["add_edge"]("b", "d")
cast bfs(g, "a")              ? ["a", "b", "c", "d"]
```

## dfs

`dfs(graph, start)` — iterative depth-first visit order (a recursive twin
would hit Arcana's recursion limit on large graphs). Neighbors are pushed
**reversed** onto the explicit stack so that the **first-inserted neighbor is
the one visited first**, matching recursive preorder.

| Parameter | Meaning |
|---|---|
| `graph` | a graph made by `graph` |
| `start` | label to start from; must exist |

```arc
bind g = graph()
g["add_edge"]("a", "b")
g["add_edge"]("a", "c")
g["add_edge"]("b", "d")
cast dfs(g, "a")              ? ["a", "b", "d", "c"]
```

## path

`path(graph, start, goal)` — a shortest path found by BFS as a list of labels
from `start` to `goal` (both included), or `null` when `goal` is unreachable.
`path(g, v, v)` is `[v]`. Ignores weights — use `shortest_path` for weighted
graphs.

| Parameter | Meaning |
|---|---|
| `graph` | a graph made by `graph` |
| `start` | label to start from; must exist |
| `goal` | label to reach; must exist |

```arc
bind g = graph()
g["add_edge"]("a", "b")
g["add_edge"]("b", "c")
g["add_edge"]("a", "c")
g["add_vertex"]("island")
cast path(g, "a", "c")        ? ["a", "c"]
cast path(g, "a", "a")        ? ["a"]
cast path(g, "a", "island")   ? null
```

## dijkstra

`dijkstra(graph, source)` — a map `{label: distance}` with every vertex
**reachable** from `source` (unreachable vertices, and only those, are
omitted; `source` itself is `0`). Unweighted graphs work: the default weight
is 1. Edge weights are checked as they are relaxed: a weight that is not a
number, or a negative one, raises AssertionError.

| Parameter | Meaning |
|---|---|
| `graph` | a graph made by `graph` |
| `source` | label to start from; must exist |

```arc
bind g = graph()
g["add_edge"]("a", "b", 4)
g["add_edge"]("a", "c", 1)
g["add_edge"]("c", "b", 2)
g["add_edge"]("b", "d", 5)
cast dijkstra(g, "a")              ? {"a": 0, "b": 3, "c": 1, "d": 8}
cast shortest_path(g, "a", "d")    ? [8, ["a", "c", "b", "d"]]
```

## shortest_path

`shortest_path(graph, start, goal)` — `[distance, [labels...]]` via Dijkstra,
or `null` when `goal` is unreachable; `[0, [start]]` when `start == goal`.

| Parameter | Meaning |
|---|---|
| `graph` | a graph made by `graph` |
| `start` | label to start from; must exist |
| `goal` | label to reach; must exist |

```arc
bind g = graph()
g["add_edge"](0, 1)
g["add_edge"](1, 2)
g["add_edge"](0, 2, 5)
cast shortest_path(g, 0, 2)        ? [2, [0, 1, 2]]
cast shortest_path(g, 2, 0)        ? [2, [2, 1, 0]]
```

## connected_components

`connected_components(graph)` — the connected components as a list of lists
(each in BFS order), ordered by the position of their first vertex in
`vertices()`. Undirected graphs only.

| Parameter | Meaning |
|---|---|
| `graph` | an undirected graph made by `graph` |

```arc
bind g = graph()
g["add_edge"](0, 1)
g["add_edge"](2, 3)
g["add_vertex"](4)
cast connected_components(g)       ? [[0, 1], [2, 3], [4]]
```

## has_cycle

`has_cycle(graph)` — whether the graph contains a cycle; self-loops count.
Directed graphs use an iterative DFS with gray/black colors (meeting a gray
vertex again is a back edge); undirected graphs use a BFS with parent
tracking.

| Parameter | Meaning |
|---|---|
| `graph` | a graph made by `graph` |

```arc
bind g = graph()
g["add_edge"](0, 1)
g["add_edge"](1, 2)
cast has_cycle(g)                  ? Off
g["add_edge"](2, 0)
cast has_cycle(g)                  ? On
```

## is_bipartite

`is_bipartite(graph)` — whether an undirected graph admits a 2-coloring,
checked by BFS over **all** components. Raises AssertionError on a directed
graph.

| Parameter | Meaning |
|---|---|
| `graph` | an undirected graph made by `graph` |

## bipartite_partition

`bipartite_partition(graph)` — `[list_a, list_b]`, the two color classes with
their members in `vertices()` order, or `null` when the graph is not
bipartite. Undirected graphs only; raises AssertionError otherwise.

| Parameter | Meaning |
|---|---|
| `graph` | an undirected graph made by `graph` |

```arc
bind g = graph()
g["add_edge"](0, 1)
g["add_edge"](1, 2)
g["add_edge"](2, 3)
cast is_bipartite(g)               ? On
cast bipartite_partition(g)        ? [[0, 2], [1, 3]]
```

## topological_sort

`topological_sort(graph)` — a topological order of a **directed** graph via
Kahn's algorithm, or `null` when the graph has a cycle. The order is
deterministic: among the ready (zero-indegree) vertices, the one earliest in
`vertices()` insertion order leaves first. Every vertex appears exactly once.

| Parameter | Meaning |
|---|---|
| `graph` | a directed graph made by `graph(On)` |

```arc
bind g = graph(On)
g["add_edge"](5, 2)
g["add_edge"](5, 0)
g["add_edge"](4, 0)
g["add_edge"](4, 1)
g["add_edge"](2, 3)
g["add_edge"](3, 1)
cast topological_sort(g)           ? [5, 2, 4, 0, 3, 1]
```

## scc

`scc(graph)` — the strongly connected components of a **directed** graph,
computed with an iterative Kosaraju (finish order on the graph, then a
component DFS on `reverse()` — no recursion anywhere). The output is
canonical: each component's members are sorted by `vertices()` insertion
order, and the component list is ordered by each component's first member's
position in `vertices()`.

| Parameter | Meaning |
|---|---|
| `graph` | a directed graph made by `graph(On)` |

```arc
bind g = graph(On)
g["add_edge"](0, 1)
g["add_edge"](1, 2)
g["add_edge"](2, 0)
g["add_edge"](2, 3)
cast scc(g)                        ? [[0, 1, 2], [3]]
```

## Complexity summary

With `V` vertices and `E` edges:

| Operation | Cost |
|---|---|
| `add_vertex`, `add_edge`, `has_vertex`, `has_edge`, `weight`, `size`, `out_degree`, `is_directed` | O(1) |
| `remove_edge` | O(deg u) |
| `remove_vertex`, `clear` | O(V + E) |
| `vertices`, `neighbors`, `in_degree` (directed), `degree` (directed) | O(V) |
| `edge_count`, `edges`, `adjacency_list`, `reverse` | O(V + E) |
| `adjacency_matrix` | O(V²) |
| `bfs`, `dfs`, `path`, `connected_components`, `has_cycle`, `is_bipartite`, `bipartite_partition` | O(V + E) |
| `dijkstra`, `shortest_path` | O(V² + E) (linear min-scan, no heap) |
| `topological_sort` | O(V² + E) |
| `scc` | O(V + E) plus sorting of the output |

## Error behaviour

Every misuse raises catchable `E2101` (`AssertionError`) with a `Candy: ...`
message:

* a vertex label that is neither text nor integer (`"Candy: vertex labels
  must be text or integer"`) — checked by every method that takes a label;
* `neighbors`, `weight`, `in_degree`, `out_degree`, `degree` and
  `remove_edge` on an unknown label (`"Candy: unknown vertex: ..."`);
* `bfs`/`dfs`/`shortest_path` with an unknown start (`"Candy: unknown start
  vertex: ..."`), `path`/`shortest_path` with an unknown goal (`"Candy:
  unknown goal vertex: ..."`), `dijkstra` with an unknown source (`"Candy:
  unknown source vertex: ..."`);
* `dijkstra`/`shortest_path` meeting an edge weight that is not a number
  (`"Candy: edge weights must be numbers"`) or a negative one (`"Candy:
  negative weight"`) the first time that weight is relaxed — weights on
  edges the search never relaxes are never examined;
* `connected_components`, `is_bipartite` and `bipartite_partition` on a
  directed graph (`"... needs an undirected graph"`);
* `topological_sort` and `scc` on an undirected graph (`"... needs a
  directed graph"`);
* `graph` with something other than `On`/`Off` (`"Candy: directed must
  be On or Off"`).

`null` results are *not* errors: `weight` returns null for a missing edge,
`path`/`shortest_path` return null for unreachable goals,
`topological_sort` returns null for a cyclic graph and
`bipartite_partition` returns null for a non-bipartite one.

Arity and type mistakes in *your* call (`bfs(g)` with no start,
`g["size"](1)`) are ordinary Arcana errors (`E3004`/`E3003`) and are not
catchable — they are programmer errors, exactly like the core language
treats them.
