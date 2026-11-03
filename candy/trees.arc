? Candy 02: trees — tree-shaped data structures for Arcana.
?
? Part of the Candy library (Tier I, module 02 of 04).
? Import from your grimoire with:
?
?     import candy.trees                 ? when the candy/ folder sits next to
?                                        ? your file, or is on ARCANA_PATH
?     import trees                       ? when this file sits next to yours
?
? Trees in this module: BinaryTree, BST, AVL, RedBlackTree, BinaryHeap,
? Trie, SegmentTree, FenwickTree, BTree, BPlusTree, SuffixArray, Treap.
?
? Design: Arcana has no user-defined types, so every Candy structure is a
? *closure object*: `bst()` declares private state, inner spells that
? touch it, and returns a map of those spells. Call methods with indexing:
?
?     bind t = bst()
?     t["insert"](7)
?     cast t["to_list"]()                ? [7]
?
? Every instance carries a "kind" text ("candy/bst", ...) so programs can
? tell structures apart, plus size() and clear() wherever they make sense.
? Misuse (min of an empty tree, a bad query range, ...) raises a catchable
? AssertionError through assert(), with a message starting "Candy: ".
?
? Ordered keys: BST, AVL, RedBlackTree, BinaryHeap, BTree, BPlusTree and
? Treap order their keys, so the keys must be all numbers or all texts. The
? first key that breaks this rule raises AssertionError "Candy: keys must
? be all numbers or all texts".
?
? Recursion: Arcana caps call depth at 1000, so every operation that can
? walk a long chain (plain BST insert/remove/contains/traversals, heap
? sifts, trie walks, suffix-array searches, segment/fenwick loops) is
? written with explicit loops and stacks. Only depth-O(log n) recursion is
? used (AVL/RB/treap rebalancing, B-tree descent).

from math import random, random_seed     ? treap priorities only

? ---------------------------------------------------------------------------
? Shared private helpers (used by several constructors below)
? ---------------------------------------------------------------------------

spell _key_kind(v) {
    bind t = type_of(v)
    if t == "integer" or t == "decimal" {
        return "number"
    }
    if t == "text" {
        return "text"
    }
    assert(Off, "Candy: keys must be all numbers or all texts")
    return null
}

spell _list_insert(xs, at, v) {
    append(xs, null)
    bind i = length(xs) - 1
    while i > at {
        xs[i] = xs[i - 1]
        i = i - 1
    }
    xs[at] = v
    return null
}

spell _remove_at(xs, at) {
    bind i = at
    while i < length(xs) - 1 {
        xs[i] = xs[i + 1]
        i = i + 1
    }
    pop(xs)
    return null
}

spell _truncate(xs, keep) {
    while length(xs) > keep {
        pop(xs)
    }
    return null
}

spell _reversed(xs) {
    bind out = []
    bind i = length(xs) - 1
    while i >= 0 {
        append(out, xs[i])
        i = i - 1
    }
    return out
}

spell _sort_texts(xs) {
    ? in-place insertion sort; Arcana's sorted() refuses nothing fancy but
    ? lives in the list module, so Candy sorts small result lists by hand
    for i in range(1, length(xs)) {
        bind cur = xs[i]
        bind j = i - 1
        while j >= 0 and xs[j] > cur {
            xs[j + 1] = xs[j]
            j = j - 1
        }
        xs[j + 1] = cur
    }
    return null
}

spell _trie_node() {
    return {"end": Off, "next": {}}
}

spell _bnode(leaf) {
    return {"keys": [], "kids": [], "leaf": leaf}
}

? ---------------------------------------------------------------------------
? BinaryTree — a complete binary tree filled in level order
? ---------------------------------------------------------------------------

? Positional, not a search tree: insert(v) appends at the next level-order
? slot, exactly like filling an array-backed heap left to right. Children
? of slot i live at 2i+1 and 2i+2. There is no remove: the shape is the
? contract. All traversals are iterative over the implicit array.

spell binary_tree() {
    bind slots = []
    bind n = 0
    bind list_pop = pop
    spell insert(v) {
        append(slots, v)
        n = n + 1
        return null
    }
    spell contains(v) {
        return v in slots
    }
    spell size() {
        return n
    }
    spell height() {
        ? empty = 0, single node = 1; smallest h with n <= 2^h - 1
        bind h = 0
        bind filled = 0
        while filled < n {
            filled = filled * 2 + 1
            h = h + 1
        }
        return h
    }
    spell is_empty() {
        return n == 0
    }
    spell to_list() {
        return copy(slots)
    }
    spell level_order() {
        return copy(slots)
    }
    spell inorder() {
        bind out = []
        bind stack = []
        bind i = 0
        bind going = On
        while going {
            while i < n {
                append(stack, i)
                i = i * 2 + 1
            }
            if length(stack) == 0 {
                going = Off
            } else {
                i = list_pop(stack)
                append(out, slots[i])
                i = i * 2 + 2
            }
        }
        return out
    }
    spell preorder() {
        bind out = []
        if n == 0 {
            return out
        }
        bind stack = [0]
        while length(stack) > 0 {
            bind i = list_pop(stack)
            append(out, slots[i])
            if i * 2 + 2 < n {
                append(stack, i * 2 + 2)
            }
            if i * 2 + 1 < n {
                append(stack, i * 2 + 1)
            }
        }
        return out
    }
    spell postorder() {
        bind out = []
        if n == 0 {
            return out
        }
        bind stack = [0]
        bind rev = []
        while length(stack) > 0 {
            bind i = list_pop(stack)
            append(rev, slots[i])
            if i * 2 + 1 < n {
                append(stack, i * 2 + 1)
            }
            if i * 2 + 2 < n {
                append(stack, i * 2 + 2)
            }
        }
        return _reversed(rev)
    }
    spell clear() {
        slots = []
        n = 0
        return null
    }
    return {"kind": "candy/binary-tree", "insert": insert, "contains": contains,
            "size": size, "height": height, "is_empty": is_empty,
            "to_list": to_list, "level_order": level_order, "inorder": inorder,
            "preorder": preorder, "postorder": postorder, "clear": clear}
}

? ---------------------------------------------------------------------------
? BST — unbalanced binary search tree (all methods iterative)
? ---------------------------------------------------------------------------

? Node layout: {"v", "left", "right"} or null. insert/remove/contains walk
? the tree with a loop, so even a degenerate chain of thousands of nodes
? never hits Arcana's recursion limit. remove() uses the inorder successor
? for the two-child case.

spell bst() {
    bind root = null
    bind n = 0
    bind key_type = null
    bind list_pop = pop
    spell check_key(v) {
        bind kind = _key_kind(v)
        if key_type == null {
            key_type = kind
            return null
        }
        if kind != key_type {
            assert(Off, "Candy: keys must be all numbers or all texts")
        }
        return null
    }
    spell guard_key(v) {
        if key_type != null {
            check_key(v)
        }
        return null
    }
    spell insert(v) {
        check_key(v)
        if root == null {
            root = {"v": v, "left": null, "right": null}
            n = n + 1
            return On
        }
        bind cur = root
        bind going = On
        while going {
            if v == cur["v"] {
                return Off
            }
            if v < cur["v"] {
                if cur["left"] == null {
                    cur["left"] = {"v": v, "left": null, "right": null}
                    going = Off
                } else {
                    cur = cur["left"]
                }
            } else {
                if cur["right"] == null {
                    cur["right"] = {"v": v, "left": null, "right": null}
                    going = Off
                } else {
                    cur = cur["right"]
                }
            }
        }
        n = n + 1
        return On
    }
    spell contains(v) {
        guard_key(v)
        bind cur = root
        while cur != null {
            if v == cur["v"] {
                return On
            }
            if v < cur["v"] {
                cur = cur["left"]
            } else {
                cur = cur["right"]
            }
        }
        return Off
    }
    spell remove(v) {
        guard_key(v)
        bind parent = null
        bind cur = root
        while cur != null and cur["v"] != v {
            parent = cur
            if v < cur["v"] {
                cur = cur["left"]
            } else {
                cur = cur["right"]
            }
        }
        if cur == null {
            return Off
        }
        if cur["left"] != null and cur["right"] != null {
            ? two children: swap in the inorder successor, delete it instead
            bind sparent = cur
            bind s = cur["right"]
            while s["left"] != null {
                sparent = s
                s = s["left"]
            }
            cur["v"] = s["v"]
            parent = sparent
            cur = s
        }
        bind child = cur["left"]
        if child == null {
            child = cur["right"]
        }
        if parent == null {
            root = child
        } else {
            if parent["left"] == cur {
                parent["left"] = child
            } else {
                parent["right"] = child
            }
        }
        n = n - 1
        return On
    }
    spell min() {
        assert(root != null, "Candy: min of an empty tree")
        bind cur = root
        while cur["left"] != null {
            cur = cur["left"]
        }
        return cur["v"]
    }
    spell max() {
        assert(root != null, "Candy: max of an empty tree")
        bind cur = root
        while cur["right"] != null {
            cur = cur["right"]
        }
        return cur["v"]
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell height() {
        if root == null {
            return 0
        }
        bind best = 0
        bind stack = [[root, 1]]
        while length(stack) > 0 {
            bind top = list_pop(stack)
            if top[1] > best {
                best = top[1]
            }
            if top[0]["left"] != null {
                append(stack, [top[0]["left"], top[1] + 1])
            }
            if top[0]["right"] != null {
                append(stack, [top[0]["right"], top[1] + 1])
            }
        }
        return best
    }
    spell inorder() {
        bind out = []
        bind stack = []
        bind cur = root
        bind going = On
        while going {
            while cur != null {
                append(stack, cur)
                cur = cur["left"]
            }
            if length(stack) == 0 {
                going = Off
            } else {
                cur = list_pop(stack)
                append(out, cur["v"])
                cur = cur["right"]
            }
        }
        return out
    }
    spell preorder() {
        bind out = []
        if root == null {
            return out
        }
        bind stack = [root]
        while length(stack) > 0 {
            bind node = list_pop(stack)
            append(out, node["v"])
            if node["right"] != null {
                append(stack, node["right"])
            }
            if node["left"] != null {
                append(stack, node["left"])
            }
        }
        return out
    }
    spell postorder() {
        bind out = []
        if root == null {
            return out
        }
        bind stack = [root]
        bind rev = []
        while length(stack) > 0 {
            bind node = list_pop(stack)
            append(rev, node["v"])
            if node["left"] != null {
                append(stack, node["left"])
            }
            if node["right"] != null {
                append(stack, node["right"])
            }
        }
        return _reversed(rev)
    }
    spell to_list() {
        return inorder()
    }
    spell clear() {
        root = null
        n = 0
        key_type = null
        return null
    }
    return {"kind": "candy/bst", "insert": insert, "remove": remove,
            "contains": contains, "min": min, "max": max, "size": size,
            "is_empty": is_empty, "height": height, "inorder": inorder,
            "preorder": preorder, "postorder": postorder, "to_list": to_list,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? AVL — self-balancing binary search tree
? ---------------------------------------------------------------------------

? Node layout: {"v", "left", "right", "h"} with h = subtree height (leaf is
? 1). Insert and remove recurse, which is safe: the height is O(log n), so
? the recursion depth for a million keys is about 30.

spell avl() {
    bind root = null
    bind n = 0
    bind key_type = null
    bind added = Off
    bind found = Off
    bind list_pop = pop
    spell check_key(v) {
        bind kind = _key_kind(v)
        if key_type == null {
            key_type = kind
            return null
        }
        if kind != key_type {
            assert(Off, "Candy: keys must be all numbers or all texts")
        }
        return null
    }
    spell guard_key(v) {
        if key_type != null {
            check_key(v)
        }
        return null
    }
    spell node_height(node) {
        if node == null {
            return 0
        }
        return node["h"]
    }
    spell fix(node) {
        bind lh = node_height(node["left"])
        bind rh = node_height(node["right"])
        if lh > rh {
            node["h"] = lh + 1
        } else {
            node["h"] = rh + 1
        }
        return null
    }
    spell balance_of(node) {
        return node_height(node["left"]) - node_height(node["right"])
    }
    spell rotate_right(y) {
        bind x = y["left"]
        y["left"] = x["right"]
        x["right"] = y
        fix(y)
        fix(x)
        return x
    }
    spell rotate_left(x) {
        bind y = x["right"]
        x["right"] = y["left"]
        y["left"] = x
        fix(x)
        fix(y)
        return y
    }
    spell rebalance(node) {
        fix(node)
        bind b = balance_of(node)
        if b > 1 {
            if balance_of(node["left"]) < 0 {
                node["left"] = rotate_left(node["left"])
            }
            return rotate_right(node)
        }
        if b < -1 {
            if balance_of(node["right"]) > 0 {
                node["right"] = rotate_right(node["right"])
            }
            return rotate_left(node)
        }
        return node
    }
    spell ins(node, v) {
        if node == null {
            n = n + 1
            added = On
            return {"v": v, "left": null, "right": null, "h": 1}
        }
        if v == node["v"] {
            return node
        }
        if v < node["v"] {
            node["left"] = ins(node["left"], v)
        } else {
            node["right"] = ins(node["right"], v)
        }
        return rebalance(node)
    }
    spell rm(node, v) {
        if node == null {
            return null
        }
        if v < node["v"] {
            node["left"] = rm(node["left"], v)
            return rebalance(node)
        }
        if v > node["v"] {
            node["right"] = rm(node["right"], v)
            return rebalance(node)
        }
        found = On
        if node["left"] == null {
            n = n - 1
            return node["right"]
        }
        if node["right"] == null {
            n = n - 1
            return node["left"]
        }
        bind s = node["right"]
        while s["left"] != null {
            s = s["left"]
        }
        node["v"] = s["v"]
        node["right"] = rm(node["right"], s["v"])
        return rebalance(node)
    }
    spell insert(v) {
        check_key(v)
        added = Off
        root = ins(root, v)
        return added
    }
    spell remove(v) {
        guard_key(v)
        found = Off
        root = rm(root, v)
        return found
    }
    spell contains(v) {
        guard_key(v)
        bind cur = root
        while cur != null {
            if v == cur["v"] {
                return On
            }
            if v < cur["v"] {
                cur = cur["left"]
            } else {
                cur = cur["right"]
            }
        }
        return Off
    }
    spell min() {
        assert(root != null, "Candy: min of an empty tree")
        bind cur = root
        while cur["left"] != null {
            cur = cur["left"]
        }
        return cur["v"]
    }
    spell max() {
        assert(root != null, "Candy: max of an empty tree")
        bind cur = root
        while cur["right"] != null {
            cur = cur["right"]
        }
        return cur["v"]
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell height() {
        return node_height(root)
    }
    spell check_balanced(node) {
        if node == null {
            return On
        }
        bind lh = node_height(node["left"])
        bind rh = node_height(node["right"])
        if lh - rh > 1 or rh - lh > 1 {
            return Off
        }
        bind want = lh
        if rh > lh {
            want = rh
        }
        if node["h"] != want + 1 {
            return Off
        }
        if not check_balanced(node["left"]) {
            return Off
        }
        return check_balanced(node["right"])
    }
    spell is_balanced() {
        return check_balanced(root)
    }
    spell inorder() {
        bind out = []
        bind stack = []
        bind cur = root
        bind going = On
        while going {
            while cur != null {
                append(stack, cur)
                cur = cur["left"]
            }
            if length(stack) == 0 {
                going = Off
            } else {
                cur = list_pop(stack)
                append(out, cur["v"])
                cur = cur["right"]
            }
        }
        return out
    }
    spell preorder() {
        bind out = []
        if root == null {
            return out
        }
        bind stack = [root]
        while length(stack) > 0 {
            bind node = list_pop(stack)
            append(out, node["v"])
            if node["right"] != null {
                append(stack, node["right"])
            }
            if node["left"] != null {
                append(stack, node["left"])
            }
        }
        return out
    }
    spell postorder() {
        bind out = []
        if root == null {
            return out
        }
        bind stack = [root]
        bind rev = []
        while length(stack) > 0 {
            bind node = list_pop(stack)
            append(rev, node["v"])
            if node["left"] != null {
                append(stack, node["left"])
            }
            if node["right"] != null {
                append(stack, node["right"])
            }
        }
        return _reversed(rev)
    }
    spell to_list() {
        return inorder()
    }
    spell clear() {
        root = null
        n = 0
        key_type = null
        return null
    }
    return {"kind": "candy/avl", "insert": insert, "remove": remove,
            "contains": contains, "min": min, "max": max, "size": size,
            "is_empty": is_empty, "height": height, "is_balanced": is_balanced,
            "inorder": inorder, "preorder": preorder, "postorder": postorder,
            "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? RedBlackTree — left-leaning red-black tree (Sedgewick)
? ---------------------------------------------------------------------------

? Node layout: {"v", "left", "right", "color"} with color "R"/"B"; null
? links count as black. insert() and remove() recurse (depth O(log n)) and
? keep the invariants: no red parent with a red child, equal black height
? on every root-to-null path, black root. remove() pre-checks contains()
? so the rebalancing always runs on a key that is present.

spell red_black_tree() {
    bind root = null
    bind n = 0
    bind key_type = null
    bind added = Off
    bind list_pop = pop
    spell check_key(v) {
        bind kind = _key_kind(v)
        if key_type == null {
            key_type = kind
            return null
        }
        if kind != key_type {
            assert(Off, "Candy: keys must be all numbers or all texts")
        }
        return null
    }
    spell guard_key(v) {
        if key_type != null {
            check_key(v)
        }
        return null
    }
    spell is_red(x) {
        if x == null {
            return Off
        }
        return x["color"] == "R"
    }
    spell red_left_left(x) {
        if x["left"] == null {
            return Off
        }
        return is_red(x["left"]["left"])
    }
    spell flip(h) {
        if h["color"] == "R" {
            h["color"] = "B"
        } else {
            h["color"] = "R"
        }
        if h["left"] != null {
            if h["left"]["color"] == "R" {
                h["left"]["color"] = "B"
            } else {
                h["left"]["color"] = "R"
            }
        }
        if h["right"] != null {
            if h["right"]["color"] == "R" {
                h["right"]["color"] = "B"
            } else {
                h["right"]["color"] = "R"
            }
        }
        return null
    }
    spell rotate_left(h) {
        bind x = h["right"]
        h["right"] = x["left"]
        x["left"] = h
        x["color"] = h["color"]
        h["color"] = "R"
        return x
    }
    spell rotate_right(h) {
        bind x = h["left"]
        h["left"] = x["right"]
        x["right"] = h
        x["color"] = h["color"]
        h["color"] = "R"
        return x
    }
    spell balance(h) {
        if is_red(h["right"]) and not is_red(h["left"]) {
            h = rotate_left(h)
        }
        if is_red(h["left"]) and is_red(h["left"]["left"]) {
            h = rotate_right(h)
        }
        if is_red(h["left"]) and is_red(h["right"]) {
            flip(h)
        }
        return h
    }
    spell ins(h, v) {
        if h == null {
            n = n + 1
            added = On
            return {"v": v, "left": null, "right": null, "color": "R"}
        }
        if v == h["v"] {
            return h
        }
        if v < h["v"] {
            h["left"] = ins(h["left"], v)
        } else {
            h["right"] = ins(h["right"], v)
        }
        if is_red(h["right"]) and not is_red(h["left"]) {
            h = rotate_left(h)
        }
        if is_red(h["left"]) and is_red(h["left"]["left"]) {
            h = rotate_right(h)
        }
        if is_red(h["left"]) and is_red(h["right"]) {
            flip(h)
        }
        return h
    }
    spell move_red_left(h) {
        flip(h)
        if h["right"] != null and is_red(h["right"]["left"]) {
            h["right"] = rotate_right(h["right"])
            h = rotate_left(h)
            flip(h)
        }
        return h
    }
    spell move_red_right(h) {
        flip(h)
        if red_left_left(h) {
            h = rotate_right(h)
            flip(h)
        }
        return h
    }
    spell delete_min(h) {
        if h["left"] == null {
            return null
        }
        if not is_red(h["left"]) and not red_left_left(h) {
            h = move_red_left(h)
        }
        h["left"] = delete_min(h["left"])
        return balance(h)
    }
    spell rm(h, v) {
        if v < h["v"] {
            if h["left"] != null {
                if not is_red(h["left"]) and not red_left_left(h) {
                    h = move_red_left(h)
                }
                h["left"] = rm(h["left"], v)
            }
            return balance(h)
        }
        if is_red(h["left"]) {
            h = rotate_right(h)
        }
        if v == h["v"] and h["right"] == null {
            n = n - 1
            return null
        }
        if h["right"] != null {
            if not is_red(h["right"]) and not is_red(h["right"]["left"]) {
                h = move_red_right(h)
            }
        }
        if v == h["v"] {
            n = n - 1
            bind s = h["right"]
            while s["left"] != null {
                s = s["left"]
            }
            h["v"] = s["v"]
            h["right"] = delete_min(h["right"])
        } else {
            h["right"] = rm(h["right"], v)
        }
        return balance(h)
    }
    spell insert(v) {
        check_key(v)
        added = Off
        root = ins(root, v)
        if root != null {
            root["color"] = "B"
        }
        return added
    }
    spell remove(v) {
        guard_key(v)
        if not contains(v) {
            return Off
        }
        if not is_red(root["left"]) and not is_red(root["right"]) {
            root["color"] = "R"
        }
        root = rm(root, v)
        if root != null {
            root["color"] = "B"
        }
        return On
    }
    spell contains(v) {
        guard_key(v)
        bind cur = root
        while cur != null {
            if v == cur["v"] {
                return On
            }
            if v < cur["v"] {
                cur = cur["left"]
            } else {
                cur = cur["right"]
            }
        }
        return Off
    }
    spell min() {
        assert(root != null, "Candy: min of an empty red-black tree")
        bind cur = root
        while cur["left"] != null {
            cur = cur["left"]
        }
        return cur["v"]
    }
    spell max() {
        assert(root != null, "Candy: max of an empty red-black tree")
        bind cur = root
        while cur["right"] != null {
            cur = cur["right"]
        }
        return cur["v"]
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell height() {
        if root == null {
            return 0
        }
        bind best = 0
        bind stack = [[root, 1]]
        while length(stack) > 0 {
            bind top = list_pop(stack)
            if top[1] > best {
                best = top[1]
            }
            if top[0]["left"] != null {
                append(stack, [top[0]["left"], top[1] + 1])
            }
            if top[0]["right"] != null {
                append(stack, [top[0]["right"], top[1] + 1])
            }
        }
        return best
    }
    spell inorder() {
        bind out = []
        bind stack = []
        bind cur = root
        bind going = On
        while going {
            while cur != null {
                append(stack, cur)
                cur = cur["left"]
            }
            if length(stack) == 0 {
                going = Off
            } else {
                cur = list_pop(stack)
                append(out, cur["v"])
                cur = cur["right"]
            }
        }
        return out
    }
    spell to_list() {
        return inorder()
    }
    spell validate(node) {
        ? black-height when valid, -1 when a rule is broken
        if node == null {
            return 0
        }
        bind lh = validate(node["left"])
        if lh < 0 {
            return -1
        }
        bind rh = validate(node["right"])
        if rh < 0 {
            return -1
        }
        if lh != rh {
            return -1
        }
        if node["color"] == "R" {
            if is_red(node["left"]) or is_red(node["right"]) {
                return -1
            }
            return lh
        }
        if node["color"] != "B" {
            return -1
        }
        return lh + 1
    }
    spell is_valid() {
        if root == null {
            return On
        }
        if root["color"] != "B" {
            return Off
        }
        return validate(root) >= 0
    }
    spell clear() {
        root = null
        n = 0
        key_type = null
        return null
    }
    return {"kind": "candy/red-black-tree", "insert": insert, "remove": remove,
            "contains": contains, "min": min, "max": max, "size": size,
            "is_empty": is_empty, "height": height, "inorder": inorder,
            "to_list": to_list, "is_valid": is_valid, "clear": clear}
}

? ---------------------------------------------------------------------------
? BinaryHeap — binary min-heap (max-heap with descending = On)
? ---------------------------------------------------------------------------

? The internal array is the heap: children of i are 2i+1 and 2i+2.
? binary_heap_from(items, ...) heapifies bottom-up in O(n) instead of
? pushing one by one. to_list() returns the array as-is, which is heap
? order, NOT sorted order. Values must be all numbers or all texts.

spell _heap_build(descending, source) {
    bind data = []
    bind key_type = null
    bind list_pop = pop
    spell before(a, b) {
        if descending {
            return a > b
        }
        return a < b
    }
    spell check_key(v) {
        bind kind = _key_kind(v)
        if key_type == null {
            key_type = kind
            return null
        }
        if kind != key_type {
            assert(Off, "Candy: keys must be all numbers or all texts")
        }
        return null
    }
    spell sift_up(i) {
        bind going = On
        while going {
            if i == 0 {
                going = Off
            } else {
                bind p = (i - 1) // 2
                if before(data[i], data[p]) {
                    bind t = data[i]
                    data[i] = data[p]
                    data[p] = t
                    i = p
                } else {
                    going = Off
                }
            }
        }
        return null
    }
    spell sift_down(i) {
        bind size = length(data)
        bind going = On
        while going {
            bind left = i * 2 + 1
            bind right = left + 1
            bind best = i
            if left < size and before(data[left], data[best]) {
                best = left
            }
            if right < size and before(data[right], data[best]) {
                best = right
            }
            if best == i {
                going = Off
            } else {
                bind t = data[i]
                data[i] = data[best]
                data[best] = t
                i = best
            }
        }
        return null
    }
    spell push(v) {
        check_key(v)
        append(data, v)
        sift_up(length(data) - 1)
        return null
    }
    spell pop() {
        assert(length(data) > 0, "Candy: pop from an empty heap")
        bind result = data[0]
        bind last = list_pop(data)
        if length(data) > 0 {
            data[0] = last
            sift_down(0)
        }
        return result
    }
    spell peek() {
        assert(length(data) > 0, "Candy: peek at an empty heap")
        return data[0]
    }
    spell size() {
        return length(data)
    }
    spell is_empty() {
        return length(data) == 0
    }
    spell to_list() {
        return copy(data)
    }
    spell clear() {
        data = []
        key_type = null
        return null
    }
    spell init() {
        if source != null {
            for v in source {
                check_key(v)
                append(data, v)
            }
            bind i = (length(data) // 2) - 1
            while i >= 0 {
                sift_down(i)
                i = i - 1
            }
        }
        return null
    }
    init()
    return {"kind": "candy/binary-heap", "push": push, "pop": pop,
            "peek": peek, "size": size, "is_empty": is_empty,
            "to_list": to_list, "clear": clear}
}

spell binary_heap(descending = Off) {
    return _heap_build(descending, null)
}

spell binary_heap_from(items, descending = Off) {
    assert(type_of(items) == "list", "Candy: binary_heap_from needs a list")
    return _heap_build(descending, items)
}

? ---------------------------------------------------------------------------
? Trie — prefix tree over texts
? ---------------------------------------------------------------------------

? Node layout: {"end": bool, "next": {ch: node}} where the map keys are
? 1-char texts. remove() prunes branches that became useless, walking the
? path bottom-up. words_with_prefix() collects with an explicit stack and
? sorts the result. The empty text is a legal word (it lives on the root).

spell trie() {
    bind root = _trie_node()
    bind n = 0
    spell node_for(word) {
        bind cur = root
        for ch in word {
            if ch in cur["next"] {
                cur = cur["next"][ch]
            } else {
                return null
            }
        }
        return cur
    }
    spell insert(word) {
        assert(type_of(word) == "text", "Candy: trie words must be text")
        bind cur = root
        for ch in word {
            if not (ch in cur["next"]) {
                cur["next"][ch] = _trie_node()
            }
            cur = cur["next"][ch]
        }
        if not cur["end"] {
            cur["end"] = On
            n = n + 1
        }
        return null
    }
    spell contains(word) {
        assert(type_of(word) == "text", "Candy: trie words must be text")
        bind cur = node_for(word)
        if cur == null {
            return Off
        }
        return cur["end"]
    }
    spell starts_with(prefix) {
        assert(type_of(prefix) == "text", "Candy: trie prefixes must be text")
        return node_for(prefix) != null
    }
    spell remove(word) {
        assert(type_of(word) == "text", "Candy: trie words must be text")
        bind path = []
        bind cur = root
        for ch in word {
            if ch in cur["next"] {
                append(path, [cur, ch])
                cur = cur["next"][ch]
            } else {
                return Off
            }
        }
        if not cur["end"] {
            return Off
        }
        cur["end"] = Off
        n = n - 1
        bind i = length(path) - 1
        while i >= 0 {
            bind parent = path[i][0]
            bind ch = path[i][1]
            bind child = parent["next"][ch]
            if not child["end"] and length(child["next"]) == 0 {
                bind fresh = {}
                for k in parent["next"] {
                    if k != ch {
                        fresh[k] = parent["next"][k]
                    }
                }
                parent["next"] = fresh
            } else {
                i = -1
            }
            i = i - 1
        }
        return On
    }
    spell words_with_prefix(prefix) {
        assert(type_of(prefix) == "text", "Candy: trie prefixes must be text")
        bind start = node_for(prefix)
        if start == null {
            return []
        }
        bind out = []
        bind stack = [[start, prefix]]
        while length(stack) > 0 {
            bind top = pop(stack)
            if top[0]["end"] {
                append(out, top[1])
            }
            for ch in top[0]["next"] {
                append(stack, [top[0]["next"][ch], top[1] + ch])
            }
        }
        _sort_texts(out)
        return out
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell to_list() {
        return words_with_prefix("")
    }
    spell longest_common_prefix() {
        bind out = ""
        bind cur = root
        bind going = On
        while going {
            if cur["end"] {
                going = Off
            } else {
                if length(cur["next"]) != 1 {
                    going = Off
                } else {
                    bind only_ch = ""
                    for ch in cur["next"] {
                        only_ch = ch
                    }
                    out = out + only_ch
                    cur = cur["next"][only_ch]
                }
            }
        }
        return out
    }
    spell clear() {
        root = _trie_node()
        n = 0
        return null
    }
    return {"kind": "candy/trie", "insert": insert, "remove": remove,
            "contains": contains, "starts_with": starts_with,
            "words_with_prefix": words_with_prefix, "size": size,
            "is_empty": is_empty, "to_list": to_list,
            "longest_common_prefix": longest_common_prefix,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? SegmentTree — range-SUM queries over a numeric list (iterative, 2n array)
? ---------------------------------------------------------------------------

? Bottom-up iterative layout: tree has 2n slots, the leaves are
? tree[n + i], internal node j holds tree[2j] + tree[2j + 1] (slot 0
? unused). query(start, end) sums the half-open range [start, end).

spell segment_tree(values) {
    assert(type_of(values) == "list", "Candy: segment tree needs a list of numbers")
    bind n = length(values)
    bind tree = []
    for i in range(0, 2 * n) {
        append(tree, 0)
    }
    for i in range(0, n) {
        bind v = values[i]
        assert(type_of(v) == "integer" or type_of(v) == "decimal",
               "Candy: segment tree values must be numbers")
        tree[n + i] = v
    }
    bind j = n - 1
    while j >= 1 {
        tree[j] = tree[j * 2] + tree[j * 2 + 1]
        j = j - 1
    }
    spell size() {
        return n
    }
    spell get(i) {
        assert(type_of(i) == "integer", "Candy: segment tree index must be an integer")
        assert(i >= 0 and i < n, "Candy: segment tree index out of range")
        return tree[i + n]
    }
    spell update(i, v) {
        assert(type_of(i) == "integer", "Candy: segment tree index must be an integer")
        assert(i >= 0 and i < n, "Candy: segment tree index out of range")
        assert(type_of(v) == "integer" or type_of(v) == "decimal",
               "Candy: segment tree values must be numbers")
        tree[i + n] = v
        bind j = (i + n) // 2
        while j >= 1 {
            tree[j] = tree[j * 2] + tree[j * 2 + 1]
            j = j // 2
        }
        return null
    }
    spell query(start, end) {
        assert(type_of(start) == "integer" and type_of(end) == "integer",
               "Candy: segment tree indexes must be integers")
        assert(start >= 0 and start <= end and end <= n,
               "Candy: segment tree query range out of bounds")
        bind result = 0
        bind lo = start + n
        bind hi = end + n
        while lo < hi {
            if lo % 2 == 1 {
                result = result + tree[lo]
                lo = lo + 1
            }
            if hi % 2 == 1 {
                hi = hi - 1
                result = result + tree[hi]
            }
            lo = lo // 2
            hi = hi // 2
        }
        return result
    }
    spell to_list() {
        bind out = []
        for i in range(n, 2 * n) {
            append(out, tree[i])
        }
        return out
    }
    spell clear() {
        tree = []
        n = 0
        return null
    }
    return {"kind": "candy/segment-tree", "size": size, "get": get,
            "update": update, "query": query, "to_list": to_list,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? FenwickTree — binary indexed tree for prefix sums (0-based API)
? ---------------------------------------------------------------------------

? fenwick_tree(n) gives n zeros; fenwick_tree(list) builds in O(n)
? by propagating each slot to its parent once. Storage is 1-based (slot 0
? unused) while add/prefix/range_sum take 0-based indexes. The lowbit table
? is precomputed so every step is O(1) arithmetic.

spell fenwick_tree(source) {
    bind n = 0
    bind tree = [0]
    bind lowbit = [0]
    spell grow_lowbit(i) {
        if i == 1 {
            append(lowbit, 1)
            return null
        }
        if i % 2 == 0 {
            append(lowbit, lowbit[i // 2] * 2)
            return null
        }
        append(lowbit, 1)
        return null
    }
    spell size() {
        return n
    }
    spell add(i, delta) {
        assert(type_of(i) == "integer", "Candy: fenwick index must be an integer")
        assert(i >= 0 and i < n, "Candy: fenwick index out of range")
        assert(type_of(delta) == "integer" or type_of(delta) == "decimal",
               "Candy: fenwick delta must be a number")
        bind j = i + 1
        while j <= n {
            tree[j] = tree[j] + delta
            j = j + lowbit[j]
        }
        return null
    }
    spell prefix(i) {
        assert(type_of(i) == "integer", "Candy: fenwick index must be an integer")
        assert(i >= 0 and i <= n, "Candy: fenwick prefix index out of range")
        bind j = i
        bind result = 0
        while j > 0 {
            result = result + tree[j]
            j = j - lowbit[j]
        }
        return result
    }
    spell range_sum(start, end) {
        assert(type_of(start) == "integer" and type_of(end) == "integer",
               "Candy: fenwick indexes must be integers")
        assert(start >= 0 and start <= end and end <= n,
               "Candy: fenwick range out of bounds")
        return prefix(end) - prefix(start)
    }
    spell to_list() {
        bind out = []
        bind i = 1
        while i <= n {
            append(out, prefix(i) - prefix(i - 1))
            i = i + 1
        }
        return out
    }
    spell clear() {
        n = 0
        tree = [0]
        lowbit = [0]
        return null
    }
    if type_of(source) == "integer" {
        assert(source >= 0, "Candy: fenwick size must be at least 0")
        n = source
        bind i = 1
        while i <= n {
            append(tree, 0)
            grow_lowbit(i)
            i = i + 1
        }
    } else {
        assert(type_of(source) == "list", "Candy: fenwick tree needs an integer or a list")
        n = length(source)
        bind i = 1
        while i <= n {
            bind v = source[i - 1]
            assert(type_of(v) == "integer" or type_of(v) == "decimal",
                   "Candy: fenwick tree values must be numbers")
            append(tree, v)
            grow_lowbit(i)
            i = i + 1
        }
        i = 1
        while i <= n {
            bind p = i + lowbit[i]
            if p <= n {
                tree[p] = tree[p] + tree[i]
            }
            i = i + 1
        }
    }
    return {"kind": "candy/fenwick-tree", "add": add, "prefix": prefix,
            "range_sum": range_sum, "size": size, "to_list": to_list,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? BTree — B-tree of the CLRS kind, proactive split on insert
? ---------------------------------------------------------------------------

? t = min_degree >= 2. Every non-root node holds t-1..2t-1 keys; keys
? within a node are sorted. insert() splits full nodes on the way down,
? remove() borrows from siblings or merges so no node is left underfull.
? All descending loops/spells are depth-O(log n).

spell b_tree(min_degree = 2) {
    assert(type_of(min_degree) == "integer" and min_degree >= 2,
           "Candy: min_degree must be an integer of at least 2")
    bind t = min_degree
    bind root = _bnode(On)
    bind n = 0
    bind key_type = null
    bind leaf_depth = -1
    bind valid = On
    spell check_key(v) {
        bind kind = _key_kind(v)
        if key_type == null {
            key_type = kind
            return null
        }
        if kind != key_type {
            assert(Off, "Candy: keys must be all numbers or all texts")
        }
        return null
    }
    spell guard_key(v) {
        if key_type != null {
            check_key(v)
        }
        return null
    }
    spell split_child(x, i) {
        bind y = x["kids"][i]
        bind z = _bnode(y["leaf"])
        for j in range(t, 2 * t - 1) {
            append(z["keys"], y["keys"][j])
        }
        if not y["leaf"] {
            for j in range(t, 2 * t) {
                append(z["kids"], y["kids"][j])
            }
        }
        bind mid = y["keys"][t - 1]
        _truncate(y["keys"], t - 1)
        if not y["leaf"] {
            _truncate(y["kids"], t)
        }
        _list_insert(x["keys"], i, mid)
        _list_insert(x["kids"], i + 1, z)
        return null
    }
    spell insert_nonfull(x, key) {
        if x["leaf"] {
            bind i = length(x["keys"]) - 1
            append(x["keys"], null)
            while i >= 0 and x["keys"][i] > key {
                x["keys"][i + 1] = x["keys"][i]
                i = i - 1
            }
            x["keys"][i + 1] = key
            return null
        }
        bind i = length(x["keys"]) - 1
        while i >= 0 and x["keys"][i] > key {
            i = i - 1
        }
        i = i + 1
        if length(x["kids"][i]["keys"]) == 2 * t - 1 {
            split_child(x, i)
            if key > x["keys"][i] {
                i = i + 1
            }
        }
        insert_nonfull(x["kids"][i], key)
        return null
    }
    spell insert(key) {
        check_key(key)
        if contains(key) {
            return Off
        }
        bind r = root
        if length(r["keys"]) == 2 * t - 1 {
            bind s = _bnode(Off)
            root = s
            append(s["kids"], r)
            split_child(s, 0)
            insert_nonfull(s, key)
        } else {
            insert_nonfull(r, key)
        }
        n = n + 1
        return On
    }
    spell contains(key) {
        guard_key(key)
        bind cur = root
        bind going = On
        while going {
            bind i = 0
            while i < length(cur["keys"]) and cur["keys"][i] < key {
                i = i + 1
            }
            if i < length(cur["keys"]) {
                if cur["keys"][i] == key {
                    return On
                }
            }
            if cur["leaf"] {
                return Off
            }
            cur = cur["kids"][i]
        }
        return Off
    }
    spell node_max(x) {
        bind cur = x
        while not cur["leaf"] {
            cur = cur["kids"][length(cur["kids"]) - 1]
        }
        return cur["keys"][length(cur["keys"]) - 1]
    }
    spell node_min(x) {
        bind cur = x
        while not cur["leaf"] {
            cur = cur["kids"][0]
        }
        return cur["keys"][0]
    }
    spell merge_kids(x, i) {
        ? CLRS merge: the separator key comes down between the two halves
        bind left = x["kids"][i]
        bind right = x["kids"][i + 1]
        append(left["keys"], x["keys"][i])
        for k in right["keys"] {
            append(left["keys"], k)
        }
        for c in right["kids"] {
            append(left["kids"], c)
        }
        _remove_at(x["keys"], i)
        _remove_at(x["kids"], i + 1)
        return null
    }
    spell borrow_from_left(x, i) {
        bind child = x["kids"][i]
        bind sib = x["kids"][i - 1]
        _list_insert(child["keys"], 0, x["keys"][i - 1])
        x["keys"][i - 1] = sib["keys"][length(sib["keys"]) - 1]
        pop(sib["keys"])
        if not child["leaf"] {
            _list_insert(child["kids"], 0, sib["kids"][length(sib["kids"]) - 1])
            pop(sib["kids"])
        }
        return null
    }
    spell borrow_from_right(x, i) {
        bind child = x["kids"][i]
        bind sib = x["kids"][i + 1]
        append(child["keys"], x["keys"][i])
        x["keys"][i] = sib["keys"][0]
        _remove_at(sib["keys"], 0)
        if not child["leaf"] {
            append(child["kids"], sib["kids"][0])
            _remove_at(sib["kids"], 0)
        }
        return null
    }
    spell fix_underfull(x, i) {
        if i > 0 and length(x["kids"][i - 1]["keys"]) >= t {
            borrow_from_left(x, i)
            return i
        }
        if i < length(x["kids"]) - 1 and length(x["kids"][i + 1]["keys"]) >= t {
            borrow_from_right(x, i)
            return i
        }
        if i > 0 {
            merge_kids(x, i - 1)
            return i - 1
        }
        merge_kids(x, i)
        return i
    }
    spell delete(x, key) {
        bind i = 0
        while i < length(x["keys"]) and x["keys"][i] < key {
            i = i + 1
        }
        if i < length(x["keys"]) and x["keys"][i] == key {
            if x["leaf"] {
                _remove_at(x["keys"], i)
                return null
            }
            if length(x["kids"][i]["keys"]) >= t {
                bind pred = node_max(x["kids"][i])
                x["keys"][i] = pred
                delete(x["kids"][i], pred)
                return null
            }
            if length(x["kids"][i + 1]["keys"]) >= t {
                bind succ = node_min(x["kids"][i + 1])
                x["keys"][i] = succ
                delete(x["kids"][i + 1], succ)
                return null
            }
            merge_kids(x, i)
            delete(x["kids"][i], key)
            return null
        }
        if x["leaf"] {
            return null
        }
        if length(x["kids"][i]["keys"]) < t {
            i = fix_underfull(x, i)
        }
        delete(x["kids"][i], key)
        return null
    }
    spell remove(key) {
        guard_key(key)
        if not contains(key) {
            return Off
        }
        delete(root, key)
        if length(root["keys"]) == 0 and not root["leaf"] {
            root = root["kids"][0]
        }
        n = n - 1
        return On
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell height() {
        if n == 0 {
            return 0
        }
        bind h = 1
        bind cur = root
        while not cur["leaf"] {
            cur = cur["kids"][0]
            h = h + 1
        }
        return h
    }
    spell walk(x, out) {
        bind i = 0
        while i < length(x["keys"]) {
            if not x["leaf"] {
                walk(x["kids"][i], out)
            }
            append(out, x["keys"][i])
            i = i + 1
        }
        if not x["leaf"] {
            walk(x["kids"][length(x["kids"]) - 1], out)
        }
        return null
    }
    spell to_list() {
        bind out = []
        walk(root, out)
        return out
    }
    spell validate(x, is_root, depth) {
        if length(x["keys"]) > 2 * t - 1 {
            valid = Off
            return null
        }
        if not is_root {
            if length(x["keys"]) < t - 1 {
                valid = Off
                return null
            }
        }
        bind i = 1
        while i < length(x["keys"]) {
            if not (x["keys"][i - 1] < x["keys"][i]) {
                valid = Off
                return null
            }
            i = i + 1
        }
        if x["leaf"] {
            if leaf_depth == -1 {
                leaf_depth = depth
            } else {
                if leaf_depth != depth {
                    valid = Off
                    return null
                }
            }
            return null
        }
        if length(x["kids"]) != length(x["keys"]) + 1 {
            valid = Off
            return null
        }
        for c in x["kids"] {
            validate(c, Off, depth + 1)
        }
        return null
    }
    spell is_valid() {
        valid = On
        leaf_depth = -1
        validate(root, On, 1)
        return valid
    }
    spell clear() {
        root = _bnode(On)
        n = 0
        key_type = null
        return null
    }
    return {"kind": "candy/b-tree", "insert": insert, "contains": contains,
            "remove": remove, "size": size, "is_empty": is_empty,
            "height": height, "to_list": to_list, "is_valid": is_valid,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? BPlusTree — B+ tree: data lives in linked leaves
? ---------------------------------------------------------------------------

? Internal nodes hold separator keys only; every key/value pair sits in a
? leaf, and leaves are chained with "next" so range_query can scan. A leaf
? split copies the right half's smallest key up; an internal split moves
? the median up. remove() borrows or merges as in a B-tree, minus the
? separator (B+ separators are routing copies, not data).

spell bplus_tree(min_degree = 2) {
    assert(type_of(min_degree) == "integer" and min_degree >= 2,
           "Candy: min_degree must be an integer of at least 2")
    bind t = min_degree
    bind root = {"keys": [], "vals": [], "next": null, "leaf": On}
    bind n = 0
    bind key_type = null
    bind leaf_depth = -1
    spell check_key(v) {
        bind kind = _key_kind(v)
        if key_type == null {
            key_type = kind
            return null
        }
        if kind != key_type {
            assert(Off, "Candy: keys must be all numbers or all texts")
        }
        return null
    }
    spell guard_key(v) {
        if key_type != null {
            check_key(v)
        }
        return null
    }
    spell child_index(x, key) {
        bind i = 0
        while i < length(x["keys"]) and key >= x["keys"][i] {
            i = i + 1
        }
        return i
    }
    spell leaf_index(leaf, key) {
        bind i = 0
        while i < length(leaf["keys"]) and leaf["keys"][i] < key {
            i = i + 1
        }
        return i
    }
    spell find_leaf(key) {
        bind cur = root
        while not cur["leaf"] {
            cur = cur["kids"][child_index(cur, key)]
        }
        return cur
    }
    spell split_child(x, i) {
        bind y = x["kids"][i]
        if y["leaf"] {
            bind z = {"keys": [], "vals": [], "next": y["next"], "leaf": On}
            for j in range(t, 2 * t - 1) {
                append(z["keys"], y["keys"][j])
                append(z["vals"], y["vals"][j])
            }
            bind sep = y["keys"][t]
            _truncate(y["keys"], t)
            _truncate(y["vals"], t)
            y["next"] = z
            _list_insert(x["keys"], i, sep)
            _list_insert(x["kids"], i + 1, z)
            return null
        }
        bind z = {"keys": [], "kids": [], "leaf": Off}
        for j in range(t, 2 * t - 1) {
            append(z["keys"], y["keys"][j])
        }
        for j in range(t, 2 * t) {
            append(z["kids"], y["kids"][j])
        }
        bind mid = y["keys"][t - 1]
        _truncate(y["keys"], t - 1)
        _truncate(y["kids"], t)
        _list_insert(x["keys"], i, mid)
        _list_insert(x["kids"], i + 1, z)
        return null
    }
    spell insert_nonfull(x, key, value) {
        if x["leaf"] {
            bind i = leaf_index(x, key)
            _list_insert(x["keys"], i, key)
            _list_insert(x["vals"], i, value)
            return null
        }
        bind i = child_index(x, key)
        if length(x["kids"][i]["keys"]) == 2 * t - 1 {
            split_child(x, i)
            if key >= x["keys"][i] {
                i = i + 1
            }
        }
        insert_nonfull(x["kids"][i], key, value)
        return null
    }
    spell insert(key, value) {
        check_key(key)
        bind leaf = find_leaf(key)
        bind i = leaf_index(leaf, key)
        if i < length(leaf["keys"]) and leaf["keys"][i] == key {
            leaf["vals"][i] = value
            return null
        }
        if length(root["keys"]) == 2 * t - 1 {
            bind s = {"keys": [], "kids": [], "leaf": Off}
            append(s["kids"], root)
            root = s
            split_child(s, 0)
            insert_nonfull(s, key, value)
        } else {
            insert_nonfull(root, key, value)
        }
        n = n + 1
        return null
    }
    spell get(key, default = null) {
        guard_key(key)
        bind leaf = find_leaf(key)
        bind i = leaf_index(leaf, key)
        if i < length(leaf["keys"]) and leaf["keys"][i] == key {
            return leaf["vals"][i]
        }
        return default
    }
    spell has(key) {
        guard_key(key)
        bind leaf = find_leaf(key)
        bind i = leaf_index(leaf, key)
        if i < length(leaf["keys"]) and leaf["keys"][i] == key {
            return On
        }
        return Off
    }
    spell merge_kids(x, i) {
        ? B+ merge: leaves just concatenate (the separator is a routing
        ? copy); internal nodes take the separator down so kids always
        ? equals keys + 1
        bind left = x["kids"][i]
        bind right = x["kids"][i + 1]
        if left["leaf"] {
            for k in right["keys"] {
                append(left["keys"], k)
            }
            for v in right["vals"] {
                append(left["vals"], v)
            }
            left["next"] = right["next"]
        } else {
            append(left["keys"], x["keys"][i])
            for k in right["keys"] {
                append(left["keys"], k)
            }
            for c in right["kids"] {
                append(left["kids"], c)
            }
        }
        _remove_at(x["keys"], i)
        _remove_at(x["kids"], i + 1)
        return null
    }
    spell borrow_from_left(x, i) {
        bind child = x["kids"][i]
        bind sib = x["kids"][i - 1]
        if child["leaf"] {
            _list_insert(child["keys"], 0, sib["keys"][length(sib["keys"]) - 1])
            _list_insert(child["vals"], 0, sib["vals"][length(sib["vals"]) - 1])
            pop(sib["keys"])
            pop(sib["vals"])
        } else {
            _list_insert(child["kids"], 0, sib["kids"][length(sib["kids"]) - 1])
            _list_insert(child["keys"], 0, x["keys"][i - 1])
            x["keys"][i - 1] = sib["keys"][length(sib["keys"]) - 1]
            pop(sib["keys"])
            pop(sib["kids"])
        }
        ? the child's minimum changed: re-canonicalise the separator
        x["keys"][i - 1] = leftmost_key(x["kids"][i])
        return null
    }
    spell borrow_from_right(x, i) {
        bind child = x["kids"][i]
        bind sib = x["kids"][i + 1]
        if child["leaf"] {
            append(child["keys"], sib["keys"][0])
            append(child["vals"], sib["vals"][0])
            _remove_at(sib["keys"], 0)
            _remove_at(sib["vals"], 0)
        } else {
            append(child["kids"], sib["kids"][0])
            append(child["keys"], x["keys"][i])
            x["keys"][i] = sib["keys"][0]
            _remove_at(sib["keys"], 0)
            _remove_at(sib["kids"], 0)
        }
        ? the sibling's minimum changed: re-canonicalise the separator
        x["keys"][i] = leftmost_key(x["kids"][i + 1])
        return null
    }
    spell fix_underfull(x, i) {
        if i > 0 and length(x["kids"][i - 1]["keys"]) >= t {
            borrow_from_left(x, i)
            return i
        }
        if i < length(x["kids"]) - 1 and length(x["kids"][i + 1]["keys"]) >= t {
            borrow_from_right(x, i)
            return i
        }
        if i > 0 {
            merge_kids(x, i - 1)
            return i - 1
        }
        merge_kids(x, i)
        return i
    }
    spell delete(x, key) {
        if x["leaf"] {
            bind i = leaf_index(x, key)
            _remove_at(x["keys"], i)
            _remove_at(x["vals"], i)
            return null
        }
        bind ci = child_index(x, key)
        if length(x["kids"][ci]["keys"]) < t {
            ci = fix_underfull(x, ci)
        }
        delete(x["kids"][ci], key)
        return null
    }
    spell remove(key) {
        guard_key(key)
        if not has(key) {
            return Off
        }
        delete(root, key)
        if not root["leaf"] and length(root["keys"]) == 0 {
            root = root["kids"][0]
        }
        n = n - 1
        return On
    }
    spell range_query(start, end) {
        guard_key(start)
        guard_key(end)
        bind out = []
        if end <= start {
            return out
        }
        bind leaf = find_leaf(start)
        while leaf != null {
            bind i = leaf_index(leaf, start)
            while i < length(leaf["keys"]) {
                if leaf["keys"][i] >= end {
                    return out
                }
                append(out, leaf["vals"][i])
                i = i + 1
            }
            leaf = leaf["next"]
        }
        return out
    }
    spell leftmost_leaf() {
        bind cur = root
        while not cur["leaf"] {
            cur = cur["kids"][0]
        }
        return cur
    }
    spell to_list() {
        bind out = []
        bind leaf = leftmost_leaf()
        while leaf != null {
            for i in range(0, length(leaf["keys"])) {
                append(out, [leaf["keys"][i], leaf["vals"][i]])
            }
            leaf = leaf["next"]
        }
        return out
    }
    spell items() {
        return to_list()
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell height() {
        if n == 0 {
            return 0
        }
        bind h = 1
        bind cur = root
        while not cur["leaf"] {
            cur = cur["kids"][0]
            h = h + 1
        }
        return h
    }
    spell leftmost_key(x) {
        bind cur = x
        while not cur["leaf"] {
            cur = cur["kids"][0]
        }
        return cur["keys"][0]
    }
    spell rightmost_key(x) {
        bind cur = x
        while not cur["leaf"] {
            cur = cur["kids"][length(cur["kids"]) - 1]
        }
        return cur["keys"][length(cur["keys"]) - 1]
    }
    spell validate(x, is_root, depth) {
        if length(x["keys"]) > 2 * t - 1 {
            return Off
        }
        if not is_root {
            if length(x["keys"]) < t - 1 {
                return Off
            }
        }
        bind i = 1
        while i < length(x["keys"]) {
            if not (x["keys"][i - 1] < x["keys"][i]) {
                return Off
            }
            i = i + 1
        }
        if x["leaf"] {
            if leaf_depth == -1 {
                leaf_depth = depth
            } else {
                if leaf_depth != depth {
                    return Off
                }
            }
            if length(x["vals"]) != length(x["keys"]) {
                return Off
            }
            return On
        }
        if length(x["kids"]) != length(x["keys"]) + 1 {
            return Off
        }
        bind j = 0
        while j < length(x["keys"]) {
            if rightmost_key(x["kids"][j]) >= x["keys"][j] {
                return Off
            }
            if leftmost_key(x["kids"][j + 1]) < x["keys"][j] {
                return Off
            }
            j = j + 1
        }
        for c in x["kids"] {
            if not validate(c, Off, depth + 1) {
                return Off
            }
        }
        return On
    }
    spell is_valid() {
        leaf_depth = -1
        if not validate(root, On, 1) {
            return Off
        }
        bind seen = 0
        bind prev = null
        bind leaf = leftmost_leaf()
        while leaf != null {
            for i in range(0, length(leaf["keys"])) {
                bind k = leaf["keys"][i]
                if prev != null and not (prev < k) {
                    return Off
                }
                prev = k
                seen = seen + 1
            }
            leaf = leaf["next"]
        }
        return seen == n
    }
    spell clear() {
        root = {"keys": [], "vals": [], "next": null, "leaf": On}
        n = 0
        key_type = null
        return null
    }
    return {"kind": "candy/bplus-tree", "insert": insert, "get": get,
            "has": has, "remove": remove, "range_query": range_query,
            "to_list": to_list, "items": items, "size": size,
            "is_empty": is_empty, "height": height, "is_valid": is_valid,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? SuffixArray — suffix-array based text index (suffix tree queries)
? ---------------------------------------------------------------------------

? Honest note: this is a suffix ARRAY plus binary searches, not a suffix
? tree — but it answers the same queries. The array of suffix start
? indexes is built by insertion-sorting the suffixes (fine for texts up to
? a few hundred characters). contains/occurrences binary-search with the
? pattern; longest_repeating_substring() uses the max LCP of adjacent
? sorted suffixes.

spell suffix_array(text) {
    assert(type_of(text) == "text", "Candy: suffix array needs a text")
    assert(length(text) >= 1, "Candy: suffix array needs a non-empty text")
    bind n = length(text)
    bind sa = [0]
    bind list_pop = pop
    spell suffix_less(i, j) {
        ? is the suffix starting at i smaller than the one at j
        bind a = i
        bind b = j
        bind going = On
        while going {
            if a >= n {
                return b < n
            }
            if b >= n {
                return Off
            }
            if text[a] < text[b] {
                return On
            }
            if text[a] > text[b] {
                return Off
            }
            a = a + 1
            b = b + 1
        }
        return Off
    }
    spell has_prefix(at, pattern) {
        if n - at < length(pattern) {
            return Off
        }
        bind i = 0
        while i < length(pattern) {
            if text[at + i] != pattern[i] {
                return Off
            }
            i = i + 1
        }
        return On
    }
    spell suffix_equals(at, t) {
        if n - at != length(t) {
            return Off
        }
        bind i = 0
        while i < length(t) {
            if text[at + i] != t[i] {
                return Off
            }
            i = i + 1
        }
        return On
    }
    spell compare_pattern(at, pattern) {
        ? -1: suffix < pattern, 0: suffix starts with pattern, 1: suffix > pattern
        bind i = 0
        while i < length(pattern) {
            if at + i >= n {
                return -1
            }
            if text[at + i] < pattern[i] {
                return -1
            }
            if text[at + i] > pattern[i] {
                return 1
            }
            i = i + 1
        }
        return 0
    }
    spell lower_bound(pattern) {
        bind lo = 0
        bind hi = n
        while lo < hi {
            bind mid = (lo + hi) // 2
            if compare_pattern(sa[mid], pattern) < 0 {
                lo = mid + 1
            } else {
                hi = mid
            }
        }
        return lo
    }
    spell upper_bound(pattern) {
        bind lo = 0
        bind hi = n
        while lo < hi {
            bind mid = (lo + hi) // 2
            if compare_pattern(sa[mid], pattern) <= 0 {
                lo = mid + 1
            } else {
                hi = mid
            }
        }
        return lo
    }
    spell lcp(a, b) {
        bind l = 0
        while a + l < n and b + l < n and text[a + l] == text[b + l] {
            l = l + 1
        }
        return l
    }
    spell slice_text(at, m) {
        bind out = ""
        bind i = 0
        while i < m {
            out = out + text[at + i]
            i = i + 1
        }
        return out
    }
    bind i = 1
    while i < n {
        bind pos = length(sa)
        bind j = 0
        bind placed = Off
        while j < length(sa) and not placed {
            if suffix_less(i, sa[j]) {
                pos = j
                placed = On
            } else {
                j = j + 1
            }
        }
        _list_insert(sa, pos, i)
        i = i + 1
    }
    spell contains(pattern) {
        assert(type_of(pattern) == "text", "Candy: pattern must be text")
        assert(length(pattern) >= 1, "Candy: empty pattern")
        bind lo = 0
        bind hi = n
        while lo < hi {
            bind mid = (lo + hi) // 2
            bind c = compare_pattern(sa[mid], pattern)
            if c == 0 {
                return On
            }
            if c < 0 {
                lo = mid + 1
            } else {
                hi = mid
            }
        }
        return Off
    }
    spell contains_suffix(t) {
        assert(type_of(t) == "text", "Candy: pattern must be text")
        assert(length(t) >= 1, "Candy: empty pattern")
        for k in sa {
            if suffix_equals(k, t) {
                return On
            }
        }
        return Off
    }
    spell occurrences(pattern) {
        assert(type_of(pattern) == "text", "Candy: pattern must be text")
        assert(length(pattern) >= 1, "Candy: empty pattern")
        return upper_bound(pattern) - lower_bound(pattern)
    }
    spell longest_repeating_substring() {
        bind best_len = 0
        bind best_at = 0
        bind k = 1
        while k < n {
            bind l = lcp(sa[k - 1], sa[k])
            if l > best_len {
                best_len = l
                best_at = k
            }
            k = k + 1
        }
        if best_len == 0 {
            return ""
        }
        return slice_text(sa[best_at], best_len)
    }
    spell size() {
        return n
    }
    spell suffixes() {
        return copy(sa)
    }
    spell clear() {
        sa = []
        n = 0
        text = ""
        return null
    }
    return {"kind": "candy/suffix-array", "contains": contains,
            "contains_suffix": contains_suffix, "occurrences": occurrences,
            "longest_repeating_substring": longest_repeating_substring,
            "size": size, "suffixes": suffixes, "clear": clear}
}

? ---------------------------------------------------------------------------
? Treap — randomized search tree via split/merge
? ---------------------------------------------------------------------------

? Every node carries a random priority; the tree is a BST by key and a
? heap by priority, which keeps it balanced with high probability. If a
? seed is passed, random_seed(seed) runs at construction — note this
? reseeds Arcana's ONE global random generator. split/merge recursion is
? expected O(log n) deep.

spell treap(seed = null) {
    if seed != null {
        assert(type_of(seed) == "integer", "Candy: treap seed must be an integer or null")
        random_seed(seed)
    }
    bind root = null
    bind n = 0
    bind key_type = null
    bind removed = Off
    spell check_key(v) {
        bind kind = _key_kind(v)
        if key_type == null {
            key_type = kind
            return null
        }
        if kind != key_type {
            assert(Off, "Candy: keys must be all numbers or all texts")
        }
        return null
    }
    spell guard_key(v) {
        if key_type != null {
            check_key(v)
        }
        return null
    }
    spell split(t, key) {
        if t == null {
            return [null, null]
        }
        if t["key"] < key {
            bind parts = split(t["right"], key)
            t["right"] = parts[0]
            return [t, parts[1]]
        }
        bind parts = split(t["left"], key)
        t["left"] = parts[1]
        return [parts[0], t]
    }
    spell merge(a, b) {
        if a == null {
            return b
        }
        if b == null {
            return a
        }
        if a["pri"] >= b["pri"] {
            a["right"] = merge(a["right"], b)
            return a
        }
        b["left"] = merge(a, b["left"])
        return b
    }
    spell insert(key) {
        check_key(key)
        if contains(key) {
            return Off
        }
        bind node = {"key": key, "pri": random(), "left": null, "right": null}
        bind parts = split(root, key)
        root = merge(parts[0], node)
        root = merge(root, parts[1])
        n = n + 1
        return On
    }
    spell rm(t, key) {
        if t == null {
            return null
        }
        if key < t["key"] {
            t["left"] = rm(t["left"], key)
            return t
        }
        if key > t["key"] {
            t["right"] = rm(t["right"], key)
            return t
        }
        removed = On
        n = n - 1
        return merge(t["left"], t["right"])
    }
    spell remove(key) {
        guard_key(key)
        removed = Off
        root = rm(root, key)
        return removed
    }
    spell contains(key) {
        guard_key(key)
        bind cur = root
        while cur != null {
            if key == cur["key"] {
                return On
            }
            if key < cur["key"] {
                cur = cur["left"]
            } else {
                cur = cur["right"]
            }
        }
        return Off
    }
    spell min() {
        assert(root != null, "Candy: min of an empty treap")
        bind cur = root
        while cur["left"] != null {
            cur = cur["left"]
        }
        return cur["key"]
    }
    spell max() {
        assert(root != null, "Candy: max of an empty treap")
        bind cur = root
        while cur["right"] != null {
            cur = cur["right"]
        }
        return cur["key"]
    }
    spell size() {
        return n
    }
    spell is_empty() {
        return n == 0
    }
    spell height() {
        if root == null {
            return 0
        }
        bind best = 0
        bind stack = [[root, 1]]
        while length(stack) > 0 {
            bind top = pop(stack)
            if top[1] > best {
                best = top[1]
            }
            if top[0]["left"] != null {
                append(stack, [top[0]["left"], top[1] + 1])
            }
            if top[0]["right"] != null {
                append(stack, [top[0]["right"], top[1] + 1])
            }
        }
        return best
    }
    spell inorder() {
        bind out = []
        bind stack = []
        bind cur = root
        bind going = On
        while going {
            while cur != null {
                append(stack, cur)
                cur = cur["left"]
            }
            if length(stack) == 0 {
                going = Off
            } else {
                cur = pop(stack)
                append(out, cur["key"])
                cur = cur["right"]
            }
        }
        return out
    }
    spell to_list() {
        return inorder()
    }
    spell clear() {
        root = null
        n = 0
        key_type = null
        return null
    }
    return {"kind": "candy/treap", "insert": insert, "remove": remove,
            "contains": contains, "min": min, "max": max, "size": size,
            "is_empty": is_empty, "height": height, "to_list": to_list,
            "clear": clear}
}
