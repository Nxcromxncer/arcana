? Candy 03: collections — hash maps, caches, unions, skip lists and more.
?
? Part of the Candy library (Tier I, module 03 of 04).
? Import from your grimoire with:
?
?     import candy.collections          ? when the candy/ folder sits next to
?                                       ? your file, or is on ARCANA_PATH
?     import collections                 ? when this file sits next to yours
?
? Structures in this module: DynamicArray, SinglyLinkedList, DoublyLinkedList,
? HashMap, HashSet, OrderedMap, OrderedSet, DSU, BloomFilter, LRUCache,
? LFUCache, SkipList.
?
? Design: Arcana has no user-defined types, so every Candy structure is a
? *closure object*: `hash_map()` declares private state, inner spells
? that touch it, and returns a map of those spells. Call methods with
? indexing:
?
?     bind m = hash_map()
?     m["set"]("gem", 3)
?     cast m["get"]("gem")               ? 3
?
? Every instance carries a "kind" text ("candy/hash-map", ...) so programs
? can tell structures apart, plus size() and friends where they make sense.
? Misuse (a non-text key, an out-of-range element, ...) raises a catchable
? AssertionError through assert(), with a message starting "Candy: ".
?
? A note on Arcana maps: they keep insertion order but have no in-place key
? deletion, so "delete" everywhere below means "rebuild the map without the
? key" (survivors keep their order). That is O(size) per delete and it is why
? hash_map and hash_set are real bucketed hash tables instead of
? thin wrappers.

from text import code_of
from math import log, ceil, random

? ---------------------------------------------------------------------------
? Shared hash helpers — integers mix through big multipliers (Arcana integers
? never overflow), texts use a polynomial rolling hash over code points.
? With a positive divisor, Arcana's % already yields a result in 0..m-1 even
? for negative hashes (floor semantics), so callers can index straight away.
? ---------------------------------------------------------------------------

spell _hash_int(n, salt) {
    bind h = (n + salt) * 2654435761 + 1013904223
    h = h * 668265261 + 374761393
    return h
}

spell _hash_text(s, salt) {
    bind h = 5381 + salt
    for ch in s {
        h = h * 33 + code_of(ch)
    }
    return h
}

spell _hash_key(k, salt = 0) {
    if type_of(k) == "integer" {
        return _hash_int(k, salt)
    }
    return _hash_text(k, salt)
}

? ---------------------------------------------------------------------------
? DynamicArray — a manual capacity-doubling array
? ---------------------------------------------------------------------------

spell dynamic_array() {
    bind list_append = append
    bind buf = []
    bind cap = 8
    for i in range(0, cap) {
        list_append(buf, null)
    }
    bind n = 0
    spell rebuild(new_cap) {
        bind fresh = []
        for i in range(0, new_cap) {
            list_append(fresh, null)
        }
        for i in range(0, n) {
            fresh[i] = buf[i]
        }
        buf = fresh
        cap = new_cap
        return null
    }
    spell assert_index(i, upper, allow_end) {
        assert(type_of(i) == "integer", "Candy: index must be an integer")
        if allow_end {
            assert(i >= 0 and i <= upper, "Candy: insert position out of range")
        } else {
            assert(i >= 0 and i < upper, "Candy: index out of range")
        }
    }
    spell append(v) {
        if n == cap {
            rebuild(cap * 2)
        }
        buf[n] = v
        n = n + 1
        return null
    }
    spell pop() {
        assert(n > 0, "Candy: pop from an empty dynamic array")
        bind v = buf[n - 1]
        buf[n - 1] = null
        n = n - 1
        if n * 4 <= cap and cap > 8 {
            rebuild(cap // 2)
        }
        return v
    }
    spell get(i) {
        assert_index(i, n, Off)
        return buf[i]
    }
    spell set(i, v) {
        assert_index(i, n, Off)
        buf[i] = v
        return null
    }
    spell insert(i, v) {
        assert_index(i, n, On)
        if n == cap {
            rebuild(cap * 2)
        }
        bind j = n
        while j > i {
            buf[j] = buf[j - 1]
            j = j - 1
        }
        buf[i] = v
        n = n + 1
        return null
    }
    spell remove_at(i) {
        assert_index(i, n, Off)
        bind v = buf[i]
        bind j = i
        while j < n - 1 {
            buf[j] = buf[j + 1]
            j = j + 1
        }
        buf[n - 1] = null
        n = n - 1
        if n * 4 <= cap and cap > 8 {
            rebuild(cap // 2)
        }
        return v
    }
    spell remove(v) {
        for i in range(0, n) {
            if buf[i] == v {
                remove_at(i)
                return On
            }
        }
        return Off
    }
    spell contains(v) {
        for i in range(0, n) {
            if buf[i] == v {
                return On
            }
        }
        return Off
    }
    spell index_of(v) {
        for i in range(0, n) {
            if buf[i] == v {
                return i
            }
        }
        return null
    }
    spell size() {
        return n
    }
    spell capacity() {
        return cap
    }
    spell is_empty() {
        return n == 0
    }
    spell to_list() {
        bind out = []
        for i in range(0, n) {
            list_append(out, buf[i])
        }
        return out
    }
    spell clear() {
        n = 0
        rebuild(8)
        return null
    }
    return {"kind": "candy/dynamic-array", "append": append, "pop": pop,
            "get": get, "set": set, "insert": insert, "remove_at": remove_at,
            "remove": remove, "contains": contains, "index_of": index_of,
            "size": size, "capacity": capacity, "is_empty": is_empty,
            "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? SinglyLinkedList — nodes {"v", "next"} with head and tail pointers
? ---------------------------------------------------------------------------

spell singly_linked_list() {
    bind list_append = append
    bind head = null
    bind tail = null
    bind n = 0
    spell node_at(i) {
        bind cur = head
        bind j = 0
        while j < i {
            cur = cur["next"]
            j = j + 1
        }
        return cur
    }
    spell assert_index(i, upper, allow_end) {
        assert(type_of(i) == "integer", "Candy: index must be an integer")
        if allow_end {
            assert(i >= 0 and i <= upper, "Candy: insert position out of range")
        } else {
            assert(i >= 0 and i < upper, "Candy: index out of range")
        }
    }
    spell push_front(v) {
        bind node = {"v": v, "next": head}
        head = node
        if tail == null {
            tail = node
        }
        n = n + 1
        return null
    }
    spell push_back(v) {
        bind node = {"v": v, "next": null}
        if tail == null {
            head = node
        } else {
            tail["next"] = node
        }
        tail = node
        n = n + 1
        return null
    }
    spell pop_front() {
        assert(n > 0, "Candy: pop_front from an empty list")
        bind node = head
        head = node["next"]
        if head == null {
            tail = null
        }
        n = n - 1
        return node["v"]
    }
    spell pop_back() {
        assert(n > 0, "Candy: pop_back from an empty list")
        ? singly linked: finding the node before the tail costs O(n)
        if head == tail {
            bind only = head["v"]
            head = null
            tail = null
            n = n - 1
            return only
        }
        bind prev = head
        while prev["next"] != tail {
            prev = prev["next"]
        }
        bind v = tail["v"]
        prev["next"] = null
        tail = prev
        n = n - 1
        return v
    }
    spell front() {
        assert(n > 0, "Candy: front of an empty list")
        return head["v"]
    }
    spell back() {
        assert(n > 0, "Candy: back of an empty list")
        return tail["v"]
    }
    spell get(i) {
        assert_index(i, n, Off)
        return node_at(i)["v"]
    }
    spell insert_at(i, v) {
        assert_index(i, n, On)
        if i == 0 {
            push_front(v)
            return null
        }
        if i == n {
            push_back(v)
            return null
        }
        bind prev = node_at(i - 1)
        bind node = {"v": v, "next": prev["next"]}
        prev["next"] = node
        n = n + 1
        return null
    }
    spell remove_at(i) {
        assert_index(i, n, Off)
        if i == 0 {
            return pop_front()
        }
        bind prev = node_at(i - 1)
        bind node = prev["next"]
        prev["next"] = node["next"]
        if node == tail {
            tail = prev
        }
        n = n - 1
        return node["v"]
    }
    spell remove(v) {
        bind prev = null
        bind cur = head
        while cur != null {
            if cur["v"] == v {
                if prev == null {
                    head = cur["next"]
                    if head == null {
                        tail = null
                    }
                } else {
                    prev["next"] = cur["next"]
                    if cur == tail {
                        tail = prev
                    }
                }
                n = n - 1
                return On
            }
            prev = cur
            cur = cur["next"]
        }
        return Off
    }
    spell contains(v) {
        bind cur = head
        while cur != null {
            if cur["v"] == v {
                return On
            }
            cur = cur["next"]
        }
        return Off
    }
    spell index_of(v) {
        bind cur = head
        bind i = 0
        while cur != null {
            if cur["v"] == v {
                return i
            }
            cur = cur["next"]
            i = i + 1
        }
        return null
    }
    spell reverse() {
        bind old_head = head
        bind prev = null
        bind cur = head
        while cur != null {
            bind nxt = cur["next"]
            cur["next"] = prev
            prev = cur
            cur = nxt
        }
        head = prev
        if old_head != null {
            tail = old_head
        }
        return null
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell to_list() {
        bind out = []
        bind cur = head
        while cur != null {
            list_append(out, cur["v"])
            cur = cur["next"]
        }
        return out
    }
    spell clear() {
        head = null
        tail = null
        n = 0
        return null
    }
    return {"kind": "candy/singly-linked-list", "push_front": push_front,
            "push_back": push_back, "pop_front": pop_front,
            "pop_back": pop_back, "front": front, "back": back, "get": get,
            "insert_at": insert_at, "remove_at": remove_at, "remove": remove,
            "contains": contains, "index_of": index_of, "reverse": reverse,
            "size": size, "is_empty": is_empty, "to_list": to_list,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? DoublyLinkedList — nodes {"v", "prev", "next"}; pop_back is O(1)
? ---------------------------------------------------------------------------

spell doubly_linked_list() {
    bind list_append = append
    bind head = null
    bind tail = null
    bind n = 0
    spell node_at(i) {
        bind cur = head
        bind j = 0
        while j < i {
            cur = cur["next"]
            j = j + 1
        }
        return cur
    }
    spell assert_index(i, upper, allow_end) {
        assert(type_of(i) == "integer", "Candy: index must be an integer")
        if allow_end {
            assert(i >= 0 and i <= upper, "Candy: insert position out of range")
        } else {
            assert(i >= 0 and i < upper, "Candy: index out of range")
        }
    }
    spell push_front(v) {
        bind node = {"v": v, "prev": null, "next": head}
        if head != null {
            head["prev"] = node
        }
        head = node
        if tail == null {
            tail = node
        }
        n = n + 1
        return null
    }
    spell push_back(v) {
        bind node = {"v": v, "prev": tail, "next": null}
        if tail != null {
            tail["next"] = node
        } else {
            head = node
        }
        tail = node
        n = n + 1
        return null
    }
    spell pop_front() {
        assert(n > 0, "Candy: pop_front from an empty list")
        bind node = head
        head = node["next"]
        if head != null {
            head["prev"] = null
        } else {
            tail = null
        }
        n = n - 1
        return node["v"]
    }
    spell pop_back() {
        assert(n > 0, "Candy: pop_back from an empty list")
        bind node = tail
        tail = node["prev"]
        if tail != null {
            tail["next"] = null
        } else {
            head = null
        }
        n = n - 1
        return node["v"]
    }
    spell front() {
        assert(n > 0, "Candy: front of an empty list")
        return head["v"]
    }
    spell back() {
        assert(n > 0, "Candy: back of an empty list")
        return tail["v"]
    }
    spell get(i) {
        assert_index(i, n, Off)
        return node_at(i)["v"]
    }
    spell insert_at(i, v) {
        assert_index(i, n, On)
        if i == 0 {
            push_front(v)
            return null
        }
        if i == n {
            push_back(v)
            return null
        }
        bind prev = node_at(i - 1)
        bind node = {"v": v, "prev": prev, "next": prev["next"]}
        prev["next"]["prev"] = node
        prev["next"] = node
        n = n + 1
        return null
    }
    spell remove_at(i) {
        assert_index(i, n, Off)
        if i == 0 {
            return pop_front()
        }
        if i == n - 1 {
            return pop_back()
        }
        bind prev = node_at(i - 1)
        bind node = prev["next"]
        prev["next"] = node["next"]
        node["next"]["prev"] = prev
        n = n - 1
        return node["v"]
    }
    spell remove(v) {
        bind cur = head
        while cur != null {
            if cur["v"] == v {
                if cur["prev"] == null {
                    head = cur["next"]
                    if head != null {
                        head["prev"] = null
                    } else {
                        tail = null
                    }
                } else {
                    cur["prev"]["next"] = cur["next"]
                    if cur["next"] != null {
                        cur["next"]["prev"] = cur["prev"]
                    } else {
                        tail = cur["prev"]
                    }
                }
                n = n - 1
                return On
            }
            cur = cur["next"]
        }
        return Off
    }
    spell contains(v) {
        bind cur = head
        while cur != null {
            if cur["v"] == v {
                return On
            }
            cur = cur["next"]
        }
        return Off
    }
    spell index_of(v) {
        bind cur = head
        bind i = 0
        while cur != null {
            if cur["v"] == v {
                return i
            }
            cur = cur["next"]
            i = i + 1
        }
        return null
    }
    spell reverse() {
        bind cur = head
        while cur != null {
            bind nxt = cur["next"]
            cur["next"] = cur["prev"]
            cur["prev"] = nxt
            cur = nxt
        }
        bind old_head = head
        head = tail
        tail = old_head
        return null
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell to_list() {
        bind out = []
        bind cur = head
        while cur != null {
            list_append(out, cur["v"])
            cur = cur["next"]
        }
        return out
    }
    spell to_list_reversed() {
        bind out = []
        bind cur = tail
        while cur != null {
            list_append(out, cur["v"])
            cur = cur["prev"]
        }
        return out
    }
    spell clear() {
        head = null
        tail = null
        n = 0
        return null
    }
    return {"kind": "candy/doubly-linked-list", "push_front": push_front,
            "push_back": push_back, "pop_front": pop_front,
            "pop_back": pop_back, "front": front, "back": back, "get": get,
            "insert_at": insert_at, "remove_at": remove_at, "remove": remove,
            "contains": contains, "index_of": index_of, "reverse": reverse,
            "size": size, "is_empty": is_empty, "to_list": to_list,
            "to_list_reversed": to_list_reversed, "clear": clear}
}

? ---------------------------------------------------------------------------
? HashMap — a real bucketed hash table (chaining), NOT a wrapper over an
? Arcana map. Keys: text or integer. Buckets double + rehash past load 0.75.
? Insertion order is kept in a parallel keys list (rebuilt on delete).
? ---------------------------------------------------------------------------

spell hash_map() {
    bind list_append = append
    bind buckets = []
    bind bucket_count = 8
    for i in range(0, bucket_count) {
        list_append(buckets, [])
    }
    bind n = 0
    bind order = []
    spell check_key(k) {
        assert(type_of(k) == "text" or type_of(k) == "integer",
               "Candy: keys must be text or integer")
    }
    spell bucket_of(k) {
        return _hash_key(k) % bucket_count
    }
    spell find_pair(bidx, k) {
        for pair in buckets[bidx] {
            if pair[0] == k {
                return pair
            }
        }
        return null
    }
    spell rehash(new_count) {
        bind fresh = []
        for i in range(0, new_count) {
            list_append(fresh, [])
        }
        for b in buckets {
            for pair in b {
                bind idx = _hash_key(pair[0]) % new_count
                list_append(fresh[idx], pair)
            }
        }
        buckets = fresh
        bucket_count = new_count
        return null
    }
    spell set(k, v) {
        check_key(k)
        bind bidx = bucket_of(k)
        bind pair = find_pair(bidx, k)
        if pair != null {
            pair[1] = v
            return null
        }
        list_append(buckets[bidx], [k, v])
        list_append(order, k)
        n = n + 1
        if n * 4 > bucket_count * 3 {
            rehash(bucket_count * 2)
        }
        return null
    }
    spell get(k, default = null) {
        check_key(k)
        bind pair = find_pair(bucket_of(k), k)
        if pair == null {
            return default
        }
        return pair[1]
    }
    spell has(k) {
        check_key(k)
        return find_pair(bucket_of(k), k) != null
    }
    spell delete(k) {
        check_key(k)
        bind bidx = bucket_of(k)
        bind pair = find_pair(bidx, k)
        if pair == null {
            return Off
        }
        bind fresh_bucket = []
        for p in buckets[bidx] {
            if p != pair {
                list_append(fresh_bucket, p)
            }
        }
        buckets[bidx] = fresh_bucket
        bind fresh_order = []
        for k2 in order {
            if k2 != k {
                list_append(fresh_order, k2)
            }
        }
        order = fresh_order
        n = n - 1
        return On
    }
    spell size() {
        return n
    }
    spell keys() {
        return copy(order)
    }
    spell values() {
        bind out = []
        for k in order {
            list_append(out, get(k))
        }
        return out
    }
    spell items() {
        bind out = []
        for k in order {
            list_append(out, [k, get(k)])
        }
        return out
    }
    spell clear() {
        buckets = []
        for i in range(0, 8) {
            list_append(buckets, [])
        }
        bucket_count = 8
        n = 0
        order = []
        return null
    }
    return {"kind": "candy/hash-map", "set": set, "get": get, "has": has,
            "delete": delete, "size": size, "keys": keys, "values": values,
            "items": items, "clear": clear}
}

? ---------------------------------------------------------------------------
? HashSet — the same bucketing machinery, values only (they are the keys).
? to_list() makes no order promise.
? ---------------------------------------------------------------------------

spell hash_set() {
    bind list_append = append
    bind buckets = []
    bind bucket_count = 8
    for i in range(0, bucket_count) {
        list_append(buckets, [])
    }
    bind n = 0
    spell check_value(v) {
        assert(type_of(v) == "text" or type_of(v) == "integer",
               "Candy: values must be text or integer")
    }
    spell bucket_of(v) {
        return _hash_key(v) % bucket_count
    }
    spell in_bucket(bidx, v) {
        for item in buckets[bidx] {
            if item == v {
                return On
            }
        }
        return Off
    }
    spell rehash(new_count) {
        bind fresh = []
        for i in range(0, new_count) {
            list_append(fresh, [])
        }
        for b in buckets {
            for item in b {
                bind idx = _hash_key(item) % new_count
                list_append(fresh[idx], item)
            }
        }
        buckets = fresh
        bucket_count = new_count
        return null
    }
    spell add(v) {
        check_value(v)
        bind bidx = bucket_of(v)
        if in_bucket(bidx, v) {
            return Off
        }
        list_append(buckets[bidx], v)
        n = n + 1
        if n * 4 > bucket_count * 3 {
            rehash(bucket_count * 2)
        }
        return On
    }
    spell has(v) {
        check_value(v)
        return in_bucket(bucket_of(v), v)
    }
    spell delete(v) {
        check_value(v)
        bind bidx = bucket_of(v)
        if not in_bucket(bidx, v) {
            return Off
        }
        bind fresh = []
        for item in buckets[bidx] {
            if item != v {
                list_append(fresh, item)
            }
        }
        buckets[bidx] = fresh
        n = n - 1
        return On
    }
    spell size() {
        return n
    }
    spell to_list() {
        bind out = []
        for b in buckets {
            for item in b {
                list_append(out, item)
            }
        }
        return out
    }
    spell clear() {
        buckets = []
        for i in range(0, 8) {
            list_append(buckets, [])
        }
        bucket_count = 8
        n = 0
        return null
    }
    return {"kind": "candy/hash-set", "add": add, "has": has, "delete": delete,
            "size": size, "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? OrderedMap — a thin, documented wrapper over Arcana's insertion-ordered
? map. Cheaper than HashMap when key counts are small; delete is O(size)
? because Arcana maps have no in-place deletion.
? ---------------------------------------------------------------------------

spell ordered_map() {
    bind map_keys = keys
    bind map_values = values
    bind m = {}
    spell check_key(k) {
        assert(type_of(k) == "text" or type_of(k) == "integer",
               "Candy: keys must be text or integer")
    }
    spell set(k, v) {
        check_key(k)
        m[k] = v
        return null
    }
    spell get(k, default = null) {
        check_key(k)
        if k in m {
            return m[k]
        }
        return default
    }
    spell has(k) {
        check_key(k)
        return k in m
    }
    spell delete(k) {
        check_key(k)
        if not (k in m) {
            return Off
        }
        bind fresh = {}
        for k2 in m {
            if k2 != k {
                fresh[k2] = m[k2]
            }
        }
        m = fresh
        return On
    }
    spell keys() {
        return map_keys(m)
    }
    spell values() {
        return map_values(m)
    }
    spell items() {
        bind out = []
        for k in m {
            append(out, [k, m[k]])
        }
        return out
    }
    spell size() {
        return length(m)
    }
    spell clear() {
        m = {}
        return null
    }
    return {"kind": "candy/ordered-map", "set": set, "get": get, "has": has,
            "delete": delete, "size": size, "keys": keys, "values": values,
            "items": items, "clear": clear}
}

? ---------------------------------------------------------------------------
? OrderedSet — insertion-ordered set over an Arcana map
? ---------------------------------------------------------------------------

spell ordered_set() {
    bind map_keys = keys
    bind m = {}
    spell check_value(v) {
        assert(type_of(v) == "text" or type_of(v) == "integer",
               "Candy: values must be text or integer")
    }
    spell add(v) {
        check_value(v)
        if v in m {
            return Off
        }
        m[v] = On
        return On
    }
    spell has(v) {
        check_value(v)
        return v in m
    }
    spell delete(v) {
        check_value(v)
        if not (v in m) {
            return Off
        }
        bind fresh = {}
        for v2 in m {
            if v2 != v {
                fresh[v2] = On
            }
        }
        m = fresh
        return On
    }
    spell to_list() {
        return map_keys(m)
    }
    spell size() {
        return length(m)
    }
    spell clear() {
        m = {}
        return null
    }
    return {"kind": "candy/ordered-set", "add": add, "has": has,
            "delete": delete, "to_list": to_list, "size": size, "clear": clear}
}

? ---------------------------------------------------------------------------
? DSU — disjoint set union over the integer elements 0..n-1. Union by size
? with full path compression.
? ---------------------------------------------------------------------------

spell dsu(n) {
    assert(type_of(n) == "integer", "Candy: n must be an integer")
    assert(n >= 0, "Candy: n must be at least 0")
    bind parent = []
    bind sizes = []
    for i in range(0, n) {
        append(parent, i)
        append(sizes, 1)
    }
    bind comp_count = n
    spell check(x) {
        assert(type_of(x) == "integer", "Candy: elements must be integers")
        assert(x >= 0 and x < n, "Candy: element out of range")
    }
    spell find(x) {
        check(x)
        bind root = x
        while parent[root] != root {
            root = parent[root]
        }
        bind cur = x
        while cur != root {
            bind nxt = parent[cur]
            parent[cur] = root
            cur = nxt
        }
        return root
    }
    spell union(x, y) {
        check(x)
        check(y)
        bind rx = find(x)
        bind ry = find(y)
        if rx == ry {
            return Off
        }
        if sizes[rx] < sizes[ry] {
            bind t = rx
            rx = ry
            ry = t
        }
        parent[ry] = rx
        sizes[rx] = sizes[rx] + sizes[ry]
        comp_count = comp_count - 1
        return On
    }
    spell connected(x, y) {
        return find(x) == find(y)
    }
    ? Arcana reserves the name `n`, so the method ships under the map
    ? key "count" with an inner spell of a different name
    spell count_components() {
        return comp_count
    }
    spell component_size(x) {
        return sizes[find(x)]
    }
    spell size() {
        return n
    }
    spell components() {
        ? groups come out ascending (members appended in i order) and the
        ? map's insertion order is the order of each group's first member,
        ? i.e. of its smallest member
        bind groups = {}
        for i in range(0, n) {
            bind r = find(i)
            if r in groups {
                append(groups[r], i)
            } else {
                groups[r] = [i]
            }
        }
        return values(groups)
    }
    return {"kind": "candy/dsu", "find": find, "union": union,
            "connected": connected, "count": count_components,
            "component_size": component_size, "size": size,
            "components": components}
}

? ---------------------------------------------------------------------------
? BloomFilter — probabilistic membership with double hashing:
? h_i = (h1 + i*h2 + i*i) mod m. False positives are possible, false
? negatives never are. Sized up front for a target false-positive rate.
? ---------------------------------------------------------------------------

spell bloom_filter(expected = 1000, rate = 0.01) {
    assert(type_of(expected) == "integer", "Candy: expected must be an integer")
    assert(expected >= 1, "Candy: expected must be at least 1")
    assert(type_of(rate) == "decimal" or type_of(rate) == "integer",
           "Candy: rate must be a number")
    assert(rate > 0 and rate < 1, "Candy: rate must be strictly between 0 and 1")
    bind m = ceil(-expected * log(rate) / (log(2) * log(2)))
    bind k = max(1, round(m / expected * log(2)))
    bind bits = []
    for i in range(0, m) {
        append(bits, 0)
    }
    bind adds_count = 0
    spell check_item(item) {
        assert(type_of(item) == "text" or type_of(item) == "integer",
               "Candy: bloom filter items must be text or integer")
    }
    spell positions(item) {
        bind h1 = _hash_key(item, 7919)
        bind h2 = _hash_key(item, 104729)
        bind p1 = h1 % m
        bind p2 = h2 % m
        bind out = []
        for i in range(0, k) {
            append(out, (p1 + i * p2 + i * i) % m)
        }
        return out
    }
    spell add(item) {
        check_item(item)
        for pos in positions(item) {
            bits[pos] = 1
        }
        adds_count = adds_count + 1
        return null
    }
    spell might_contain(item) {
        check_item(item)
        for pos in positions(item) {
            if bits[pos] == 0 {
                return Off
            }
        }
        return On
    }
    spell bit_count() {
        bind set_bits = 0
        for b in bits {
            if b == 1 {
                set_bits = set_bits + 1
            }
        }
        return set_bits
    }
    spell capacity() {
        return m
    }
    spell hash_count() {
        return k
    }
    spell estimated_fp() {
        bind filled = (1 - 1 / m) ** (k * adds_count)
        return 1 - filled
    }
    spell adds() {
        return adds_count
    }
    spell clear() {
        for i in range(0, m) {
            bits[i] = 0
        }
        adds_count = 0
        return null
    }
    return {"kind": "candy/bloom-filter", "add": add,
            "might_contain": might_contain, "bit_count": bit_count,
            "capacity": capacity, "hash_count": hash_count,
            "estimated_fp": estimated_fp, "adds": adds, "clear": clear}
}

? ---------------------------------------------------------------------------
? LRU cache — TRUE O(1): a slot array of [key, value, prev_idx, next_idx]
? nodes forms a doubly linked recency chain (head = MRU, tail = LRU), a free
? list recycles slots, and an Arcana map key -> slot gives O(1) lookup.
? Arcana maps cannot delete a key in place, so evicted/deleted keys leave a
? stale map entry; slot_of() detects staleness (freed slot, or slot reused
? by another key) and the map is compacted (rebuilt from the live chain)
? once stale entries dominate. Compaction is amortised O(1).
? ---------------------------------------------------------------------------

spell lru_cache(cap) {
    assert(type_of(cap) == "integer", "Candy: capacity must be an integer")
    assert(cap >= 1, "Candy: capacity must be at least 1")
    bind list_append = append
    bind list_pop = pop
    bind nodes = []
    bind free = []
    bind index_map = {}
    bind head = -1
    bind tail = -1
    bind n = 0
    spell slot_of(k) {
        if not (k in index_map) {
            return -1
        }
        bind s = index_map[k]
        if nodes[s] == null {
            return -1
        }
        if nodes[s][0] != k {
            return -1
        }
        return s
    }
    spell compact() {
        if length(index_map) > n * 2 + 8 {
            bind fresh = {}
            bind s = head
            while s != -1 {
                fresh[nodes[s][0]] = s
                s = nodes[s][3]
            }
            index_map = fresh
        }
        return null
    }
    spell detach(s) {
        bind p = nodes[s][2]
        bind n = nodes[s][3]
        if p == -1 {
            head = n
        } else {
            nodes[p][3] = n
        }
        if n == -1 {
            tail = p
        } else {
            nodes[n][2] = p
        }
        nodes[s][2] = -1
        nodes[s][3] = -1
        return null
    }
    spell push_front(s) {
        nodes[s][2] = -1
        nodes[s][3] = head
        if head != -1 {
            nodes[head][2] = s
        }
        head = s
        if tail == -1 {
            tail = s
        }
        return null
    }
    spell touch(s) {
        if head != s {
            detach(s)
            push_front(s)
        }
        return null
    }
    spell evict_lru() {
        bind victim = tail
        detach(victim)
        nodes[victim] = null
        list_append(free, victim)
        n = n - 1
        compact()
        return null
    }
    spell get(k, default = null) {
        bind s = slot_of(k)
        if s == -1 {
            return default
        }
        touch(s)
        return nodes[s][1]
    }
    spell put(k, v) {
        bind s = slot_of(k)
        if s != -1 {
            nodes[s][1] = v
            touch(s)
            return null
        }
        if n == cap {
            evict_lru()
        }
        bind slot = -1
        if length(free) > 0 {
            slot = list_pop(free)
        } else {
            slot = length(nodes)
            list_append(nodes, null)
        }
        nodes[slot] = [k, v, -1, -1]
        push_front(slot)
        index_map[k] = slot
        n = n + 1
        compact()
        return null
    }
    spell has(k) {
        return slot_of(k) != -1
    }
    spell delete(k) {
        bind s = slot_of(k)
        if s == -1 {
            return Off
        }
        detach(s)
        nodes[s] = null
        list_append(free, s)
        n = n - 1
        compact()
        return On
    }
    spell size() {
        return n
    }
    spell capacity() {
        return cap
    }
    spell keys() {
        bind out = []
        bind s = head
        while s != -1 {
            list_append(out, nodes[s][0])
            s = nodes[s][3]
        }
        return out
    }
    spell to_list() {
        bind out = []
        bind s = head
        while s != -1 {
            list_append(out, [nodes[s][0], nodes[s][1]])
            s = nodes[s][3]
        }
        return out
    }
    spell clear() {
        nodes = []
        free = []
        index_map = {}
        head = -1
        tail = -1
        n = 0
        return null
    }
    return {"kind": "candy/lru-cache", "get": get, "put": put, "has": has,
            "delete": delete, "size": size, "capacity": capacity,
            "keys": keys, "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? LFU cache — same slot-array storage as the LRU cache, plus a frequency
? and a last-use stamp (from one global counter) per node. Eviction scans
? all live slots for the lowest frequency, ties broken by the oldest stamp,
? so get/put are O(capacity) worst case — the price of exact LFU without
? per-frequency bucket lists.
? ---------------------------------------------------------------------------

spell lfu_cache(cap) {
    assert(type_of(cap) == "integer", "Candy: capacity must be an integer")
    assert(cap >= 1, "Candy: capacity must be at least 1")
    bind list_append = append
    bind list_pop = pop
    bind nodes = []
    bind free = []
    bind index_map = {}
    bind head = -1
    bind tail = -1
    bind n = 0
    bind stamp_seq = 0
    spell slot_of(k) {
        if not (k in index_map) {
            return -1
        }
        bind s = index_map[k]
        if nodes[s] == null {
            return -1
        }
        if nodes[s][0] != k {
            return -1
        }
        return s
    }
    spell compact() {
        if length(index_map) > n * 2 + 8 {
            bind fresh = {}
            bind s = head
            while s != -1 {
                fresh[nodes[s][0]] = s
                s = nodes[s][3]
            }
            index_map = fresh
        }
        return null
    }
    spell detach(s) {
        bind p = nodes[s][2]
        bind n = nodes[s][3]
        if p == -1 {
            head = n
        } else {
            nodes[p][3] = n
        }
        if n == -1 {
            tail = p
        } else {
            nodes[n][2] = p
        }
        nodes[s][2] = -1
        nodes[s][3] = -1
        return null
    }
    spell push_front(s) {
        nodes[s][2] = -1
        nodes[s][3] = head
        if head != -1 {
            nodes[head][2] = s
        }
        head = s
        if tail == -1 {
            tail = s
        }
        return null
    }
    spell touch(s) {
        nodes[s][4] = nodes[s][4] + 1
        stamp_seq = stamp_seq + 1
        nodes[s][5] = stamp_seq
        if head != s {
            detach(s)
            push_front(s)
        }
        return null
    }
    spell evict_lfu() {
        bind s = head
        bind best = -1
        while s != -1 {
            if best == -1 {
                best = s
            } else {
                if nodes[s][4] < nodes[best][4] {
                    best = s
                } else {
                    if nodes[s][4] == nodes[best][4] and nodes[s][5] < nodes[best][5] {
                        best = s
                    }
                }
            }
            s = nodes[s][3]
        }
        detach(best)
        nodes[best] = null
        list_append(free, best)
        n = n - 1
        compact()
        return null
    }
    spell get(k, default = null) {
        bind s = slot_of(k)
        if s == -1 {
            return default
        }
        touch(s)
        return nodes[s][1]
    }
    spell put(k, v) {
        bind s = slot_of(k)
        if s != -1 {
            nodes[s][1] = v
            touch(s)
            return null
        }
        if n == cap {
            evict_lfu()
        }
        bind slot = -1
        if length(free) > 0 {
            slot = list_pop(free)
        } else {
            slot = length(nodes)
            list_append(nodes, null)
        }
        stamp_seq = stamp_seq + 1
        nodes[slot] = [k, v, -1, -1, 1, stamp_seq]
        push_front(slot)
        index_map[k] = slot
        n = n + 1
        compact()
        return null
    }
    spell has(k) {
        return slot_of(k) != -1
    }
    spell delete(k) {
        bind s = slot_of(k)
        if s == -1 {
            return Off
        }
        detach(s)
        nodes[s] = null
        list_append(free, s)
        n = n - 1
        compact()
        return On
    }
    spell size() {
        return n
    }
    spell capacity() {
        return cap
    }
    spell keys() {
        bind out = []
        bind s = head
        while s != -1 {
            list_append(out, nodes[s][0])
            s = nodes[s][3]
        }
        return out
    }
    spell to_list() {
        bind out = []
        bind s = head
        while s != -1 {
            list_append(out, [nodes[s][0], nodes[s][1]])
            s = nodes[s][3]
        }
        return out
    }
    spell clear() {
        nodes = []
        free = []
        index_map = {}
        head = -1
        tail = -1
        n = 0
        stamp_seq = 0
        return null
    }
    return {"kind": "candy/lfu-cache", "get": get, "put": put, "has": has,
            "delete": delete, "size": size, "capacity": capacity,
            "keys": keys, "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? Skip list — probabilistic sorted map over comparable keys (all numbers or
? all texts). Nodes are {"key", "val", "forward"} with one forward pointer
? per level; the head sentinel has max_level of them. A node's level is
? 1 + the number of consecutive random() < p draws, capped at max_level.
? ---------------------------------------------------------------------------

spell skip_list(max_level = 16, p = 0.5) {
    bind int_range = range
    assert(type_of(max_level) == "integer", "Candy: max_level must be an integer")
    assert(max_level >= 1 and max_level <= 32,
           "Candy: max_level must be between 1 and 32")
    assert(type_of(p) == "decimal" or type_of(p) == "integer",
           "Candy: p must be a number")
    assert(p > 0 and p < 1, "Candy: p must be strictly between 0 and 1")
    bind head = {"key": null, "val": null, "forward": []}
    for i in int_range(0, max_level) {
        append(head["forward"], null)
    }
    bind level = 1
    bind n = 0
    bind key_kind = null
    spell check_key(k) {
        bind t = type_of(k)
        if t == "integer" or t == "decimal" {
            if key_kind == null {
                key_kind = "number"
            } else {
                assert(key_kind == "number",
                       "Candy: keys must be all numbers or all texts")
            }
            return null
        }
        if t == "text" {
            if key_kind == null {
                key_kind = "text"
            } else {
                assert(key_kind == "text",
                       "Candy: keys must be all numbers or all texts")
            }
            return null
        }
        assert(Off, "Candy: keys must be all numbers or all texts")
        return null
    }
    spell random_level() {
        bind lvl = 1
        while lvl < max_level and random() < p {
            lvl = lvl + 1
        }
        return lvl
    }
    ? walk returns the update path: for every level, the rightmost node
    ? whose key is < k (the head itself when none)
    spell walk(k) {
        bind update = []
        for i in int_range(0, max_level) {
            append(update, head)
        }
        bind node = head
        bind i = level - 1
        while i >= 0 {
            bind nxt = node["forward"][i]
            while nxt != null and nxt["key"] < k {
                node = nxt
                nxt = node["forward"][i]
            }
            update[i] = node
            i = i - 1
        }
        return update
    }
    spell find_node(k) {
        bind node = head
        bind i = level - 1
        while i >= 0 {
            bind nxt = node["forward"][i]
            while nxt != null and nxt["key"] < k {
                node = nxt
                nxt = node["forward"][i]
            }
            i = i - 1
        }
        return node["forward"][0]
    }
    spell link(update, node, lvl) {
        for i in int_range(0, lvl) {
            node["forward"][i] = update[i]["forward"][i]
            update[i]["forward"][i] = node
        }
        return null
    }
    spell add(key, value = null) {
        check_key(key)
        bind update = walk(key)
        bind cand = update[0]["forward"][0]
        if cand != null and cand["key"] == key {
            return Off
        }
        bind lvl = random_level()
        if lvl > level {
            for i in int_range(level, lvl) {
                update[i] = head
            }
            level = lvl
        }
        bind node = {"key": key, "val": value, "forward": []}
        for i in int_range(0, lvl) {
            append(node["forward"], null)
        }
        link(update, node, lvl)
        n = n + 1
        return On
    }
    spell set(key, value) {
        check_key(key)
        bind node = find_node(key)
        if node != null and node["key"] == key {
            node["val"] = value
            return null
        }
        add(key, value)
        return null
    }
    spell get(key, default = null) {
        check_key(key)
        bind node = find_node(key)
        if node != null and node["key"] == key {
            return node["val"]
        }
        return default
    }
    spell has(key) {
        check_key(key)
        bind node = find_node(key)
        return node != null and node["key"] == key
    }
    spell delete(key) {
        check_key(key)
        bind update = walk(key)
        bind cand = update[0]["forward"][0]
        if cand == null or cand["key"] != key {
            return Off
        }
        for i in int_range(0, level) {
            if update[i]["forward"][i] == cand {
                update[i]["forward"][i] = cand["forward"][i]
            }
        }
        while level > 1 and head["forward"][level - 1] == null {
            level = level - 1
        }
        n = n - 1
        return On
    }
    spell size() {
        return n
    }
    spell first() {
        assert(n > 0, "Candy: first of an empty skip list")
        return head["forward"][0]["key"]
    }
    spell last() {
        assert(n > 0, "Candy: last of an empty skip list")
        bind node = head
        bind i = level - 1
        while i >= 0 {
            while node["forward"][i] != null {
                node = node["forward"][i]
            }
            i = i - 1
        }
        return node["key"]
    }
    spell range(start, end) {
        check_key(start)
        check_key(end)
        bind node = head
        bind i = level - 1
        while i >= 0 {
            bind nxt = node["forward"][i]
            while nxt != null and nxt["key"] < start {
                node = nxt
                nxt = node["forward"][i]
            }
            i = i - 1
        }
        bind out = []
        bind cur = node["forward"][0]
        while cur != null and cur["key"] < end {
            append(out, cur["key"])
            cur = cur["forward"][0]
        }
        return out
    }
    spell to_list() {
        bind out = []
        bind cur = head["forward"][0]
        while cur != null {
            append(out, cur["key"])
            cur = cur["forward"][0]
        }
        return out
    }
    spell items() {
        bind out = []
        bind cur = head["forward"][0]
        while cur != null {
            append(out, [cur["key"], cur["val"]])
            cur = cur["forward"][0]
        }
        return out
    }
    spell clear() {
        head = {"key": null, "val": null, "forward": []}
        for i in int_range(0, max_level) {
            append(head["forward"], null)
        }
        level = 1
        n = 0
        key_kind = null
        return null
    }
    return {"kind": "candy/skip-list", "add": add, "set": set, "get": get,
            "has": has, "delete": delete, "size": size, "first": first,
            "last": last, "range": range, "to_list": to_list,
            "items": items, "clear": clear}
}
