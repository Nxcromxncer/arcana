? Candy 01: structures — linear data structures for Arcana.
?
? Part of the Candy library (Tier I, module 01 of 04).
? Import from your grimoire with:
?
?     import candy.structures            ? when the candy/ folder sits next to
?                                        ? your file, or is on ARCANA_PATH
?     import structures                  ? when this file sits next to yours
?
? Structures in this module: Stack, Queue, Deque, CircularQueue,
? PriorityQueue, MonotonicStack, MonotonicQueue, MinStack, MaxStack.
?
? Design: Arcana has no user-defined types, so every Candy structure is a
? *closure object*: `stack()` declares private state, inner spells that
? touch it, and returns a map of those spells. Call methods with indexing:
?
?     bind s = stack()
?     s["push"](7)
?     cast s["pop"]()                    ? 7
?
? Every instance carries a "kind" text ("candy/stack", ...) so programs can
? tell structures apart, plus size(), is_empty() and to_list() where they
? make sense. Misuse (popping an empty structure, bad capacity, ...) raises
? a catchable AssertionError through assert().

? ---------------------------------------------------------------------------
? Stack — last in, first out
? ---------------------------------------------------------------------------

spell stack() {
    bind items = []
    bind list_pop = pop
    spell push(v) {
        append(items, v)
        return null
    }
    spell pop() {
        assert(length(items) > 0, "Candy: pop from an empty stack")
        return list_pop(items)
    }
    spell peek() {
        assert(length(items) > 0, "Candy: peek at an empty stack")
        return items[length(items) - 1]
    }
    spell size() {
        return length(items)
    }
    spell is_empty() {
        return length(items) == 0
    }
    spell contains(v) {
        return v in items
    }
    spell to_list() {
        return copy(items)
    }
    spell clear() {
        items = []
        return null
    }
    return {"kind": "candy/stack", "push": push, "pop": pop, "peek": peek,
            "size": size, "is_empty": is_empty, "contains": contains,
            "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? Queue — first in, first out (amortised O(1) via a head index)
? ---------------------------------------------------------------------------

spell queue() {
    bind items = []
    bind head = 0
    spell enqueue(v) {
        append(items, v)
        return null
    }
    spell dequeue() {
        assert(head < length(items), "Candy: dequeue from an empty queue")
        bind v = items[head]
        head = head + 1
        if head == length(items) {
            items = []
            head = 0
        } else {
            if head > 64 and head * 2 >= length(items) {
                bind kept = []
                for i in range(head, length(items)) {
                    append(kept, items[i])
                }
                items = kept
                head = 0
            }
        }
        return v
    }
    spell front() {
        assert(head < length(items), "Candy: front of an empty queue")
        return items[head]
    }
    spell back() {
        assert(head < length(items), "Candy: back of an empty queue")
        return items[length(items) - 1]
    }
    spell size() {
        return length(items) - head
    }
    spell is_empty() {
        return head == length(items)
    }
    spell contains(v) {
        for i in range(head, length(items)) {
            if items[i] == v {
                return On
            }
        }
        return Off
    }
    spell to_list() {
        bind out = []
        for i in range(head, length(items)) {
            append(out, items[i])
        }
        return out
    }
    spell clear() {
        items = []
        head = 0
        return null
    }
    return {"kind": "candy/queue", "enqueue": enqueue, "dequeue": dequeue,
            "front": front, "back": back, "size": size, "is_empty": is_empty,
            "contains": contains, "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? Ring buffer shared by Deque and CircularQueue
? ---------------------------------------------------------------------------

spell _ring_new(cap) {
    bind buf = []
    for i in range(0, cap) {
        append(buf, null)
    }
    return {"buf": buf, "cap": cap, "head": 0, "count": 0}
}

spell _ring_grow(r) {
    bind fresh = _ring_new(r["cap"] * 2)
    for i in range(0, r["count"]) {
        fresh["buf"][i] = r["buf"][(r["head"] + i) % r["cap"]]
    }
    fresh["count"] = r["count"]
    return fresh
}

? ---------------------------------------------------------------------------
? Deque — double-ended queue on a growable ring buffer
? ---------------------------------------------------------------------------

spell deque() {
    bind r = _ring_new(8)
    spell push_front(v) {
        if r["count"] == r["cap"] {
            r = _ring_grow(r)
        }
        r["head"] = (r["head"] + r["cap"] - 1) % r["cap"]
        r["buf"][r["head"]] = v
        r["count"] = r["count"] + 1
        return null
    }
    spell push_back(v) {
        if r["count"] == r["cap"] {
            r = _ring_grow(r)
        }
        r["buf"][(r["head"] + r["count"]) % r["cap"]] = v
        r["count"] = r["count"] + 1
        return null
    }
    spell pop_front() {
        assert(r["count"] > 0, "Candy: pop_front from an empty deque")
        bind v = r["buf"][r["head"]]
        r["buf"][r["head"]] = null
        r["head"] = (r["head"] + 1) % r["cap"]
        r["count"] = r["count"] - 1
        return v
    }
    spell pop_back() {
        assert(r["count"] > 0, "Candy: pop_back from an empty deque")
        bind at = (r["head"] + r["count"] - 1) % r["cap"]
        bind v = r["buf"][at]
        r["buf"][at] = null
        r["count"] = r["count"] - 1
        return v
    }
    spell front() {
        assert(r["count"] > 0, "Candy: front of an empty deque")
        return r["buf"][r["head"]]
    }
    spell back() {
        assert(r["count"] > 0, "Candy: back of an empty deque")
        return r["buf"][(r["head"] + r["count"] - 1) % r["cap"]]
    }
    spell size() {
        return r["count"]
    }
    spell is_empty() {
        return r["count"] == 0
    }
    spell contains(v) {
        for i in range(0, r["count"]) {
            if r["buf"][(r["head"] + i) % r["cap"]] == v {
                return On
            }
        }
        return Off
    }
    spell to_list() {
        bind out = []
        for i in range(0, r["count"]) {
            append(out, r["buf"][(r["head"] + i) % r["cap"]])
        }
        return out
    }
    spell clear() {
        r = _ring_new(8)
        return null
    }
    return {"kind": "candy/deque", "push_front": push_front, "push_back": push_back,
            "pop_front": pop_front, "pop_back": pop_back, "front": front,
            "back": back, "size": size, "is_empty": is_empty,
            "contains": contains, "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? CircularQueue — fixed-capacity queue that wraps around
? ---------------------------------------------------------------------------

spell circular_queue(cap) {
    assert(type_of(cap) == "integer" and cap >= 1,
           "Candy: capacity must be an integer of at least 1")
    bind r = _ring_new(cap)
    spell enqueue(v) {
        assert(r["count"] < r["cap"], "Candy: enqueue into a full circular queue")
        r["buf"][(r["head"] + r["count"]) % r["cap"]] = v
        r["count"] = r["count"] + 1
        return null
    }
    spell dequeue() {
        assert(r["count"] > 0, "Candy: dequeue from an empty circular queue")
        bind v = r["buf"][r["head"]]
        r["buf"][r["head"]] = null
        r["head"] = (r["head"] + 1) % r["cap"]
        r["count"] = r["count"] - 1
        return v
    }
    spell front() {
        assert(r["count"] > 0, "Candy: front of an empty circular queue")
        return r["buf"][r["head"]]
    }
    spell size() {
        return r["count"]
    }
    spell capacity() {
        return r["cap"]
    }
    spell is_empty() {
        return r["count"] == 0
    }
    spell is_full() {
        return r["count"] == r["cap"]
    }
    spell to_list() {
        bind out = []
        for i in range(0, r["count"]) {
            append(out, r["buf"][(r["head"] + i) % r["cap"]])
        }
        return out
    }
    spell clear() {
        r = _ring_new(cap)
        return null
    }
    return {"kind": "candy/circular-queue", "enqueue": enqueue, "dequeue": dequeue,
            "front": front, "size": size, "capacity": capacity,
            "is_empty": is_empty, "is_full": is_full, "to_list": to_list,
            "clear": clear}
}

? ---------------------------------------------------------------------------
? PriorityQueue — binary heap of [priority, seq, value]; stable on equal
? priorities (earlier push leaves first). Min-queue by default, pass On for
? a max-queue. Priorities must all be numbers, or all be texts.
? ---------------------------------------------------------------------------

spell priority_queue(descending = Off) {
    bind heap = []
    bind seq = 0
    bind list_pop = pop
    spell less(a, b) {
        if a[0] != b[0] {
            if descending {
                return a[0] > b[0]
            }
            return a[0] < b[0]
        }
        return a[1] < b[1]
    }
    spell sift_up(i) {
        bind going = On
        while going {
            if i == 0 {
                going = Off
            } else {
                bind p = (i - 1) // 2
                if less(heap[i], heap[p]) {
                    bind t = heap[i]
                    heap[i] = heap[p]
                    heap[p] = t
                    i = p
                } else {
                    going = Off
                }
            }
        }
        return null
    }
    spell sift_down(i) {
        bind n = length(heap)
        bind going = On
        while going {
            bind left = i * 2 + 1
            bind right = left + 1
            bind best = i
            if left < n and less(heap[left], heap[best]) {
                best = left
            }
            if right < n and less(heap[right], heap[best]) {
                best = right
            }
            if best == i {
                going = Off
            } else {
                bind t = heap[i]
                heap[i] = heap[best]
                heap[best] = t
                i = best
            }
        }
        return null
    }
    spell push(v, priority) {
        seq = seq + 1
        append(heap, [priority, seq, v])
        sift_up(length(heap) - 1)
        return null
    }
    spell pop() {
        assert(length(heap) > 0, "Candy: pop from an empty priority queue")
        bind result = heap[0][2]
        bind last = list_pop(heap)
        if length(heap) > 0 {
            heap[0] = last
            sift_down(0)
        }
        return result
    }
    spell peek() {
        assert(length(heap) > 0, "Candy: peek at an empty priority queue")
        return heap[0][2]
    }
    spell size() {
        return length(heap)
    }
    spell is_empty() {
        return length(heap) == 0
    }
    spell to_list() {
        ? all [value, priority] pairs in pop order, without consuming
        bind triples = copy(heap)
        for i in range(1, length(triples)) {
            bind cur = triples[i]
            bind j = i - 1
            while j >= 0 and less(cur, triples[j]) {
                triples[j + 1] = triples[j]
                j = j - 1
            }
            triples[j + 1] = cur
        }
        bind out = []
        for t in triples {
            append(out, [t[2], t[0]])
        }
        return out
    }
    spell clear() {
        heap = []
        seq = 0
        return null
    }
    return {"kind": "candy/priority-queue", "push": push, "pop": pop,
            "peek": peek, "size": size, "is_empty": is_empty,
            "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? MonotonicStack — keeps values strictly monotonic from bottom to top.
? push(v) removes every value that breaks the order and returns them as a
? list (oldest first); the classic "next greater element" helper.
? Default: strictly decreasing (increasing = On gives strictly increasing).
? ---------------------------------------------------------------------------

spell monotonic_stack(increasing = Off) {
    bind items = []
    bind list_pop = pop
    spell push(v) {
        bind dropped = []
        while length(items) > 0 {
            bind top = items[length(items) - 1]
            if increasing {
                if top < v {
                    append(dropped, list_pop(items))
                    continue
                }
            } else {
                if top <= v {
                    append(dropped, list_pop(items))
                    continue
                }
            }
            break
        }
        append(items, v)
        return dropped
    }
    spell pop() {
        assert(length(items) > 0, "Candy: pop from an empty monotonic stack")
        return list_pop(items)
    }
    spell peek() {
        assert(length(items) > 0, "Candy: peek at an empty monotonic stack")
        return items[length(items) - 1]
    }
    spell size() {
        return length(items)
    }
    spell is_empty() {
        return length(items) == 0
    }
    spell to_list() {
        return copy(items)
    }
    spell clear() {
        items = []
        return null
    }
    return {"kind": "candy/monotonic-stack", "push": push, "pop": pop,
            "peek": peek, "size": size, "is_empty": is_empty,
            "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? MonotonicQueue — front is always the largest value seen in the current
? window (or the smallest when descending = On). push(v) drops dominated
? values from the back; expire(v) removes the front when a window element
? leaves. Values must be comparable (all numbers or all texts).
? ---------------------------------------------------------------------------

spell monotonic_queue(descending = Off) {
    bind items = []
    bind head = 0
    spell push(v) {
        while length(items) > head {
            bind back = items[length(items) - 1]
            if descending {
                if back > v {
                    pop(items)
                    continue
                }
            } else {
                if back < v {
                    pop(items)
                    continue
                }
            }
            break
        }
        append(items, v)
        return null
    }
    spell pop_front() {
        assert(head < length(items), "Candy: pop_front from an empty monotonic queue")
        bind v = items[head]
        head = head + 1
        if head == length(items) {
            items = []
            head = 0
        }
        return v
    }
    spell front() {
        assert(head < length(items), "Candy: front of an empty monotonic queue")
        return items[head]
    }
    spell back() {
        assert(head < length(items), "Candy: back of an empty monotonic queue")
        return items[length(items) - 1]
    }
    spell expire(v) {
        if head < length(items) and items[head] == v {
            pop_front()
            return On
        }
        return Off
    }
    spell size() {
        return length(items) - head
    }
    spell is_empty() {
        return head == length(items)
    }
    spell to_list() {
        bind out = []
        for i in range(head, length(items)) {
            append(out, items[i])
        }
        return out
    }
    spell clear() {
        items = []
        head = 0
        return null
    }
    return {"kind": "candy/monotonic-queue", "push": push, "pop_front": pop_front,
            "front": front, "back": back, "expire": expire, "size": size,
            "is_empty": is_empty, "to_list": to_list, "clear": clear}
}

? ---------------------------------------------------------------------------
? MinStack / MaxStack — stacks whose extreme value is O(1)
? ---------------------------------------------------------------------------

spell _extreme_stack(track_max) {
    bind items = []
    bind list_pop = pop
    spell push(v) {
        if length(items) == 0 {
            append(items, [v, v])
        } else {
            bind best = items[length(items) - 1][1]
            if track_max {
                if v > best {
                    best = v
                }
            } else {
                if v < best {
                    best = v
                }
            }
            append(items, [v, best])
        }
        return null
    }
    spell pop() {
        assert(length(items) > 0, "Candy: pop from an empty stack")
        return list_pop(items)[0]
    }
    spell peek() {
        assert(length(items) > 0, "Candy: peek at an empty stack")
        return items[length(items) - 1][0]
    }
    spell extreme() {
        assert(length(items) > 0, "Candy: extreme of an empty stack")
        return items[length(items) - 1][1]
    }
    spell size() {
        return length(items)
    }
    spell is_empty() {
        return length(items) == 0
    }
    spell to_list() {
        bind out = []
        for pair in items {
            append(out, pair[0])
        }
        return out
    }
    spell clear() {
        items = []
        return null
    }
    return {"kind": "candy/extreme-stack", "push": push, "pop": pop,
            "peek": peek, "extreme": extreme, "size": size,
            "is_empty": is_empty, "to_list": to_list, "clear": clear}
}

spell min_stack() {
    bind s = _extreme_stack(Off)
    return {"kind": "candy/min-stack", "push": s["push"], "pop": s["pop"],
            "peek": s["peek"], "min": s["extreme"], "size": s["size"],
            "is_empty": s["is_empty"], "to_list": s["to_list"],
            "clear": s["clear"]}
}

spell max_stack() {
    bind s = _extreme_stack(On)
    return {"kind": "candy/max-stack", "push": s["push"], "pop": s["pop"],
            "peek": s["peek"], "max": s["extreme"], "size": s["size"],
            "is_empty": s["is_empty"], "to_list": s["to_list"],
            "clear": s["clear"]}
}
