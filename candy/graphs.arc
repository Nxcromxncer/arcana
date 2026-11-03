? Candy 04: graphs — labelled graphs and classic graph algorithms for Arcana.
?
? Part of the Candy library (Tier I, module 04 of 04).
? Import from your grimoire with:
?
?     import candy.graphs                ? when the candy/ folder sits next to
?                                        ? your file, or is on ARCANA_PATH
?     import graphs                      ? when this file sits next to yours
?
? The one structure here is graph(directed = Off), a *closure object*
? whose adjacency list is an Arcana map label -> (map label -> weight).
? Vertex labels may be texts or integers; insertion order is preserved
? everywhere (vertices(), neighbors(), edges()). The kind tag is
? "candy/graph" for undirected graphs and "candy/digraph" for directed ones.
?
?     bind g = graph()
?     g["add_edge"]("a", "b")
?     cast bfs(g, "a")                   ? ["a", "b"]
?
? Free spells: bfs, dfs, path, dijkstra, shortest_path, connected_components,
? has_cycle, is_bipartite, bipartite_partition, topological_sort, scc.
? Every traversal is ITERATIVE (explicit stack/queue lists): Arcana caps
? recursion near 1000 frames, so walks here survive 2000-vertex chains.
? Misuse raises a catchable AssertionError through assert(), with a message
? that starts with "Candy: ".

? ---------------------------------------------------------------------------
? Small helpers shared by the spells below
? ---------------------------------------------------------------------------

spell _map_without(m, k) {
    ? a new map without key k; surviving keys keep their insertion order
    bind out = {}
    for key in m {
        if key != k {
            out[key] = m[key]
        }
    }
    return out
}

spell _unreverse(items) {
    ? a new list with the items in opposite order
    bind out = []
    bind i = length(items) - 1
    while i >= 0 {
        append(out, items[i])
        i = i - 1
    }
    return out
}

spell _pos_map(labels) {
    ? label -> position in the given order (usually the vertices() order)
    bind pos = {}
    bind i = 0
    for v in labels {
        pos[v] = i
        i = i + 1
    }
    return pos
}

spell _sort_by_pos(items, pos) {
    ? insertion sort of labels by pos[label] (sorted() cannot order labels
    ? against a custom key); keeps output deterministic
    bind arr = copy(items)
    for j in range(1, length(arr)) {
        bind cur = arr[j]
        bind k = j - 1
        while k >= 0 and pos[arr[k]] > pos[cur] {
            arr[k + 1] = arr[k]
            k = k - 1
        }
        arr[k + 1] = cur
    }
    return arr
}

spell _sort_components(comps, pos) {
    ? insertion sort of component lists by the position of their first member
    bind arr = copy(comps)
    for j in range(1, length(arr)) {
        bind cur = arr[j]
        bind k = j - 1
        while k >= 0 and pos[arr[k][0]] > pos[cur[0]] {
            arr[k + 1] = arr[k]
            k = k - 1
        }
        arr[k + 1] = cur
    }
    return arr
}

? ---------------------------------------------------------------------------
? graph — the graph closure object
? ---------------------------------------------------------------------------

spell _graph_from(adj, directed) {
    ? builds the method map around a ready adjacency map; graph() and
    ? reverse() both come through here, so vertex order is whatever the map
    ? already carries
    bind kind = "candy/graph"
    if directed {
        kind = "candy/digraph"
    }
    spell check_label(v) {
        bind t = type_of(v)
        assert(t == "text" or t == "integer", "Candy: vertex labels must be text or integer")
        return null
    }
    spell add_vertex(v) {
        check_label(v)
        if v in adj {
            return Off
        }
        adj[v] = {}
        return On
    }
    spell add_edge(u, v, w = 1) {
        check_label(u)
        check_label(v)
        if not (u in adj) {
            adj[u] = {}
        }
        if not (v in adj) {
            adj[v] = {}
        }
        adj[u][v] = w
        if not directed {
            adj[v][u] = w
        }
        return null
    }
    spell remove_edge(u, v) {
        check_label(u)
        check_label(v)
        assert(u in adj, "Candy: unknown vertex: " + to_text(u))
        assert(v in adj, "Candy: unknown vertex: " + to_text(v))
        if not (v in adj[u]) {
            return Off
        }
        adj[u] = _map_without(adj[u], v)
        if not directed {
            adj[v] = _map_without(adj[v], u)
        }
        return On
    }
    spell remove_vertex(v) {
        check_label(v)
        if not (v in adj) {
            return Off
        }
        bind fresh = {}
        for u in adj {
            if u != v {
                fresh[u] = _map_without(adj[u], v)
            }
        }
        adj = fresh
        return On
    }
    spell has_vertex(v) {
        check_label(v)
        return v in adj
    }
    spell has_edge(u, v) {
        check_label(u)
        check_label(v)
        if not (u in adj) {
            return Off
        }
        return v in adj[u]
    }
    spell weight(u, v) {
        check_label(u)
        check_label(v)
        assert(u in adj, "Candy: unknown vertex: " + to_text(u))
        assert(v in adj, "Candy: unknown vertex: " + to_text(v))
        if v in adj[u] {
            return adj[u][v]
        }
        return null
    }
    spell vertices() {
        return keys(adj)
    }
    spell neighbors(v) {
        check_label(v)
        assert(v in adj, "Candy: unknown vertex: " + to_text(v))
        return keys(adj[v])
    }
    spell in_degree(v) {
        check_label(v)
        assert(v in adj, "Candy: unknown vertex: " + to_text(v))
        if not directed {
            return length(adj[v])
        }
        bind n = 0
        for u in adj {
            if v in adj[u] {
                n = n + 1
            }
        }
        return n
    }
    spell out_degree(v) {
        check_label(v)
        assert(v in adj, "Candy: unknown vertex: " + to_text(v))
        return length(adj[v])
    }
    spell degree(v) {
        check_label(v)
        assert(v in adj, "Candy: unknown vertex: " + to_text(v))
        if not directed {
            return length(adj[v])
        }
        return in_degree(v) + out_degree(v)
    }
    spell size() {
        return length(adj)
    }
    spell edge_count() {
        bind total = 0
        bind loops = 0
        for u in adj {
            total = total + length(adj[u])
            if u in adj[u] {
                loops = loops + 1
            }
        }
        if directed {
            return total
        }
        return (total - loops) // 2 + loops
    }
    spell edges() {
        ? undirected edges are reported once, at the endpoint that comes
        ? first in vertex order (self-loops at their own vertex)
        bind out = []
        bind pos = _pos_map(keys(adj))
        for u in adj {
            for v in adj[u] {
                if directed {
                    append(out, [u, v, adj[u][v]])
                } else {
                    if u == v or pos[u] < pos[v] {
                        append(out, [u, v, adj[u][v]])
                    }
                }
            }
        }
        return out
    }
    spell adjacency_list() {
        bind out = {}
        for u in adj {
            out[u] = keys(adj[u])
        }
        return out
    }
    spell adjacency_matrix() {
        bind labels = keys(adj)
        bind matrix = []
        for u in labels {
            bind row = []
            for v in labels {
                if v in adj[u] {
                    append(row, adj[u][v])
                } else {
                    append(row, null)
                }
            }
            append(matrix, row)
        }
        return {"labels": labels, "matrix": matrix}
    }
    spell reverse() {
        ? directed: a new graph with every arc flipped; undirected: an exact
        ? structural copy (same vertex and neighbor order)
        bind fresh = {}
        for u in adj {
            fresh[u] = {}
        }
        for u in adj {
            for v in adj[u] {
                if directed {
                    fresh[v][u] = adj[u][v]
                } else {
                    fresh[u][v] = adj[u][v]
                }
            }
        }
        return _graph_from(fresh, directed)
    }
    spell is_directed() {
        return directed
    }
    spell clear() {
        adj = {}
        return null
    }
    return {"kind": kind, "add_vertex": add_vertex, "add_edge": add_edge,
            "remove_edge": remove_edge, "remove_vertex": remove_vertex,
            "has_vertex": has_vertex, "has_edge": has_edge, "weight": weight,
            "vertices": vertices, "neighbors": neighbors,
            "in_degree": in_degree, "out_degree": out_degree, "degree": degree,
            "size": size, "edge_count": edge_count, "edges": edges,
            "adjacency_list": adjacency_list, "adjacency_matrix": adjacency_matrix,
            "reverse": reverse, "is_directed": is_directed, "clear": clear}
}

spell graph(directed = Off) {
    assert(directed == On or directed == Off, "Candy: directed must be On or Off")
    return _graph_from({}, directed)
}

? ---------------------------------------------------------------------------
? Traversals: bfs, dfs, path
? ---------------------------------------------------------------------------

spell bfs(graph, start) {
    ? breadth-first visit order; neighbors expand in insertion order and
    ? only start's component is visited
    assert(graph["has_vertex"](start), "Candy: unknown start vertex: " + to_text(start))
    bind seen = {}
    seen[start] = On
    bind order = []
    bind queue = [start]
    bind head = 0
    while head < length(queue) {
        bind u = queue[head]
        head = head + 1
        append(order, u)
        for v in graph["neighbors"](u) {
            if not (v in seen) {
                seen[v] = On
                append(queue, v)
            }
        }
    }
    return order
}

spell dfs(graph, start) {
    ? iterative depth-first search (a recursive twin would die on big graphs).
    ? Neighbors are pushed REVERSED onto the stack so that the FIRST-inserted
    ? neighbor is the one visited first, matching recursive preorder.
    assert(graph["has_vertex"](start), "Candy: unknown start vertex: " + to_text(start))
    bind seen = {}
    bind order = []
    bind stack = [start]
    while length(stack) > 0 {
        bind u = pop(stack)
        if u in seen {
            continue
        }
        seen[u] = On
        append(order, u)
        bind ns = graph["neighbors"](u)
        bind i = length(ns) - 1
        while i >= 0 {
            if not (ns[i] in seen) {
                append(stack, ns[i])
            }
            i = i - 1
        }
    }
    return order
}

spell path(graph, start, goal) {
    ? BFS shortest path as a list of labels; null when unreachable;
    ? [start] when start == goal
    assert(graph["has_vertex"](start), "Candy: unknown start vertex: " + to_text(start))
    assert(graph["has_vertex"](goal), "Candy: unknown goal vertex: " + to_text(goal))
    if start == goal {
        return [start]
    }
    bind parent = {}
    parent[start] = null
    bind queue = [start]
    bind head = 0
    bind found = Off
    while head < length(queue) and not found {
        bind u = queue[head]
        head = head + 1
        for v in graph["neighbors"](u) {
            if not (v in parent) {
                parent[v] = u
                append(queue, v)
                if v == goal {
                    found = On
                }
            }
        }
    }
    if not found {
        return null
    }
    bind rev = [goal]
    bind cur = goal
    while cur != start {
        cur = parent[cur]
        append(rev, cur)
    }
    return _unreverse(rev)
}

? ---------------------------------------------------------------------------
? Shortest paths: dijkstra, shortest_path
? ---------------------------------------------------------------------------

spell _dijkstra(graph, source) {
    ? linear-scan Dijkstra (no heap): returns {"dist": ..., "parent": ...}
    ? for every REACHABLE vertex; unreachable ones never enter the map.
    ? Weights are checked as they are relaxed.
    bind dist = {}
    bind parent = {}
    dist[source] = 0
    bind visited = {}
    bind vs = graph["vertices"]()
    bind going = On
    while going {
        bind best = null
        bind best_d = 0
        for v in vs {
            if not (v in visited) and v in dist {
                if best == null or dist[v] < best_d {
                    best = v
                    best_d = dist[v]
                }
            }
        }
        if best == null {
            going = Off
        } else {
            visited[best] = On
            for v in graph["neighbors"](best) {
                bind w = graph["weight"](best, v)
                bind tw = type_of(w)
                assert(tw == "integer" or tw == "decimal", "Candy: edge weights must be numbers")
                assert(w >= 0, "Candy: negative weight")
                bind nd = best_d + w
                if not (v in dist) or nd < dist[v] {
                    dist[v] = nd
                    parent[v] = best
                }
            }
        }
    }
    return {"dist": dist, "parent": parent}
}

spell dijkstra(graph, source) {
    assert(graph["has_vertex"](source), "Candy: unknown source vertex: " + to_text(source))
    bind res = _dijkstra(graph, source)
    return res["dist"]
}

spell shortest_path(graph, start, goal) {
    ? [distance, [labels...]] via Dijkstra (unweighted graphs work: the
    ? default weight is 1); null when unreachable; [0, [start]] when equal
    assert(graph["has_vertex"](start), "Candy: unknown start vertex: " + to_text(start))
    assert(graph["has_vertex"](goal), "Candy: unknown goal vertex: " + to_text(goal))
    if start == goal {
        return [0, [start]]
    }
    bind res = _dijkstra(graph, start)
    if not (goal in res["dist"]) {
        return null
    }
    bind rev = [goal]
    bind cur = goal
    while cur != start {
        cur = res["parent"][cur]
        append(rev, cur)
    }
    return [res["dist"][goal], _unreverse(rev)]
}

? ---------------------------------------------------------------------------
? Components, cycles, bipartiteness
? ---------------------------------------------------------------------------

spell connected_components(graph) {
    ? one BFS per unvisited vertex; components come out in the order their
    ? first vertex appears in vertices()
    assert(not graph["is_directed"](), "Candy: connected_components needs an undirected graph")
    bind seen = {}
    bind comps = []
    for v in graph["vertices"]() {
        if not (v in seen) {
            bind comp = bfs(graph, v)
            for u in comp {
                seen[u] = On
            }
            append(comps, comp)
        }
    }
    return comps
}

spell _directed_has_cycle(graph) {
    ? iterative DFS with gray/black colors; meeting a gray vertex again is a
    ? back edge (self-loops included: the vertex is gray while scanning it)
    bind color = {}
    for s in graph["vertices"]() {
        if s in color {
            continue
        }
        color[s] = "gray"
        bind stack = [[s, 0]]
        while length(stack) > 0 {
            bind top = stack[length(stack) - 1]
            bind u = top[0]
            bind ns = graph["neighbors"](u)
            bind i = top[1]
            if i < length(ns) {
                top[1] = i + 1
                bind v = ns[i]
                if v in color {
                    if color[v] == "gray" {
                        return On
                    }
                } else {
                    color[v] = "gray"
                    append(stack, [v, 0])
                }
            } else {
                color[u] = "black"
                pop(stack)
            }
        }
    }
    return Off
}

spell _undirected_has_cycle(graph) {
    ? BFS with parent tracking; meeting an already-seen neighbor that is not
    ? our parent closes a cycle (self-loops included)
    bind parent = {}
    for s in graph["vertices"]() {
        if s in parent {
            continue
        }
        parent[s] = null
        bind queue = [s]
        bind head = 0
        bind found = Off
        while head < length(queue) and not found {
            bind u = queue[head]
            head = head + 1
            for v in graph["neighbors"](u) {
                if not (v in parent) {
                    parent[v] = u
                    append(queue, v)
                } else {
                    if v != parent[u] {
                        found = On
                        break
                    }
                }
            }
        }
        if found {
            return On
        }
    }
    return Off
}

spell has_cycle(graph) {
    if graph["is_directed"]() {
        return _directed_has_cycle(graph)
    }
    return _undirected_has_cycle(graph)
}

spell _bipartite_colors(graph) {
    ? 2-colors every component by BFS; null as soon as a conflict appears
    bind color = {}
    for s in graph["vertices"]() {
        if not (s in color) {
            color[s] = 0
            bind queue = [s]
            bind head = 0
            while head < length(queue) {
                bind u = queue[head]
                head = head + 1
                for v in graph["neighbors"](u) {
                    if not (v in color) {
                        if color[u] == 0 {
                            color[v] = 1
                        } else {
                            color[v] = 0
                        }
                        append(queue, v)
                    } else {
                        if color[v] == color[u] {
                            return null
                        }
                    }
                }
            }
        }
    }
    return color
}

spell is_bipartite(graph) {
    assert(not graph["is_directed"](), "Candy: is_bipartite needs an undirected graph")
    return _bipartite_colors(graph) != null
}

spell bipartite_partition(graph) {
    ? [list_a, list_b] with the sides in vertices() order, or null when the
    ? graph is not bipartite
    assert(not graph["is_directed"](), "Candy: bipartite_partition needs an undirected graph")
    bind color = _bipartite_colors(graph)
    if color == null {
        return null
    }
    bind side_a = []
    bind side_b = []
    for v in graph["vertices"]() {
        if color[v] == 0 {
            append(side_a, v)
        } else {
            append(side_b, v)
        }
    }
    return [side_a, side_b]
}

? ---------------------------------------------------------------------------
? Directed acyclic work: topological_sort, scc
? ---------------------------------------------------------------------------

spell topological_sort(graph) {
    ? Kahn's algorithm; among the ready (zero-indegree) vertices the one
    ? earliest in vertices() insertion order leaves first; null on a cycle
    assert(graph["is_directed"](), "Candy: topological_sort needs a directed graph")
    bind vs = graph["vertices"]()
    bind indeg = {}
    for v in vs {
        indeg[v] = graph["in_degree"](v)
    }
    bind placed = {}
    bind out = []
    while length(out) < length(vs) {
        bind picked = null
        for v in vs {
            if indeg[v] == 0 and not (v in placed) {
                picked = v
                break
            }
        }
        if picked == null {
            return null
        }
        placed[picked] = On
        append(out, picked)
        for w in graph["neighbors"](picked) {
            indeg[w] = indeg[w] - 1
        }
    }
    return out
}

spell _dfs_finish_order(graph) {
    ? iterative DFS post-order: labels in the order their exploration finishes
    bind seen = {}
    bind finish = []
    for s in graph["vertices"]() {
        if s in seen {
            continue
        }
        seen[s] = On
        bind stack = [[s, 0]]
        while length(stack) > 0 {
            bind top = stack[length(stack) - 1]
            bind u = top[0]
            bind ns = graph["neighbors"](u)
            bind i = top[1]
            if i < length(ns) {
                top[1] = i + 1
                bind v = ns[i]
                if not (v in seen) {
                    seen[v] = On
                    append(stack, [v, 0])
                }
            } else {
                append(finish, u)
                pop(stack)
            }
        }
    }
    return finish
}

spell scc(graph) {
    ? iterative Kosaraju: finish order on the graph, then component DFS on
    ? the reverse. Output is canonical: members sorted by vertices() order,
    ? the component list ordered by each component's first member.
    assert(graph["is_directed"](), "Candy: scc needs a directed graph")
    bind fin = _dfs_finish_order(graph)
    bind rev = graph["reverse"]()
    bind assigned = {}
    bind comps = []
    bind k = length(fin) - 1
    while k >= 0 {
        bind s = fin[k]
        k = k - 1
        if not (s in assigned) {
            assigned[s] = On
            bind comp = []
            bind stack = [s]
            while length(stack) > 0 {
                bind u = pop(stack)
                append(comp, u)
                for v in rev["neighbors"](u) {
                    if not (v in assigned) {
                        assigned[v] = On
                        append(stack, v)
                    }
                }
            }
            append(comps, comp)
        }
    }
    bind pos = _pos_map(graph["vertices"]())
    bind canon = []
    for c in comps {
        append(canon, _sort_by_pos(c, pos))
    }
    return _sort_components(canon, pos)
}
