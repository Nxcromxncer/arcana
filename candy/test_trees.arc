? Candy test: trees — run with  arcana run candy/test_trees.arc
import trees
import math

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

? ---- oracle helpers: a sorted list kept by hand ----

spell insert_sorted(xs, v) {
    ? returns On if v was added, Off if it was already there
    bind i = 0
    while i < length(xs) and xs[i] < v {
        i = i + 1
    }
    if i < length(xs) and xs[i] == v {
        return Off
    }
    append(xs, null)
    bind j = length(xs) - 1
    while j > i {
        xs[j] = xs[j - 1]
        j = j - 1
    }
    xs[i] = v
    return On
}

spell oracle_remove(xs, v) {
    ? removes the first equal item in place; On if it was there
    bind at = -1
    bind i = 0
    while i < length(xs) {
        if xs[i] == v {
            at = i
            i = length(xs)
        } else {
            i = i + 1
        }
    }
    if at == -1 {
        return Off
    }
    bind j = at
    while j < length(xs) - 1 {
        xs[j] = xs[j + 1]
        j = j + 1
    }
    pop(xs)
    return On
}

spell sorted_copy(xs) {
    ? insertion sort into a fresh list, duplicates kept
    bind out = []
    for v in xs {
        append(out, v)
    }
    for i in range(1, length(out)) {
        bind cur = out[i]
        bind j = i - 1
        while j >= 0 and out[j] > cur {
            out[j + 1] = out[j]
            j = j - 1
        }
        out[j + 1] = cur
    }
    return out
}

spell ceil_log2(m) {
    bind h = 0
    bind p = 1
    while p < m {
        p = p * 2
        h = h + 1
    }
    return h
}

spell shuffled(n, seed) {
    ? Fisher-Yates over 0..n-1 with the seeded global generator
    math.random_seed(seed)
    bind xs = []
    for i in range(0, n) {
        append(xs, i)
    }
    bind i = n - 1
    while i > 0 {
        bind j = math.random_int(0, i)
        bind t = xs[i]
        xs[i] = xs[j]
        xs[j] = t
        i = i - 1
    }
    return xs
}

spell naive_sum(xs, start, end) {
    bind t = 0
    for i in range(start, end) {
        t = t + xs[i]
    }
    return t
}

spell count_occurrences(text, pat) {
    bind c = 0
    for i in range(0, length(text) - length(pat) + 1) {
        bind j = 0
        bind ok = On
        while j < length(pat) {
            if text[i + j] != pat[j] {
                ok = Off
                j = length(pat)
            } else {
                j = j + 1
            }
        }
        if ok {
            c = c + 1
        }
    }
    return c
}

spell is_strictly_sorted(xs) {
    for i in range(1, length(xs)) {
        if not (xs[i - 1] < xs[i]) {
            return Off
        }
    }
    return On
}

spell main() {
    ? ----- BinaryTree (positional, complete, level order) -----
    bind bt = trees.binary_tree()
    assert(bt["kind"] == "candy/binary-tree", "binary tree kind")
    assert(bt["is_empty"]() and bt["size"]() == 0 and bt["height"]() == 0,
           "binary tree starts empty")
    for i in range(1, 8) {
        bt["insert"](i)
    }
    assert(bt["size"]() == 7 and bt["height"]() == 3, "binary tree size/height")
    assert(bt["to_list"]() == [1, 2, 3, 4, 5, 6, 7], "binary tree level order")
    assert(bt["level_order"]() == [1, 2, 3, 4, 5, 6, 7], "level_order == to_list")
    assert(bt["inorder"]() == [4, 2, 5, 1, 6, 3, 7], "binary tree inorder")
    assert(bt["preorder"]() == [1, 2, 4, 5, 3, 6, 7], "binary tree preorder")
    assert(bt["postorder"]() == [4, 5, 2, 6, 7, 3, 1], "binary tree postorder")
    assert(bt["contains"](4) and not bt["contains"](9), "binary tree contains")
    bt["insert"](8)
    assert(bt["height"]() == 4 and bt["size"]() == 8, "binary tree grew a level")
    bind bt5 = trees.binary_tree()
    for i in range(1, 6) {
        bt5["insert"](i)
    }
    assert(bt5["inorder"]() == [4, 2, 5, 1, 3], "5-node inorder")
    assert(bt5["postorder"]() == [4, 5, 2, 3, 1], "5-node postorder")
    bt5["clear"]()
    assert(bt5["is_empty"]() and bt5["inorder"]() == [], "binary tree clear")

    ? ----- BST: sequential 1..200 (iterative must not hit depth limits) -----
    bind b1 = trees.bst()
    assert(b1["kind"] == "candy/bst", "bst kind")
    bind o1 = []
    for i in range(1, 201) {
        assert(b1["insert"](i) == insert_sorted(o1, i), "bst sequential insert")
    }
    assert(b1["to_list"]() == o1, "bst inorder sorted 1..200")
    assert(b1["min"]() == 1 and b1["max"]() == 200, "bst min/max")
    assert(b1["size"]() == 200, "bst size")
    assert(b1["height"]() == 200, "degenerate chain height 200")
    assert(b1["insert"](100) == Off and b1["size"]() == 200, "bst duplicates Off")
    for i in range(2, 201, 2) {
        assert(b1["remove"](i) == On, "bst remove even")
    }
    assert(b1["remove"](2) == Off, "bst remove absent")
    bind o1b = []
    for i in range(1, 201, 2) {
        append(o1b, i)
    }
    assert(b1["to_list"]() == o1b, "bst holds the odds")
    assert(b1["contains"](199) and not b1["contains"](200), "bst contains after removes")
    assert(b1["preorder"]()[0] == 1, "bst preorder starts at root")
    assert(length(b1["postorder"]()) == 100, "bst postorder length")
    b1["clear"]()
    assert(b1["is_empty"]() and b1["to_list"]() == [], "bst clear")

    ? ----- BST: degenerate chain of 100 still works -----
    bind chain = trees.bst()
    for i in range(1, 101) {
        chain["insert"](i)
    }
    assert(chain["height"]() == 100, "chain of 100 has height 100")
    assert(chain["contains"](50), "chain contains middle")
    assert(chain["remove"](50) == On, "chain remove middle")
    assert(chain["height"]() == 99, "chain height after remove")
    assert(is_strictly_sorted(chain["inorder"]()), "chain inorder sorted")

    ? ----- BST: seeded random ops vs oracle -----
    bind b2 = trees.bst()
    bind o2 = []
    math.random_seed(101)
    for k in range(0, 300) {
        bind v = math.random_int(0, 49)
        assert(b2["insert"](v) == insert_sorted(o2, v), "bst random insert vs oracle")
    }
    for k in range(0, 150) {
        bind v = math.random_int(0, 49)
        assert(b2["remove"](v) == oracle_remove(o2, v), "bst random remove vs oracle")
    }
    for v in range(0, 50) {
        assert(b2["contains"](v) == (v in o2), "bst contains vs oracle")
    }
    assert(b2["to_list"]() == o2, "bst inorder == oracle")
    assert(b2["min"]() == o2[0] and b2["max"]() == o2[length(o2) - 1], "bst min/max vs oracle")

    ? ----- AVL: 1000 sequential inserts -----
    bind av = trees.avl()
    assert(av["kind"] == "candy/avl", "avl kind")
    for i in range(0, 1000) {
        assert(av["insert"](i) == On, "avl sequential insert")
    }
    assert(av["is_balanced"](), "avl balanced after 1000 inserts")
    assert(av["height"]() <= 1.45 * ceil_log2(1001), "avl height within 1.45*log2")
    assert(av["min"]() == 0 and av["max"]() == 999, "avl min/max")
    bind avo = []
    for i in range(0, 1000) {
        append(avo, i)
    }
    assert(av["to_list"]() == avo, "avl inorder sorted 0..999")
    assert(av["remove"](500) == On and av["remove"](500) == Off, "avl remove once")
    assert(av["is_balanced"]() and av["size"]() == 999, "avl balanced after remove")

    ? ----- AVL: seeded random ops vs oracle -----
    bind av2 = trees.avl()
    bind avo2 = []
    math.random_seed(555)
    for k in range(0, 400) {
        bind v = math.random_int(0, 59)
        bind op = math.random_int(0, 2)
        if op == 0 {
            assert(av2["insert"](v) == insert_sorted(avo2, v), "avl random insert vs oracle")
        } else {
            assert(av2["remove"](v) == oracle_remove(avo2, v), "avl random remove vs oracle")
        }
    }
    assert(av2["to_list"]() == avo2, "avl inorder == oracle")
    assert(av2["is_balanced"](), "avl balanced after random ops")
    assert(av2["height"]() <= 1.45 * ceil_log2(length(avo2) + 1), "avl random height bound")

    ? ----- Red-black: seeded batches, is_valid after each -----
    bind rb = trees.red_black_tree()
    assert(rb["kind"] == "candy/red-black-tree", "rb kind")
    bind rbo = []
    math.random_seed(31337)
    for batch in range(0, 5) {
        for k in range(0, 50) {
            bind v = math.random_int(0, 199)
            assert(rb["insert"](v) == insert_sorted(rbo, v), "rb insert vs oracle")
        }
        assert(rb["is_valid"](), "rb valid after insert batch")
        for k in range(0, 25) {
            bind v = math.random_int(0, 199)
            assert(rb["remove"](v) == oracle_remove(rbo, v), "rb remove vs oracle")
        }
        assert(rb["is_valid"](), "rb valid after remove batch")
    }
    assert(rb["to_list"]() == rbo, "rb inorder == oracle")
    assert(is_strictly_sorted(rb["inorder"]()), "rb inorder strictly sorted")
    assert(rb["size"]() == length(rbo), "rb size")
    assert(rb["height"]() <= 2 * ceil_log2(length(rbo) + 1), "rb height <= 2*log2(n+1)")
    assert(rb["contains"](rbo[0]) and not rb["contains"](-1), "rb contains")
    rb["clear"]()
    assert(rb["is_valid"]() and rb["is_empty"](), "rb clear")

    ? ----- BinaryHeap: from-build == push-built, drains sorted -----
    bind hs = []
    math.random_seed(8080)
    for i in range(0, 40) {
        append(hs, math.random_int(-99, 99))
    }
    bind hp = trees.binary_heap()
    assert(hp["kind"] == "candy/binary-heap", "heap kind")
    for v in hs {
        hp["push"](v)
    }
    bind hf = trees.binary_heap_from(hs)
    bind dp = []
    bind df = []
    while not hp["is_empty"]() {
        append(dp, hp["pop"]())
    }
    while not hf["is_empty"]() {
        append(df, hf["pop"]())
    }
    assert(dp == df, "heap from-build matches push-built")
    assert(dp == sorted_copy(hs), "min-heap drains ascending")
    bind hmax = trees.binary_heap_from(hs, On)
    bind dmax = []
    while not hmax["is_empty"]() {
        append(dmax, hmax["pop"]())
    }
    bind asc = sorted_copy(hs)
    bind want_desc = []
    bind r = length(asc) - 1
    while r >= 0 {
        append(want_desc, asc[r])
        r = r - 1
    }
    assert(dmax == want_desc, "max-heap drains descending")
    bind hp2 = trees.binary_heap_from([5])
    assert(hp2["peek"]() == 5 and hp2["size"]() == 1, "heap peek/size")
    assert(length(hp2["to_list"]()) == 1, "heap to_list snapshot")
    assert(sorted_copy(hp2["to_list"]())[0] == 5, "heap to_list content")
    hp2["clear"]()
    assert(hp2["size"]() == 0, "heap clear")

    ? ----- Trie -----
    bind tr = trees.trie()
    assert(tr["kind"] == "candy/trie", "trie kind")
    for w in ["cat", "car", "card", "dog"] {
        tr["insert"](w)
    }
    assert(tr["size"]() == 4, "trie size")
    assert(tr["contains"]("car") and tr["contains"]("card"), "trie contains words")
    assert(tr["contains"]("cat") and tr["contains"]("dog"), "trie contains more")
    assert(not tr["contains"]("ca") and not tr["contains"]("cars"), "trie prefixes are not words")
    assert(not tr["contains"]("cow"), "trie missing word")
    assert(tr["starts_with"]("ca") and tr["starts_with"]("card"), "trie starts_with")
    assert(tr["starts_with"](""), "empty prefix always matches")
    assert(not tr["starts_with"]("caz") and not tr["starts_with"]("dox"), "trie misses")
    assert(tr["words_with_prefix"]("car") == ["car", "card"], "trie words_with_prefix car")
    assert(tr["words_with_prefix"]("c") == ["car", "card", "cat"], "trie words_with_prefix c")
    assert(tr["words_with_prefix"]("d") == ["dog"], "trie words_with_prefix d")
    assert(tr["words_with_prefix"]("x") == [], "trie words_with_prefix miss")
    assert(tr["longest_common_prefix"]() == "", "trie lcp across branches")
    assert(tr["remove"]("car") == On and tr["remove"]("car") == Off, "trie remove once")
    assert(not tr["contains"]("car"), "trie car gone")
    assert(tr["contains"]("card"), "trie remove keeps card")
    assert(tr["size"]() == 3, "trie size after remove")
    assert(tr["remove"]("card") == On, "trie remove card")
    assert(not tr["starts_with"]("card"), "trie starts_with card gone")
    assert(tr["to_list"]() == ["cat", "dog"], "trie to_list sorted")
    tr["insert"]("cat")
    assert(tr["size"]() == 2, "trie duplicate insert does not grow")
    bind tr2 = trees.trie()
    tr2["insert"]("car")
    assert(tr2["longest_common_prefix"]() == "car", "single word lcp")
    tr2["insert"]("card")
    assert(tr2["longest_common_prefix"]() == "car", "car/card lcp")
    tr2["insert"]("care")
    assert(tr2["longest_common_prefix"]() == "car", "car/card/care lcp")
    tr2["clear"]()
    assert(tr2["is_empty"]() and tr2["longest_common_prefix"]() == "", "trie clear")
    bind tr3 = trees.trie()
    assert(tr3["to_list"]() == [] and tr3["words_with_prefix"]("a") == [], "empty trie")
    tr3["insert"]("")
    assert(tr3["contains"]("") and tr3["words_with_prefix"]("") == [""], "empty word round-trip")

    ? ----- Segment tree: random updates/queries vs naive -----
    math.random_seed(2024)
    bind arr = []
    for i in range(0, 40) {
        append(arr, math.random_int(-50, 50))
    }
    bind seg = trees.segment_tree(arr)
    assert(seg["kind"] == "candy/segment-tree", "segment kind")
    assert(seg["size"]() == 40, "segment size")
    assert(seg["to_list"]() == arr, "segment to_list")
    for k in range(0, 60) {
        bind op = math.random_int(0, 1)
        if op == 0 {
            bind i = math.random_int(0, 39)
            bind v = math.random_int(-50, 50)
            seg["update"](i, v)
            arr[i] = v
        } else {
            bind s = math.random_int(0, 39)
            bind e = math.random_int(s, 40)
            assert(seg["query"](s, e) == naive_sum(arr, s, e), "segment query vs naive")
        }
    }
    assert(seg["to_list"]() == arr, "segment to_list after updates")
    assert(seg["get"](7) == arr[7], "segment get")
    assert(seg["query"](0, 40) == naive_sum(arr, 0, 40), "segment full query")

    ? ----- Fenwick tree: random adds/range sums vs naive -----
    bind farr = []
    for i in range(0, 40) {
        append(farr, math.random_int(0, 20))
    }
    bind fw = trees.fenwick_tree(farr)
    assert(fw["kind"] == "candy/fenwick-tree", "fenwick kind")
    assert(fw["size"]() == 40, "fenwick size")
    for k in range(0, 60) {
        bind op = math.random_int(0, 1)
        if op == 0 {
            bind i = math.random_int(0, 39)
            bind d = math.random_int(-10, 10)
            fw["add"](i, d)
            farr[i] = farr[i] + d
        } else {
            bind s = math.random_int(0, 39)
            bind e = math.random_int(s, 40)
            assert(fw["range_sum"](s, e) == naive_sum(farr, s, e), "fenwick range_sum vs naive")
            assert(fw["prefix"](e) == naive_sum(farr, 0, e), "fenwick prefix vs naive")
        }
    }
    assert(fw["to_list"]() == farr, "fenwick to_list")
    bind fwz = trees.fenwick_tree(6)
    assert(fwz["size"]() == 6 and fwz["prefix"](6) == 0, "fenwick zeros constructor")
    fwz["add"](5, 4)
    assert(fwz["prefix"](6) == 4 and fwz["range_sum"](0, 6) == 4, "fenwick zeros add")
    fw["clear"]()
    assert(fw["size"]() == 0 and fw["to_list"]() == [], "fenwick clear")

    ? ----- B-tree t=2: seeded shuffled 0..99, remove half -----
    bind bt2 = trees.b_tree(2)
    assert(bt2["kind"] == "candy/b-tree", "b-tree kind")
    assert(bt2["height"]() == 0 and bt2["is_empty"](), "b-tree starts empty")
    bind vals = shuffled(100, 31)
    for v in vals {
        assert(bt2["insert"](v) == On, "b-tree insert")
    }
    assert(bt2["insert"](41) == Off, "b-tree duplicate Off")
    assert(bt2["is_valid"](), "b-tree valid after inserts")
    assert(bt2["to_list"]() == sorted_copy(vals), "b-tree to_list sorted")
    assert(bt2["height"]() >= 2, "b-tree grew height")
    for v in range(1, 100, 2) {
        assert(bt2["remove"](v) == On, "b-tree remove odd")
    }
    assert(bt2["remove"](1) == Off, "b-tree remove absent")
    assert(bt2["is_valid"](), "b-tree valid after removes")
    bind want_evens = []
    for v in range(0, 100, 2) {
        append(want_evens, v)
    }
    assert(bt2["to_list"]() == want_evens, "b-tree holds evens")
    assert(bt2["contains"](0) and bt2["contains"](98), "b-tree contains evens")
    assert(not bt2["contains"](1) and not bt2["contains"](99), "b-tree odds gone")
    assert(bt2["size"]() == 50, "b-tree size")
    spell btree_mixed() {
        bt2["insert"]("a")
    }
    expect_error(btree_mixed, "keys must be")

    ? ----- B-tree t=3 small case -----
    bind bt3 = trees.b_tree(3)
    bind v3 = shuffled(40, 77)
    for v in v3 {
        bt3["insert"](v)
    }
    assert(bt3["is_valid"](), "b-tree t3 valid after inserts")
    for v in range(0, 20) {
        assert(bt3["remove"](v) == On, "b-tree t3 remove")
    }
    assert(bt3["is_valid"](), "b-tree t3 valid after removes")
    bind want_t3 = []
    for v in range(20, 40) {
        append(want_t3, v)
    }
    assert(bt3["to_list"]() == want_t3, "b-tree t3 contents")

    ? ----- B+ tree: 0..49 with value = key*10 -----
    bind bp = trees.bplus_tree(2)
    assert(bp["kind"] == "candy/bplus-tree", "bplus kind")
    for k in range(0, 50) {
        bp["insert"](k, k * 10)
    }
    assert(bp["is_valid"](), "bplus valid after inserts")
    assert(bp["size"]() == 50 and bp["height"]() >= 2, "bplus size/height")
    assert(bp["range_query"](0, 5) == [0, 10, 20, 30, 40], "bplus range 0..5")
    assert(bp["range_query"](10, 20) == [100, 110, 120, 130, 140, 150, 160, 170, 180, 190],
           "bplus range 10..20")
    assert(bp["range_query"](45, 50) == [450, 460, 470, 480, 490], "bplus range 45..50")
    assert(bp["range_query"](7, 8) == [70], "bplus single-key range")
    assert(bp["range_query"](20, 20) == [], "bplus empty range")
    assert(bp["range_query"](48, 60) == [480, 490], "bplus range past end")
    bind oracle_pairs = []
    for k in range(0, 50) {
        append(oracle_pairs, [k, k * 10])
    }
    assert(bp["to_list"]() == oracle_pairs, "bplus to_list sorted pairs")
    assert(bp["items"]() == bp["to_list"](), "bplus items == to_list")
    assert(bp["get"](25) == 250 and bp["has"](25), "bplus get/has")
    assert(bp["get"](99) == null and not bp["has"](99), "bplus missing get/has")
    for k in range(0, 50, 2) {
        assert(bp["remove"](k) == On, "bplus remove even")
    }
    assert(bp["remove"](0) == Off, "bplus remove absent")
    assert(bp["is_valid"](), "bplus valid after removes")
    assert(bp["has"](1) and not bp["has"](2), "bplus has after removes")
    assert(bp["get"](3) == 30 and bp["get"](2, "gone") == "gone", "bplus get with default")
    bind want_pairs = []
    for k in range(1, 50, 2) {
        append(want_pairs, [k, k * 10])
    }
    assert(bp["to_list"]() == want_pairs, "bplus holds odds")
    assert(bp["range_query"](0, 10) == [10, 30, 50, 70, 90], "bplus range after removes")
    bp["insert"](3, 999)
    assert(bp["get"](3) == 999 and bp["size"]() == 25, "bplus insert updates value")
    assert(bp["is_valid"](), "bplus valid after update")
    spell bplus_mixed() {
        bp["insert"]("a", 1)
    }
    expect_error(bplus_mixed, "keys must be")

    ? ----- Suffix array on "banana" -----
    bind sa = trees.suffix_array("banana")
    assert(sa["kind"] == "candy/suffix-array", "suffix kind")
    assert(sa["size"]() == 6, "suffix size")
    assert(sa["suffixes"]() == [5, 3, 1, 0, 4, 2], "banana suffix array")
    assert(sa["occurrences"]("ana") == 2, "banana occurrences ana")
    assert(sa["occurrences"]("a") == 3, "banana occurrences a")
    assert(sa["occurrences"]("banana") == 1, "banana occurrences whole")
    assert(sa["contains"]("na"), "banana contains na")
    assert(not sa["contains"]("nab"), "banana not contains nab")
    assert(sa["contains_suffix"]("ana"), "banana suffix ana")
    assert(sa["contains_suffix"]("banana"), "banana suffix whole")
    assert(not sa["contains_suffix"]("nab"), "banana suffix nab Off")
    assert(sa["longest_repeating_substring"]() == "ana", "banana lrs")
    assert(sa["contains"]("b") and not sa["contains"]("z"), "banana single chars")

    ? ----- Suffix array: random binary text vs brute force -----
    math.random_seed(99)
    bind txt = ""
    for i in range(0, 60) {
        if math.random_int(0, 1) == 0 {
            txt = txt + "a"
        } else {
            txt = txt + "b"
        }
    }
    bind sa2 = trees.suffix_array(txt)
    for k in range(0, 6) {
        bind p0 = math.random_int(0, 56)
        bind plen = math.random_int(2, 4)
        bind pat = ""
        for j in range(0, plen) {
            pat = pat + txt[p0 + j]
        }
        assert(sa2["occurrences"](pat) == count_occurrences(txt, pat),
               "suffix occurrences vs brute force for " + pat)
        assert(sa2["contains"](pat) == (count_occurrences(txt, pat) > 0), "suffix contains vs brute")
    }
    bind flip = "a"
    if txt[0] == "a" {
        flip = "b"
    }
    bind pat2 = txt[10] + txt[11] + flip
    assert(sa2["occurrences"](pat2) == count_occurrences(txt, pat2), "near-miss pattern count")

    ? ----- Treap (seeded): 500 random ops vs oracle -----
    bind tp = trees.treap(12345)
    assert(tp["kind"] == "candy/treap", "treap kind")
    bind tor = []
    math.random_seed(6789)
    for k in range(0, 500) {
        bind v = math.random_int(0, 99)
        bind op = math.random_int(0, 3)
        if op == 0 or op == 1 {
            assert(tp["insert"](v) == insert_sorted(tor, v), "treap insert vs oracle")
        } else {
            if op == 2 {
                assert(tp["remove"](v) == oracle_remove(tor, v), "treap remove vs oracle")
            } else {
                assert(tp["contains"](v) == (v in tor), "treap contains vs oracle")
            }
        }
    }
    assert(tp["to_list"]() == tor, "treap to_list == oracle")
    assert(tp["size"]() == length(tor), "treap size")
    assert(tp["height"]() < 60, "treap height sane")
    assert(tp["min"]() == tor[0] and tp["max"]() == tor[length(tor) - 1], "treap min/max")
    tp["clear"]()
    assert(tp["is_empty"]() and tp["to_list"]() == [], "treap clear")

    ? ----- Error paths -----
    bind eb = trees.bst()
    spell bst_min_empty() {
        eb["min"]()
    }
    spell bst_max_empty() {
        eb["max"]()
    }
    expect_error(bst_min_empty, "empty")
    expect_error(bst_max_empty, "empty")
    eb["insert"](1)
    spell bst_mixed() {
        eb["insert"]("a")
    }
    expect_error(bst_mixed, "keys must be")

    bind eav = trees.avl()
    eav["insert"](5)
    spell avl_mixed() {
        eav["insert"]("a")
    }
    expect_error(avl_mixed, "keys must be")
    spell avl_min_empty() {
        trees.avl()["min"]()
    }
    expect_error(avl_min_empty, "empty")

    bind erb = trees.red_black_tree()
    erb["insert"](2)
    spell rb_mixed() {
        erb["insert"]("a")
    }
    expect_error(rb_mixed, "keys must be")
    spell rb_max_empty() {
        trees.red_black_tree()["max"]()
    }
    expect_error(rb_max_empty, "empty")

    bind eh = trees.binary_heap()
    spell heap_pop_empty() {
        eh["pop"]()
    }
    spell heap_peek_empty() {
        eh["peek"]()
    }
    expect_error(heap_pop_empty, "empty")
    expect_error(heap_peek_empty, "empty")
    bind ehf = trees.binary_heap_from([])
    spell heapf_pop_empty() {
        ehf["pop"]()
    }
    expect_error(heapf_pop_empty, "empty")
    eh["push"](1)
    spell heap_mixed() {
        eh["push"]("x")
    }
    expect_error(heap_mixed, "keys must be")

    bind etp = trees.treap()
    etp["insert"](3)
    spell treap_mixed() {
        etp["insert"]("a")
    }
    expect_error(treap_mixed, "keys must be")
    spell treap_min_empty() {
        trees.treap()["min"]()
    }
    spell treap_max_empty() {
        trees.treap(7)["max"]()
    }
    expect_error(treap_min_empty, "empty")
    expect_error(treap_max_empty, "empty")

    bind eseg = trees.segment_tree([1, 2, 3])
    spell seg_bad_range1() {
        eseg["query"](2, 1)
    }
    spell seg_bad_range2() {
        eseg["query"](-1, 2)
    }
    spell seg_bad_range3() {
        eseg["query"](0, 4)
    }
    spell seg_bad_get() {
        eseg["get"](3)
    }
    spell seg_bad_value() {
        eseg["update"](0, "x")
    }
    spell seg_bad_build() {
        trees.segment_tree([1, "a"])
    }
    expect_error(seg_bad_range1, "range")
    expect_error(seg_bad_range2, "range")
    expect_error(seg_bad_range3, "range")
    expect_error(seg_bad_get, "range")
    expect_error(seg_bad_value, "numbers")
    expect_error(seg_bad_build, "numbers")

    bind efw = trees.fenwick_tree(5)
    spell fw_bad_add() {
        efw["add"](5, 1)
    }
    spell fw_bad_prefix() {
        efw["prefix"](6)
    }
    spell fw_bad_build() {
        trees.fenwick_tree("nope")
    }
    expect_error(fw_bad_add, "range")
    expect_error(fw_bad_prefix, "range")
    expect_error(fw_bad_build, "integer or a list")

    bind esa = trees.suffix_array("banana")
    spell sa_empty_contains() {
        esa["contains"]("")
    }
    spell sa_empty_occ() {
        esa["occurrences"]("")
    }
    spell sa_empty_build() {
        trees.suffix_array("")
    }
    expect_error(sa_empty_contains, "empty pattern")
    expect_error(sa_empty_occ, "empty pattern")
    expect_error(sa_empty_build, "non-empty")

    ? constructor validation, reported like the pilot's capacity check
    bind saw_bad_t = Off
    try {
        trees.b_tree(1)
    } catch e {
        saw_bad_t = On
        assert("min_degree" in e["message"], "min_degree error message")
    }
    assert(saw_bad_t, "b-tree min_degree 1 rejected")
    bind saw_bad_bp = Off
    try {
        trees.bplus_tree(0)
    } catch e {
        saw_bad_bp = On
        assert("min_degree" in e["message"], "bplus min_degree error message")
    }
    assert(saw_bad_bp, "bplus min_degree 0 rejected")

    cast "ALL TREES TESTS PASSED"
}
