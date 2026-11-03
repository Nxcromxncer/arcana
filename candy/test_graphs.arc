? Candy test: graphs — run with  arcana run candy/test_graphs.arc
import graphs

spell expect_error(thunk, message_part) {
    bind failed = Off
    try {
        thunk()
    } catch e {
        failed = On
        assert("Candy" in e["message"] or message_part in e["message"],
               "wrong error message: " + e["message"])
    }
    assert(failed, "expected an error containing: " + message_part)
    return null
}

spell canon_sccs(graph, comps) {
    ? normalise SCC output to a set of sets: members and component lists are
    ? both sorted by vertices() insertion order before comparing
    bind pos = {}
    bind i = 0
    for v in graph["vertices"]() {
        pos[v] = i
        i = i + 1
    }
    bind out = []
    for c in comps {
        bind arr = copy(c)
        for j in range(1, length(arr)) {
            bind cur = arr[j]
            bind k = j - 1
            while k >= 0 and pos[arr[k]] > pos[cur] {
                arr[k + 1] = arr[k]
                k = k - 1
            }
            arr[k + 1] = cur
        }
        append(out, arr)
    }
    for j in range(1, length(out)) {
        bind cur2 = out[j]
        bind k2 = j - 1
        while k2 >= 0 and pos[out[k2][0]] > pos[cur2[0]] {
            out[k2 + 1] = out[k2]
            k2 = k2 - 1
        }
        out[k2 + 1] = cur2
    }
    return out
}

spell main() {
    ? ----- graph: undirected CRUD -----
    bind g = graphs.graph()
    assert(g["kind"] == "candy/graph", "graph kind")
    assert(g["is_directed"]() == Off, "undirected flag")
    assert(g["size"]() == 0 and g["edge_count"]() == 0, "empty graph")
    assert(g["vertices"]() == [], "no vertices yet")
    assert(g["add_vertex"]("a") == On, "add_vertex new")
    assert(g["add_vertex"]("a") == Off, "add_vertex duplicate")
    assert(g["add_vertex"](1) == On, "integer label allowed")
    assert(g["size"]() == 2, "size counts labels")
    g["add_edge"]("a", "b")
    assert(g["has_vertex"]("b"), "add_edge adds missing endpoints")
    assert(g["size"]() == 3, "size after auto-add")
    assert(g["has_edge"]("a", "b") and g["has_edge"]("b", "a"), "undirected symmetry")
    assert(g["weight"]("a", "b") == 1, "default weight is 1")
    assert(g["weight"]("a", 1) == null, "missing edge weight is null")
    g["add_edge"]("a", "b", 5)
    assert(g["weight"]("a", "b") == 5 and g["weight"]("b", "a") == 5,
           "re-adding replaces the weight on both sides")
    g["add_edge"]("c", "c", 2)
    assert(g["has_edge"]("c", "c"), "self-loop stored")
    assert(g["weight"]("c", "c") == 2, "self-loop weight")
    assert(g["edge_count"]() == 2, "self-loop counts once")
    g["add_edge"]("b", 1, 7)
    assert(g["degree"]("b") == 2, "degree b")
    assert(g["in_degree"]("b") == 2 and g["out_degree"]("b") == 2, "undirected in == out")
    assert(g["degree"]("c") == 1, "self-loop contributes 1 to undirected degree")

    ? ----- adjacency views -----
    assert(g["vertices"]() == ["a", 1, "b", "c"], "vertex insertion order")
    assert(g["adjacency_list"]() == {"a": ["b"], 1: ["b"], "b": ["a", 1], "c": ["c"]},
           "adjacency list")
    assert(g["edges"]() == [["a", "b", 5], [1, "b", 7], ["c", "c", 2]],
           "undirected edges deduplicated, first-seen order")
    assert(g["adjacency_matrix"]() ==
           {"labels": ["a", 1, "b", "c"],
            "matrix": [[null, null, 5, null],
                       [null, null, 7, null],
                       [5, 7, null, null],
                       [null, null, null, 2]]}, "adjacency matrix")

    ? ----- unknown-vertex AssertionErrors -----
    spell neighbors_unknown() {
        g["neighbors"]("zz")
    }
    expect_error(neighbors_unknown, "unknown vertex")
    spell weight_unknown() {
        g["weight"]("a", "zz")
    }
    expect_error(weight_unknown, "unknown vertex")
    spell remove_edge_unknown() {
        g["remove_edge"]("a", "zz")
    }
    expect_error(remove_edge_unknown, "unknown vertex")
    spell bad_label() {
        g["add_vertex"]([1, 2])
    }
    expect_error(bad_label, "labels must be text or integer")
    assert(g["remove_vertex"]("zz") == Off, "remove_vertex unknown is Off, no raise")

    ? ----- removals and clear -----
    assert(g["remove_edge"]("b", "a") == On, "remove_edge via mirror endpoint")
    assert(not g["has_edge"]("a", "b") and not g["has_edge"]("b", "a"),
           "undirected remove drops both directions")
    assert(g["remove_edge"]("a", "b") == Off, "remove_edge absent is Off")
    assert(g["has_vertex"]("a") and g["has_vertex"]("b"), "vertices survive edge removal")
    assert(g["edge_count"]() == 2, "edge count after removal")
    assert(g["remove_vertex"](1) == On, "remove_vertex existing")
    assert(not g["has_vertex"](1) and g["size"]() == 3, "vertex gone, incident edge dropped")
    assert(g["edges"]() == [["c", "c", 2]], "only the self-loop remains")
    assert(g["remove_vertex"](1) == Off, "remove_vertex again is Off")
    g["remove_vertex"]("c")
    assert(g["size"]() == 2, "self-loop vertex removed")
    g["clear"]()
    assert(g["size"]() == 0 and g["edge_count"]() == 0 and g["vertices"]() == [], "clear")

    ? ----- directed CRUD + reverse -----
    bind dg = graphs.graph(On)
    assert(dg["kind"] == "candy/digraph", "digraph kind")
    assert(dg["is_directed"]() == On, "directed flag")
    dg["add_edge"](1, 2, 4)
    dg["add_edge"](2, 3, 5)
    dg["add_edge"](3, 1, 6)
    assert(dg["edge_count"]() == 3, "directed edge count counts arcs")
    assert(dg["has_edge"](2, 1) == Off, "no mirror arc in a digraph")
    assert(dg["edges"]() == [[1, 2, 4], [2, 3, 5], [3, 1, 6]], "directed edges are arcs")
    assert(dg["adjacency_list"]() == {1: [2], 2: [3], 3: [1]}, "directed adjacency list")
    assert(dg["in_degree"](1) == 1 and dg["out_degree"](1) == 1 and dg["degree"](1) == 2,
           "directed degree is in + out")
    assert(dg["remove_edge"](1, 2) == On, "directed remove_edge")
    assert(not dg["has_edge"](1, 2) and dg["has_edge"](2, 1) == Off,
           "directed remove drops only u->v")
    dg["add_edge"](1, 2, 4)
    bind rdg = dg["reverse"]()
    assert(rdg["kind"] == "candy/digraph", "reverse keeps directed")
    assert(rdg["size"]() == 3, "reverse has the same vertices")
    assert(rdg["edges"]() == [[1, 3, 6], [2, 1, 4], [3, 2, 5]], "arcs flipped")
    assert(rdg["weight"](2, 1) == 4 and rdg["weight"](1, 2) == null, "reverse weights")
    assert(dg["edges"]() == [[1, 2, 4], [2, 3, 5], [3, 1, 6]],
           "reverse is a new graph, original untouched")

    ? ----- reverse on undirected is an independent copy -----
    bind g8 = graphs.graph()
    g8["add_edge"](0, 2)
    g8["add_edge"](0, 1)
    g8["add_edge"](1, 3)
    g8["add_edge"](1, 4)
    g8["add_edge"](2, 5)
    g8["add_edge"](2, 6)
    g8["add_edge"](6, 7)
    g8["add_edge"](3, 7)
    g8["add_vertex"](8)
    g8["add_edge"](9, 10)
    bind gcopy = g8["reverse"]()
    assert(gcopy["kind"] == "candy/graph", "copy kind")
    assert(gcopy["vertices"]() == g8["vertices"](), "copy keeps vertex order")
    assert(gcopy["neighbors"](7) == g8["neighbors"](7), "copy keeps neighbor order")
    assert(gcopy["edges"]() == g8["edges"](), "copy edges")
    assert(gcopy["adjacency_matrix"]() == g8["adjacency_matrix"](), "copy matrix")
    gcopy["add_vertex"]("extra")
    gcopy["add_edge"](0, 8)
    assert(not g8["has_vertex"]("extra") and not g8["has_edge"](0, 8),
           "copy is fully independent")

    ? ----- bfs / dfs on the known 8-vertex graph -----
    assert(graphs.bfs(g8, 0) == [0, 2, 1, 5, 6, 3, 4, 7], "bfs visit order")
    assert(graphs.dfs(g8, 0) == [0, 2, 5, 6, 7, 3, 1, 4],
           "dfs visit order, first-inserted neighbor first")
    assert(graphs.bfs(g8, 9) == [9, 10], "bfs stays in start's component")
    assert(graphs.dfs(g8, 8) == [8], "isolated dfs")

    ? ----- path -----
    assert(graphs.path(g8, 0, 4) == [0, 1, 4], "shortest path")
    assert(graphs.path(g8, 0, 7) == [0, 2, 6, 7], "shortest path via 6")
    assert(graphs.path(g8, 5, 5) == [5], "start == goal is [start]")
    assert(graphs.path(g8, 0, 8) == null, "unreachable is null")

    ? ----- 2000-vertex chain: traversals must be iterative -----
    bind chain = graphs.graph()
    for i in range(0, 2000) {
        chain["add_vertex"](i)
    }
    for i in range(0, 1999) {
        chain["add_edge"](i, i + 1)
    }
    assert(chain["size"]() == 2000 and chain["edge_count"]() == 1999, "chain built")
    bind cbfs = graphs.bfs(chain, 0)
    assert(length(cbfs) == 2000 and cbfs[0] == 0 and cbfs[1999] == 1999, "chain bfs complete")
    bind cdfs = graphs.dfs(chain, 0)
    assert(length(cdfs) == 2000 and cdfs[0] == 0 and cdfs[1999] == 1999, "chain dfs complete")
    bind cpath = graphs.path(chain, 0, 1999)
    assert(length(cpath) == 2000, "chain path length 2000")
    assert(cpath[0] == 0 and cpath[1999] == 1999, "chain path ends")

    ? ----- dijkstra / shortest_path -----
    bind wg = graphs.graph()
    wg["add_edge"](0, 1, 4)
    wg["add_edge"](0, 2, 1)
    wg["add_edge"](2, 1, 2)
    wg["add_edge"](1, 3, 5)
    wg["add_edge"](2, 3, 8)
    wg["add_edge"](3, 4, 3)
    wg["add_vertex"](5)
    bind dist = graphs.dijkstra(wg, 0)
    assert(dist == {0: 0, 1: 3, 2: 1, 3: 8, 4: 11},
           "dijkstra distances, vertex 5 unreachable and omitted")
    assert(graphs.shortest_path(wg, 0, 4) == [11, [0, 2, 1, 3, 4]], "shortest_path dist + path")
    assert(graphs.shortest_path(wg, 0, 5) == null, "shortest_path unreachable is null")
    assert(graphs.shortest_path(wg, 0, 0) == [0, [0]], "shortest_path start == goal")
    bind ug = graphs.graph()
    ug["add_edge"](0, 1)
    ug["add_edge"](1, 2)
    ug["add_edge"](0, 2, 5)
    assert(graphs.shortest_path(ug, 0, 2) == [2, [0, 1, 2]],
           "unweighted graphs work: default weight 1")
    spell negative() {
        bind ng = graphs.graph()
        ng["add_edge"](0, 1, 2)
        ng["add_edge"](1, 2, -5)
        graphs.dijkstra(ng, 0)
    }
    expect_error(negative, "negative weight")
    spell negative_sp() {
        bind ng2 = graphs.graph()
        ng2["add_edge"](0, 1, 2)
        ng2["add_edge"](1, 2, -5)
        graphs.shortest_path(ng2, 0, 2)
    }
    expect_error(negative_sp, "negative weight")

    ? ----- connected_components -----
    bind comps = graphs.connected_components(g8)
    assert(comps == [[0, 2, 1, 5, 6, 3, 4, 7], [8], [9, 10]], "three known components")
    spell comps_directed() {
        graphs.connected_components(dg)
    }
    expect_error(comps_directed, "connected_components needs an undirected graph")

    ? ----- has_cycle -----
    bind tree = graphs.graph()
    tree["add_edge"](0, 1)
    tree["add_edge"](1, 2)
    assert(graphs.has_cycle(tree) == Off, "undirected tree has no cycle")
    bind tri = graphs.graph()
    tri["add_edge"](0, 1)
    tri["add_edge"](1, 2)
    tri["add_edge"](2, 0)
    assert(graphs.has_cycle(tri) == On, "undirected triangle has a cycle")
    bind loopg = graphs.graph()
    loopg["add_edge"](0, 1)
    loopg["add_edge"](1, 1)
    assert(graphs.has_cycle(loopg) == On, "undirected self-loop is a cycle")
    bind dag = graphs.graph(On)
    dag["add_edge"](5, 2)
    dag["add_edge"](5, 0)
    dag["add_edge"](4, 0)
    dag["add_edge"](4, 1)
    dag["add_edge"](2, 3)
    dag["add_edge"](3, 1)
    assert(graphs.has_cycle(dag) == Off, "directed DAG has no cycle")
    bind dcyc = graphs.graph(On)
    dcyc["add_edge"](1, 2)
    dcyc["add_edge"](2, 3)
    dcyc["add_edge"](3, 1)
    assert(graphs.has_cycle(dcyc) == On, "directed 3-cycle")
    bind dself = graphs.graph(On)
    dself["add_edge"](1, 2)
    dself["add_edge"](2, 2)
    assert(graphs.has_cycle(dself) == On, "directed self-loop is a cycle")

    ? ----- is_bipartite / bipartite_partition -----
    bind c6 = graphs.graph()
    c6["add_edge"](0, 1)
    c6["add_edge"](1, 2)
    c6["add_edge"](2, 3)
    c6["add_edge"](3, 4)
    c6["add_edge"](4, 5)
    c6["add_edge"](5, 0)
    assert(graphs.is_bipartite(c6) == On, "even cycle is bipartite")
    bind parts = graphs.bipartite_partition(c6)
    assert(length(parts) == 2, "partition has two sides")
    assert(length(parts[0]) == 3 and length(parts[1]) == 3, "partition sizes")
    assert(parts[0] == [0, 2, 4] and parts[1] == [1, 3, 5], "partition sides")
    bind c5 = graphs.graph()
    c5["add_edge"](0, 1)
    c5["add_edge"](1, 2)
    c5["add_edge"](2, 3)
    c5["add_edge"](3, 4)
    c5["add_edge"](4, 0)
    assert(graphs.is_bipartite(c5) == Off, "odd cycle is not bipartite")
    assert(graphs.bipartite_partition(c5) == null, "odd cycle partition is null")
    bind bg = graphs.graph()
    bg["add_edge"](0, 1)
    bg["add_edge"](2, 3)
    bg["add_edge"](4, 5)
    assert(graphs.is_bipartite(bg) == On, "disconnected bipartite graph")
    assert(graphs.bipartite_partition(bg) == [[0, 2, 4], [1, 3, 5]],
           "disconnected partition covers all components")
    spell bip_directed() {
        graphs.is_bipartite(dg)
    }
    expect_error(bip_directed, "is_bipartite needs an undirected graph")
    spell part_directed() {
        graphs.bipartite_partition(dg)
    }
    expect_error(part_directed, "bipartite_partition needs an undirected graph")

    ? ----- topological_sort -----
    assert(graphs.topological_sort(dag) == [5, 2, 4, 0, 3, 1],
           "kahn order with insertion-order ties")
    bind topos = graphs.topological_sort(dag)
    bind tpos = {}
    bind ti = 0
    for v in topos {
        tpos[v] = ti
        ti = ti + 1
    }
    assert(length(topos) == dag["size"](), "every vertex appears exactly once")
    for e in dag["edges"]() {
        assert(tpos[e[0]] < tpos[e[1]], "every edge points forward in topo order")
    }
    bind cyc2 = graphs.graph(On)
    cyc2["add_edge"](1, 2)
    cyc2["add_edge"](2, 3)
    cyc2["add_edge"](3, 1)
    assert(graphs.topological_sort(cyc2) == null, "cyclic digraph gives null")
    spell topo_undirected() {
        graphs.topological_sort(g8)
    }
    expect_error(topo_undirected, "topological_sort needs a directed graph")

    ? ----- scc -----
    bind sccg = graphs.graph(On)
    sccg["add_edge"](0, 1)
    sccg["add_edge"](1, 2)
    sccg["add_edge"](2, 0)
    sccg["add_edge"](2, 3)
    sccg["add_edge"](3, 4)
    sccg["add_edge"](4, 5)
    sccg["add_edge"](5, 3)
    sccg["add_edge"](5, 6)
    sccg["add_edge"](6, 7)
    sccg["add_edge"](7, 6)
    bind got = canon_sccs(sccg, graphs.scc(sccg))
    assert(got == [[0, 1, 2], [3, 4, 5], [6, 7]], "three known SCCs, compared as sets of sets")
    bind singles = canon_sccs(dag, graphs.scc(dag))
    assert(singles == [[5], [2], [0], [4], [1], [3]], "DAG SCCs are singletons")
    spell scc_undirected() {
        graphs.scc(g8)
    }
    expect_error(scc_undirected, "scc needs a directed graph")

    ? ----- free-spell error paths -----
    spell bfs_unknown() {
        graphs.bfs(g8, 99)
    }
    expect_error(bfs_unknown, "unknown start vertex")
    spell dfs_unknown() {
        graphs.dfs(g8, 99)
    }
    expect_error(dfs_unknown, "unknown start vertex")
    spell path_start_unknown() {
        graphs.path(g8, 99, 0)
    }
    expect_error(path_start_unknown, "unknown start vertex")
    spell path_goal_unknown() {
        graphs.path(g8, 0, 99)
    }
    expect_error(path_goal_unknown, "unknown goal vertex")
    spell dijkstra_unknown() {
        graphs.dijkstra(wg, 99)
    }
    expect_error(dijkstra_unknown, "unknown source vertex")
    spell sp_start_unknown() {
        graphs.shortest_path(wg, 99, 0)
    }
    expect_error(sp_start_unknown, "unknown start vertex")
    spell sp_goal_unknown() {
        graphs.shortest_path(wg, 0, 99)
    }
    expect_error(sp_goal_unknown, "unknown goal vertex")

    cast "ALL GRAPHS TESTS PASSED"
}
