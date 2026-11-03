? Candy test: collections — run with  arcana run candy/test_collections.arc
import collections
from math import random_seed, random_int
from list import sorted

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

? ----- oracle helpers (mutate the caller's list in place) -----

spell list_insert_at(items, i, v) {
    append(items, null)
    bind j = length(items) - 1
    while j > i {
        items[j] = items[j - 1]
        j = j - 1
    }
    items[i] = v
    return null
}

spell list_remove_at(items, i) {
    bind v = items[i]
    bind j = i
    while j < length(items) - 1 {
        items[j] = items[j + 1]
        j = j + 1
    }
    pop(items)
    return v
}

spell list_push_front(items, v) {
    list_insert_at(items, 0, v)
    return null
}

spell index_in(items, v) {
    for j in range(0, length(items)) {
        if items[j] == v {
            return j
        }
    }
    return -1
}

? canonical form of a component list: members ascending, groups ordered
? by their smallest member (insertion sort — sorted() rejects lists-of-lists)
spell canon_groups(comp) {
    bind out = []
    for g in comp {
        append(out, sorted(g))
    }
    for i in range(1, length(out)) {
        bind cur = out[i]
        bind j = i - 1
        while j >= 0 and out[j][0] > cur[0] {
            out[j + 1] = out[j]
            j = j - 1
        }
        out[j + 1] = cur
    }
    return out
}

spell comp_index(comp, x) {
    bind i = 0
    for g in comp {
        if x in g {
            return i
        }
        i = i + 1
    }
    return -1
}

? ----- shared behavioral test for both linked lists -----

spell exercise_list(mk, kind_text) {
    bind lst = mk()
    assert(lst["kind"] == kind_text, kind_text + " kind")
    bind oracle = []
    ? scripted phase
    for i in range(0, 5) {
        lst["push_back"](i)
        append(oracle, i)
    }
    lst["push_front"](-1)
    list_push_front(oracle, -1)
    assert(lst["size"]() == 6 and lst["front"]() == -1 and lst["back"]() == 4,
           kind_text + " ends after push_front")
    assert(lst["pop_front"]() == -1, "pop_front returns old front")
    list_remove_at(oracle, 0)
    assert(lst["pop_back"]() == 4, "pop_back returns old back")
    list_remove_at(oracle, length(oracle) - 1)
    assert(lst["to_list"]() == oracle, "matches oracle after pops")
    lst["insert_at"](0, -2)
    list_insert_at(oracle, 0, -2)
    lst["insert_at"](lst["size"](), 5)
    list_insert_at(oracle, length(oracle), 5)
    lst["insert_at"](2, 99)
    list_insert_at(oracle, 2, 99)
    assert(lst["remove_at"](0) == -2, "remove_at(0)")
    list_remove_at(oracle, 0)
    assert(lst["remove_at"](lst["size"]() - 1) == 5, "remove_at(size - 1)")
    list_remove_at(oracle, length(oracle) - 1)
    assert(lst["remove"](99), "remove(99) found")
    list_remove_at(oracle, index_in(oracle, 99))
    assert(not lst["remove"](777), "remove(777) missing")
    assert(lst["index_of"](2) == index_in(oracle, 2), "index_of found")
    assert(lst["index_of"](777) == null, "index_of missing is null")
    assert(lst["contains"](3) and not lst["contains"](777), "contains")
    assert(lst["get"](0) == 0 and lst["get"](3) == 3, "get after churn")
    lst["reverse"]()
    bind reversed_oracle = []
    for x in oracle {
        list_push_front(reversed_oracle, x)
    }
    oracle = reversed_oracle
    assert(lst["to_list"]() == oracle, "reverse in place")
    ? random phase — same seed for both lists, so they see identical ops
    random_seed(2202)
    for r in range(0, 300) {
        bind op = random_int(0, 6)
        if op == 0 {
            bind v = random_int(0, 50)
            lst["push_back"](v)
            append(oracle, v)
        }
        if op == 1 {
            bind v = random_int(0, 50)
            lst["push_front"](v)
            list_push_front(oracle, v)
        }
        if op == 2 and length(oracle) > 0 {
            assert(lst["pop_front"]() == oracle[0], "random pop_front")
            list_remove_at(oracle, 0)
        }
        if op == 3 and length(oracle) > 0 {
            assert(lst["pop_back"]() == oracle[length(oracle) - 1], "random pop_back")
            pop(oracle)
        }
        if op == 4 {
            bind i = random_int(0, length(oracle))
            bind v = random_int(0, 50)
            lst["insert_at"](i, v)
            list_insert_at(oracle, i, v)
        }
        if op == 5 and length(oracle) > 0 {
            bind i = random_int(0, length(oracle) - 1)
            assert(lst["remove_at"](i) == oracle[i], "random remove_at")
            list_remove_at(oracle, i)
        }
        if op == 6 {
            bind v = random_int(0, 50)
            bind want = index_in(oracle, v) != -1
            assert(lst["remove"](v) == want, "random remove matches oracle")
            if want {
                list_remove_at(oracle, index_in(oracle, v))
            }
        }
        if r % 50 == 49 {
            assert(lst["to_list"]() == oracle, "list matches oracle mid-run")
        }
    }
    assert(lst["to_list"]() == oracle, "list matches oracle after random ops")
    assert(lst["size"]() == length(oracle), "size matches oracle")
    ? clear, then misuse on the empty list
    lst["clear"]()
    assert(lst["is_empty"]() and lst["size"]() == 0 and lst["to_list"]() == [],
           "clear empties the list")
    spell pop_front_empty() {
        lst["pop_front"]()
    }
    spell pop_back_empty() {
        lst["pop_back"]()
    }
    spell front_empty() {
        lst["front"]()
    }
    spell back_empty() {
        lst["back"]()
    }
    spell get_oob() {
        lst["get"](0)
    }
    expect_error(pop_front_empty, "empty")
    expect_error(pop_back_empty, "empty")
    expect_error(front_empty, "empty")
    expect_error(back_empty, "empty")
    expect_error(get_oob, "range")
    return null
}

spell main() {
    ? ----- Dynamic array -----
    random_seed(1101)
    bind da = collections.dynamic_array()
    assert(da["kind"] == "candy/dynamic-array", "dynamic array kind")
    assert(da["is_empty"]() and da["size"]() == 0 and da["capacity"]() == 8,
           "array starts empty at capacity 8")
    bind oracle_arr = []
    for i in range(0, 300) {
        bind v = random_int(0, 999)
        da["append"](v)
        append(oracle_arr, v)
    }
    assert(da["size"]() == 300, "300 appends")
    assert(da["capacity"]() >= 300, "capacity grew to fit")
    bind cap_pow = da["capacity"]()
    while cap_pow % 2 == 0 {
        cap_pow = cap_pow // 2
    }
    assert(cap_pow == 1, "capacity is a power of two (doubling)")
    for r in range(0, 400) {
        bind op = random_int(0, 3)
        if op == 0 {
            bind gi = random_int(0, length(oracle_arr) - 1)
            assert(da["get"](gi) == oracle_arr[gi], "get matches oracle")
        }
        if op == 1 {
            bind si = random_int(0, length(oracle_arr) - 1)
            bind sv = random_int(0, 999)
            da["set"](si, sv)
            oracle_arr[si] = sv
        }
        if op == 2 {
            bind ii = random_int(0, length(oracle_arr))
            bind iv = random_int(0, 999)
            da["insert"](ii, iv)
            list_insert_at(oracle_arr, ii, iv)
        }
        if op == 3 {
            bind ri = random_int(0, length(oracle_arr) - 1)
            assert(da["remove_at"](ri) == oracle_arr[ri], "remove_at matches oracle")
            list_remove_at(oracle_arr, ri)
        }
        assert(da["capacity"]() >= da["size"](), "capacity always covers size")
    }
    assert(da["to_list"]() == oracle_arr, "array matches oracle after random ops")
    ? shrink path: 200 appends then drain to 20 halves 256 -> 128 -> 64
    bind shrink = collections.dynamic_array()
    for i in range(0, 200) {
        shrink["append"](i)
    }
    assert(shrink["capacity"]() == 256, "capacity 256 after 200 appends")
    while shrink["size"]() > 20 {
        shrink["remove_at"](0)
    }
    assert(shrink["size"]() == 20, "drained to 20")
    assert(shrink["capacity"]() == 64, "capacity shrank to 64")
    ? pop / remove / index_of / contains
    bind da2 = collections.dynamic_array()
    da2["append"](5)
    da2["append"](7)
    da2["append"](5)
    assert(da2["pop"]() == 5, "pop returns last")
    assert(da2["remove"](5), "remove first occurrence")
    assert(da2["to_list"]() == [7], "one element left")
    assert(not da2["remove"](5), "remove missing is Off")
    da2["append"](9)
    assert(da2["index_of"](9) == 1 and da2["index_of"](5) == null, "index_of")
    assert(da2["contains"](7) and not da2["contains"](5), "contains")
    assert(da2["get"](0) == 7 and da2["get"](1) == 9, "get after churn")
    da2["clear"]()
    assert(da2["size"]() == 0 and da2["capacity"]() == 8, "clear resets")
    spell pop_empty_da() {
        da2["pop"]()
    }
    spell get_oob_da() {
        da2["get"](0)
    }
    spell insert_past_end() {
        da2["insert"](1, 5)
    }
    spell insert_negative() {
        da2["insert"](-1, 5)
    }
    spell set_oob_da() {
        da2["set"](0, 1)
    }
    spell remove_at_oob_da() {
        da2["remove_at"](0)
    }
    expect_error(pop_empty_da, "empty")
    expect_error(get_oob_da, "range")
    expect_error(insert_past_end, "range")
    expect_error(insert_negative, "range")
    expect_error(set_oob_da, "range")
    expect_error(remove_at_oob_da, "range")

    ? ----- Singly and doubly linked lists (shared behavioral test) -----
    exercise_list(collections.singly_linked_list, "candy/singly-linked-list")
    exercise_list(collections.doubly_linked_list, "candy/doubly-linked-list")
    ? doubly-specific: to_list_reversed and O(1) pop_back correctness
    bind dl = collections.doubly_linked_list()
    for i in range(0, 5) {
        dl["push_back"](i)
    }
    assert(dl["to_list_reversed"]() == [4, 3, 2, 1, 0], "to_list_reversed")
    assert(dl["pop_back"]() == 4, "doubly pop_back correctness")
    assert(dl["to_list"]() == [0, 1, 2, 3] and dl["to_list_reversed"]() == [3, 2, 1, 0],
           "both directions consistent after pop_back")
    dl["reverse"]()
    assert(dl["to_list"]() == [3, 2, 1, 0] and dl["to_list_reversed"]() == [0, 1, 2, 3],
           "reverse keeps both directions")

    ? ----- Hash map -----
    random_seed(3303)
    bind hm = collections.hash_map()
    assert(hm["kind"] == "candy/hash-map", "hash map kind")
    assert(hm["size"]() == 0, "hash map starts empty")
    bind oracle_map = {}
    for r in range(0, 2000) {
        bind op = random_int(0, 3)
        bind key = 0
        if random_int(0, 1) == 0 {
            key = random_int(0, 99)
        } else {
            key = "k" + to_text(random_int(0, 99))
        }
        if op == 0 {
            bind v = random_int(0, 999)
            hm["set"](key, v)
            oracle_map[key] = v
        }
        if op == 1 {
            assert(hm["delete"](key) == (key in oracle_map), "hash map delete matches")
            if key in oracle_map {
                bind fresh = {}
                for k2 in oracle_map {
                    if k2 != key {
                        fresh[k2] = oracle_map[k2]
                    }
                }
                oracle_map = fresh
            }
        }
        if op == 2 {
            assert(hm["has"](key) == (key in oracle_map), "hash map has matches")
        }
        if op == 3 {
            bind want = "MISSING"
            if key in oracle_map {
                want = oracle_map[key]
            }
            assert(hm["get"](key, "MISSING") == want, "hash map get matches")
        }
    }
    assert(hm["size"]() == length(oracle_map), "hash map size matches")
    assert(hm["keys"]() == keys(oracle_map), "hash map keys order matches")
    bind want_items = []
    for k in oracle_map {
        append(want_items, [k, oracle_map[k]])
    }
    assert(hm["items"]() == want_items, "hash map items match")
    assert(hm["values"]() == values(oracle_map), "hash map values match")
    ? scripted interleave: insertion order survives deletes and updates
    bind hm2 = collections.hash_map()
    hm2["set"]("a", 1)
    hm2["set"]("b", 2)
    hm2["set"]("c", 3)
    hm2["delete"]("b")
    hm2["set"]("d", 4)
    hm2["set"]("a", 10)
    assert(hm2["keys"]() == ["a", "c", "d"], "insertion order after deletes")
    assert(hm2["get"]("a") == 10, "set over existing updates")
    hm2["delete"]("a")
    assert(hm2["keys"]() == ["c", "d"], "order after more deletes")
    assert(hm2["get"]("zz", 5) == 5 and hm2["get"]("zz") == null, "get default")
    hm2["set"](1, "int-one")
    hm2["set"]("1", "text-one")
    assert(hm2["get"](1) == "int-one" and hm2["get"]("1") == "text-one",
           "integer and text keys coexist")
    assert(hm2["has"](1) and hm2["has"]("1"), "has on both key types")
    spell bad_hash_key() {
        hm2["set"]([1, 2], 1)
    }
    spell bad_hash_key_get() {
        hm2["get"]({"a": 1})
    }
    expect_error(bad_hash_key, "text or integer")
    expect_error(bad_hash_key_get, "text or integer")

    ? ----- Hash set -----
    random_seed(4404)
    bind hs = collections.hash_set()
    assert(hs["kind"] == "candy/hash-set", "hash set kind")
    bind oracle_set = {}
    for r in range(0, 1000) {
        bind v = random_int(0, 149)
        bind op = random_int(0, 2)
        if op == 0 {
            bind want_add = not (v in oracle_set)
            assert(hs["add"](v) == want_add, "set add reports new-ness")
            if want_add {
                oracle_set[v] = On
            }
        }
        if op == 1 {
            assert(hs["delete"](v) == (v in oracle_set), "set delete matches")
            if v in oracle_set {
                bind fresh = {}
                for v2 in oracle_set {
                    if v2 != v {
                        fresh[v2] = On
                    }
                }
                oracle_set = fresh
            }
        }
        if op == 2 {
            assert(hs["has"](v) == (v in oracle_set), "set has matches")
        }
    }
    assert(hs["size"]() == length(oracle_set), "set size matches")
    assert(sorted(hs["to_list"]()) == sorted(keys(oracle_set)), "set contents match")
    bind hs2 = collections.hash_set()
    assert(hs2["add"]("x"), "first add is On")
    assert(not hs2["add"]("x"), "second add is Off")
    assert(hs2["size"]() == 1, "duplicate add keeps size 1")
    assert(hs2["delete"]("x") and not hs2["delete"]("x"), "set delete twice")
    spell bad_set_value() {
        hs2["add"]([1])
    }
    expect_error(bad_set_value, "text or integer")

    ? ----- Ordered map / ordered set -----
    bind om = collections.ordered_map()
    assert(om["kind"] == "candy/ordered-map", "ordered map kind")
    om["set"]("a", 1)
    om["set"]("b", 2)
    om["set"]("c", 3)
    om["delete"]("b")
    om["set"]("d", 4)
    assert(om["keys"]() == ["a", "c", "d"], "ordered map order after delete")
    assert(om["items"]() == [["a", 1], ["c", 3], ["d", 4]], "ordered map items")
    assert(om["values"]() == [1, 3, 4], "ordered map values")
    assert(om["has"]("c") and not om["has"]("b"), "ordered map has")
    assert(om["get"]("c") == 3 and om["get"]("x", 9) == 9, "ordered map get")
    assert(om["size"]() == 3, "ordered map size")
    assert(not om["delete"]("b"), "ordered map delete missing")
    om["clear"]()
    assert(om["size"]() == 0 and om["keys"]() == [], "ordered map clear")
    bind os = collections.ordered_set()
    assert(os["kind"] == "candy/ordered-set", "ordered set kind")
    assert(os["add"](1) and os["add"](2) and os["add"](3), "ordered set adds")
    assert(not os["add"](2), "ordered set duplicate add")
    assert(os["to_list"]() == [1, 2, 3], "ordered set insertion order")
    assert(os["has"](2) and not os["has"](9), "ordered set has")
    assert(os["delete"](2) and not os["delete"](2), "ordered set delete twice")
    assert(os["to_list"]() == [1, 3] and os["size"]() == 2, "ordered set after delete")
    os["clear"]()
    assert(os["size"]() == 0 and os["to_list"]() == [], "ordered set clear")

    ? ----- DSU -----
    random_seed(5505)
    bind n_dsu = 50
    bind d = collections.dsu(n_dsu)
    assert(d["kind"] == "candy/dsu", "dsu kind")
    assert(d["size"]() == n_dsu, "dsu size is n")
    assert(d["count"]() == n_dsu, "dsu starts with n components")
    bind comp = []
    for i in range(0, n_dsu) {
        append(comp, [i])
    }
    for r in range(0, 200) {
        bind x = random_int(0, n_dsu - 1)
        bind y = random_int(0, n_dsu - 1)
        bind ix = comp_index(comp, x)
        bind iy = comp_index(comp, y)
        bind want_merge = ix != iy
        assert(d["union"](x, y) == want_merge, "union reports merge-ness")
        if want_merge {
            bind merged = []
            for m in comp[ix] {
                append(merged, m)
            }
            for m in comp[iy] {
                append(merged, m)
            }
            bind hi = ix
            bind lo = iy
            if ix < iy {
                hi = iy
                lo = ix
            }
            list_remove_at(comp, hi)
            list_remove_at(comp, lo)
            append(comp, merged)
        }
        assert(d["count"]() == length(comp), "count matches components")
    }
    for t in range(0, 50) {
        bind x = random_int(0, n_dsu - 1)
        bind y = random_int(0, n_dsu - 1)
        assert(d["connected"](x, y) == (comp_index(comp, x) == comp_index(comp, y)),
               "connected matches oracle")
    }
    for x in range(0, n_dsu) {
        assert(d["component_size"](x) == length(comp[comp_index(comp, x)]),
               "component_size matches oracle")
    }
    assert(d["components"]() == canon_groups(comp), "components match canonical oracle")
    bind d2 = collections.dsu(5)
    assert(d2["union"](1, 2), "first union On")
    assert(not d2["union"](1, 2), "second union Off")
    assert(d2["connected"](1, 2), "connected after union")
    assert(not d2["connected"](1, 3), "not connected across components")
    assert(d2["component_size"](2) == 2, "component size 2")
    assert(d2["count"]() == 4, "count 4 after one union")
    spell find_oob() {
        d2["find"](5)
    }
    spell find_negative() {
        d2["find"](-1)
    }
    spell dsu_bad_n() {
        collections.dsu(-1)
    }
    expect_error(find_oob, "range")
    expect_error(find_negative, "range")
    expect_error(dsu_bad_n, "n must")

    ? ----- Bloom filter -----
    bind bf = collections.bloom_filter(1000, 0.01)
    assert(bf["kind"] == "candy/bloom-filter", "bloom kind")
    assert(bf["capacity"]() >= 1000, "m at least expected")
    assert(bf["hash_count"]() >= 1, "k at least 1")
    assert(bf["adds"]() == 0 and bf["bit_count"]() == 0, "fresh bloom is empty")
    for i in range(0, 400) {
        bf["add"]("key" + to_text(i))
    }
    assert(bf["adds"]() == 400, "adds counter")
    for i in range(0, 400) {
        assert(bf["might_contain"]("key" + to_text(i)), "no false negatives")
    }
    bind fp = 0
    for i in range(0, 400) {
        if bf["might_contain"]("unseen" + to_text(i)) {
            fp = fp + 1
        }
    }
    assert(fp < 32, "false positives under 8 percent, got " + to_text(fp))
    assert(bf["bit_count"]() <= bf["capacity"](), "bit_count bounded by m")
    assert(bf["bit_count"]() > 0, "bits were set")
    bind est = bf["estimated_fp"]()
    bind m_bf = bf["capacity"]()
    bind k_bf = bf["hash_count"]()
    bind want_fill = 1 - (1 - 1 / m_bf) ** (k_bf * 400)
    assert(abs(est - want_fill) < 0.0000001,
           "estimated_fp matches the set-bit fraction formula")
    assert(est > 0 and est < 1, "estimated fraction in (0, 1): " + to_text(est))
    assert(est ** k_bf < 0.08, "derived fp probability under 8 percent")
    bf["add"]("one-more")
    assert(bf["estimated_fp"]() > est, "estimate grows with adds")
    bf["add"](42)
    assert(bf["might_contain"](42), "integer items work")
    bf["clear"]()
    assert(bf["adds"]() == 0 and bf["bit_count"]() == 0, "clear resets counters")
    assert(not bf["might_contain"]("key1") and not bf["might_contain"](42),
           "nothing might_contain after clear")
    spell bad_item() {
        bf["add"]([1, 2])
    }
    spell bloom_bad_expected() {
        collections.bloom_filter(0, 0.5)
    }
    spell bloom_bad_rate() {
        collections.bloom_filter(100, 1.5)
    }
    expect_error(bad_item, "text or integer")
    expect_error(bloom_bad_expected, "expected")
    expect_error(bloom_bad_rate, "rate")

    ? ----- LRU cache -----
    bind lru = collections.lru_cache(3)
    assert(lru["kind"] == "candy/lru-cache", "lru kind")
    assert(lru["size"]() == 0 and lru["capacity"]() == 3, "lru fresh")
    lru["put"]("a", 1)
    lru["put"]("b", 2)
    lru["put"]("c", 3)
    assert(lru["keys"]() == ["c", "b", "a"], "keys MRU-first")
    assert(lru["to_list"]() == [["c", 3], ["b", 2], ["a", 1]], "to_list MRU-first")
    assert(lru["get"]("a") == 1, "lru get")
    assert(lru["keys"]() == ["a", "c", "b"], "get touches recency")
    assert(lru["has"]("b"), "lru has")
    assert(lru["keys"]() == ["a", "c", "b"], "has does not touch recency")
    lru["put"]("d", 4)
    assert(lru["keys"]() == ["d", "a", "c"], "put evicts the LRU item (b)")
    assert(not lru["has"]("b"), "evicted key is gone")
    assert(lru["get"]("b", -1) == -1, "get of missing returns default")
    assert(lru["size"]() == 3, "size stays at capacity")
    lru["put"]("c", 33)
    assert(lru["keys"]() == ["c", "d", "a"], "put on existing touches")
    assert(lru["get"]("c") == 33, "put on existing updates value")
    assert(lru["delete"]("d"), "delete On")
    assert(not lru["delete"]("d"), "delete Off when missing")
    assert(lru["keys"]() == ["c", "a"], "after delete")
    lru["put"]("x", 1)
    lru["put"]("y", 2)
    assert(lru["keys"]() == ["y", "x", "c"], "over-capacity evicted LRU (a)")
    lru["clear"]()
    assert(lru["size"]() == 0 and lru["keys"]() == [], "lru clear")
    ? churn: exercises slot reuse, stale map entries and compaction
    bind lru5 = collections.lru_cache(3)
    bind live = []
    for i in range(0, 200) {
        bind key = "n" + to_text(i)
        lru5["put"](key, i)
        list_push_front(live, key)
        if length(live) > 3 {
            pop(live)
        }
        if i >= 3 {
            assert(lru5["keys"]() == live, "churn keys MRU-first")
            assert(lru5["size"]() == 3, "churn size")
        }
    }
    assert(not lru5["has"]("n0"), "oldest churned key evicted")
    assert(lru5["get"]("n199") == 199, "churned key n199 intact")
    assert(lru5["get"]("n198") == 198, "churned key n198 intact")
    assert(lru5["get"]("n197") == 197, "churned key n197 intact")
    lru5["delete"]("n199")
    lru5["put"]("n199", 2000)
    assert(lru5["get"]("n199") == 2000, "delete then re-put after slot churn")
    lru5["put"]("n0", 1000)
    assert(lru5["get"]("n0") == 1000, "evicted key can return")
    spell lru_bad_cap() {
        collections.lru_cache(0)
    }
    expect_error(lru_bad_cap, "capacity")

    ? ----- LFU cache -----
    bind lfu = collections.lfu_cache(3)
    assert(lfu["kind"] == "candy/lfu-cache", "lfu kind")
    lfu["put"]("a", 1)
    lfu["put"]("b", 2)
    lfu["put"]("c", 3)
    lfu["get"]("a")
    lfu["get"]("a")
    lfu["put"]("d", 4)
    assert(not lfu["has"]("b"), "lfu evicts lowest-frequency b")
    assert(lfu["keys"]() == ["d", "a", "c"], "lfu keys MRU-first after eviction")
    assert(lfu["get"]("a") == 1 and lfu["get"]("d") == 4, "lfu values intact")
    ? tie-break: equal frequencies, oldest stamp leaves first
    bind lfu2 = collections.lfu_cache(2)
    lfu2["put"]("x", 1)
    lfu2["put"]("y", 2)
    lfu2["put"]("z", 3)
    assert(not lfu2["has"]("x") and lfu2["has"]("y"), "oldest freq-1 evicted")
    ? put on an existing key bumps its frequency
    bind lfu3 = collections.lfu_cache(2)
    lfu3["put"]("p", 1)
    lfu3["put"]("q", 2)
    lfu3["put"]("p", 10)
    lfu3["put"]("r", 3)
    assert(not lfu3["has"]("q") and lfu3["has"]("p"), "put bumps frequency")
    assert(lfu3["get"]("p") == 10, "lfu updated value")
    ? get bumps frequency too
    bind lfu4 = collections.lfu_cache(2)
    lfu4["put"]("m", 1)
    lfu4["put"]("n", 2)
    lfu4["get"]("m")
    lfu4["put"]("o", 3)
    assert(not lfu4["has"]("n"), "get bumps frequency")
    assert(lfu4["delete"]("o"), "lfu delete")
    lfu4["put"]("w", 9)
    assert(lfu4["get"]("w", -1) == 9, "lfu works after delete")
    lfu4["clear"]()
    assert(lfu4["size"]() == 0 and lfu4["keys"]() == [], "lfu clear")
    ? churn: same slot machinery as the LRU cache under constant eviction
    bind lfu5 = collections.lfu_cache(2)
    for i in range(0, 100) {
        lfu5["put"]("m" + to_text(i), i)
        if i >= 2 {
            assert(lfu5["size"]() == 2, "lfu churn size")
        }
    }
    assert(not lfu5["has"]("m0") and lfu5["has"]("m99"), "lfu churn eviction")
    assert(lfu5["get"]("m99") == 99, "lfu churn value")
    spell lfu_bad_cap() {
        collections.lfu_cache(0)
    }
    expect_error(lfu_bad_cap, "capacity")

    ? ----- Skip list -----
    random_seed(7707)
    bind sl = collections.skip_list()
    assert(sl["kind"] == "candy/skip-list", "skip list kind")
    assert(sl["size"]() == 0, "skip list starts empty")
    bind oracle_sl = []
    for r in range(0, 800) {
        bind op = random_int(0, 3)
        bind key = random_int(0, 199)
        if op == 0 {
            bind want_new = not (key in oracle_sl)
            assert(sl["add"](key, key * 10) == want_new, "skip add reports new-ness")
            if want_new {
                append(oracle_sl, key)
            }
        }
        if op == 1 {
            bind present = key in oracle_sl
            assert(sl["delete"](key) == present, "skip delete matches")
            if present {
                list_remove_at(oracle_sl, index_in(oracle_sl, key))
            }
        }
        if op == 2 {
            assert(sl["has"](key) == (key in oracle_sl), "skip has matches")
        }
        if op == 3 {
            bind want = -1
            if key in oracle_sl {
                want = key * 10
            }
            assert(sl["get"](key, -1) == want, "skip get matches")
        }
        if r % 100 == 99 {
            assert(sl["to_list"]() == sorted(oracle_sl), "skip list sorted mid-run")
            assert(sl["size"]() == length(oracle_sl), "skip size mid-run")
        }
    }
    assert(sl["to_list"]() == sorted(oracle_sl), "skip list sorted at end")
    bind oracle_sorted = sorted(oracle_sl)
    bind want_skip_items = []
    for k in oracle_sorted {
        append(want_skip_items, [k, k * 10])
    }
    assert(sl["items"]() == want_skip_items, "skip items sorted with values")
    for t in range(0, 20) {
        bind rs = random_int(-10, 210)
        bind re = random_int(-10, 210)
        bind want_range = []
        for k in oracle_sorted {
            if k >= rs and k < re {
                append(want_range, k)
            }
        }
        assert(sl["range"](rs, re) == want_range, "skip range matches oracle slice")
    }
    assert(sl["first"]() == oracle_sorted[0], "skip first")
    assert(sl["last"]() == oracle_sorted[length(oracle_sorted) - 1], "skip last")
    bind probe_key = oracle_sorted[0]
    sl["set"](probe_key, "changed")
    assert(sl["get"](probe_key) == "changed", "set overwrites value")
    sl["set"](5000, "new")
    assert(sl["has"](5000) and sl["get"](5000) == "new", "set inserts new key")
    assert(sl["delete"](5000), "delete the inserted key")
    ? text keys
    bind sls = collections.skip_list()
    sls["add"]("pear", 1)
    sls["add"]("apple", 2)
    sls["add"]("fig", 3)
    assert(sls["to_list"]() == ["apple", "fig", "pear"], "text keys sorted")
    assert(sls["first"]() == "apple" and sls["last"]() == "pear", "text first/last")
    assert(sls["range"]("b", "q") == ["fig", "pear"], "text range")
    assert(sls["get"]("fig") == 3, "text get")
    spell mixed_key() {
        sls["add"](7)
    }
    expect_error(mixed_key, "all numbers or all texts")
    ? empty and constructor misuse
    bind slx = collections.skip_list()
    spell first_empty() {
        slx["first"]()
    }
    spell last_empty() {
        slx["last"]()
    }
    spell bad_key_type() {
        slx["add"]([1, 2])
    }
    spell bad_p() {
        collections.skip_list(16, 1.5)
    }
    spell bad_level() {
        collections.skip_list(0, 0.5)
    }
    spell bad_level_type() {
        collections.skip_list(2.5, 0.5)
    }
    expect_error(first_empty, "empty")
    expect_error(last_empty, "empty")
    expect_error(bad_key_type, "all numbers or all texts")
    expect_error(bad_p, "p")
    expect_error(bad_level, "max_level")
    expect_error(bad_level_type, "max_level")

    cast "ALL COLLECTIONS TESTS PASSED"
}
