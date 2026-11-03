# Candy 01 — `structures`

Linear data structures for Arcana: stacks, queues, deques, a fixed-capacity
ring, a priority queue, monotonic helpers and extreme-tracking stacks.

```arc
import candy.structures

spell main() {
    bind s = candy.structures.stack()
    s["push"](1)
    s["push"](2)
    cast s["pop"]()                    ? 2
}
```

## Importing

| Your file sits...                        | Import                          |
|------------------------------------------|---------------------------------|
| in the `arcana` project root             | `import candy.structures`       |
| next to a `candy/` folder                | `import candy.structures`       |
| in a folder on `ARCANA_PATH`'s parent... | see below                       |
| next to the module itself                | `import structures`             |

```arc
import candy.structures            ? full name
import candy.structures as st      ? aliased
from candy.structures import stack, queue
import candy                       ? then candy.structures.stack()
```

`import candy` (the aggregator grimoire `candy.arc`, kept next to the `candy/`
folder) exposes every Tier I module: `candy.structures`, `candy.trees`,
`candy.collections`, `candy.graphs`.

## Conventions shared by every Candy structure

1. **Structures are closure objects.** A constructor such as `stack()`
   returns a map of spells sharing private state. Call methods by indexing:
   `s["push"](7)`. There are no classes in Arcana 1.0.
2. **Every instance has a `"kind"` tag** — a text like `"candy/stack"` — so
   programs can tell structures apart: `s["kind"]`.
3. **Misuse raises catchable `AssertionError` (E2101)** with a message that
   starts with `Candy:` (popping an empty structure, enqueueing into a full
   circular queue, a non-integer capacity, ...). Catch them with `try`/`catch`
   and inspect `e["message"]`.
4. **`size()`, `is_empty()` and `to_list()`** exist wherever they make sense.
   `to_list()` always returns a fresh list (a snapshot), never the internal one.
5. **Mutate-in-place vs. return**: methods that change the structure return
   `null`; queries return values.
6. **Comparability**: wherever order matters (priority queue, monotonic
   helpers), values must be all numbers or all texts, exactly like `list.sort`.

---

## Stack

`stack()` — last in, first out. All methods O(1).

| Method | Returns | Notes |
|---|---|---|
| `push(v)` | null | put `v` on top |
| `pop()` | the top value | AssertionError when empty |
| `peek()` | the top value | without removing |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `contains(v)` | boolean | `v in items` |
| `to_list()` | list | bottom … top |
| `clear()` | null | |

```arc
bind s = stack()
s["push"](1)
s["push"](2)
cast s["peek"]()      ? 2
cast s["pop"]()       ? 2
cast s["to_list"]()   ? [1]
```

## Queue

`queue()` — first in, first out. `enqueue`/`dequeue` are amortised O(1):
the queue keeps a head index and compacts the backing list once it is more
than half consumed.

| Method | Returns | Notes |
|---|---|---|
| `enqueue(v)` | null | add at the back |
| `dequeue()` | the front value | AssertionError when empty |
| `front()` | the front value | without removing |
| `back()` | the back value | |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `contains(v)` | boolean | scans live items only |
| `to_list()` | list | front … back |
| `clear()` | null | |

## Deque

`deque()` — double-ended queue on a growable ring buffer. All methods
amortised O(1); the ring doubles when full.

| Method | Returns | Notes |
|---|---|---|
| `push_front(v)` | null | |
| `push_back(v)` | null | |
| `pop_front()` | value | AssertionError when empty |
| `pop_back()` | value | AssertionError when empty |
| `front()` | value | |
| `back()` | value | |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `contains(v)` | boolean | |
| `to_list()` | list | front … back |
| `clear()` | null | |

## CircularQueue

`circular_queue(cap)` — fixed-capacity ring; `cap` must be an integer of
at least 1 (otherwise AssertionError at construction). Enqueueing into a full
ring raises AssertionError; nothing is silently dropped.

| Method | Returns | Notes |
|---|---|---|
| `enqueue(v)` | null | AssertionError when full |
| `dequeue()` | value | AssertionError when empty |
| `front()` | value | |
| `size()` | integer | |
| `capacity()` | integer | the fixed cap |
| `is_empty()` | boolean | |
| `is_full()` | boolean | |
| `to_list()` | list | oldest … newest |
| `clear()` | null | |

## PriorityQueue

`priority_queue(descending = Off)` — binary min-heap of
`[priority, seq, value]` entries. Ties are stable: among equal priorities, the
value pushed first leaves first. Pass `On` for a max-queue. Push/pop/peek are
O(log n).

Priorities must be comparable: all numbers, or all texts. Values may be
anything.

| Method | Returns | Notes |
|---|---|---|
| `push(v, priority)` | null | |
| `pop()` | value | the highest-priority value; AssertionError when empty |
| `peek()` | value | without removing |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `to_list()` | list of `[value, priority]` | in pop order, without consuming |
| `clear()` | null | |

```arc
bind pq = priority_queue()
pq["push"]("email", 3)
pq["push"]("alarm", 1)
pq["push"]("note", 2)
cast pq["pop"]()      ? alarm
cast pq["pop"]()      ? note
cast pq["pop"]()      ? email
```

## MonotonicStack

`monotonic_stack(increasing = Off)` — keeps values strictly decreasing
from bottom to top (pass `On` for strictly increasing). `push(v)` removes and
**returns as a list** every value that would break the monotonic order — the
classic "next greater element" helper.

| Method | Returns | Notes |
|---|---|---|
| `push(v)` | list | the popped (dominated) values, oldest first |
| `pop()` | value | AssertionError when empty |
| `peek()` | value | |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `to_list()` | list | bottom … top |
| `clear()` | null | |

```arc
? next greater element
bind ms = monotonic_stack()
bind a = [4, 5, 2, 25]
bind nge = []
for x in a {
    bind dropped = ms["push"](x)
    for d in dropped { append(nge, [d, x]) }
}
? nge is [[4, 5], [2, 25]]
```

## MonotonicQueue

`monotonic_queue(descending = Off)` — the front is always the **largest**
value of the current window (`descending = On`: the smallest). `push` drops
dominated values from the back; `expire(v)` drops the front when the window
slides past it. Sliding-window maximum in amortised O(1) per element.

| Method | Returns | Notes |
|---|---|---|
| `push(v)` | null | |
| `pop_front()` | value | AssertionError when empty |
| `front()` | value | the window maximum (minimum with `On`) |
| `back()` | value | |
| `expire(v)` | boolean | removes the front iff it equals `v` |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `to_list()` | list | front … back |
| `clear()` | null | |

```arc
? sliding window maximum, window 3
bind mq = monotonic_queue()
bind a = [1, 3, -1, -3, 5, 3, 6, 7]
bind wins = []
for i in range(0, length(a)) {
    mq["push"](a[i])
    if i >= 3 { mq["expire"](a[i - 3]) }
    if i >= 2 { append(wins, mq["front"]()) }
}
? wins is [3, 3, 5, 5, 6, 7]
```

## MinStack / MaxStack

`min_stack()` and `max_stack()` — a stack that remembers the
smallest (largest) value of its remaining contents. Every method O(1).

| Method | Returns | Notes |
|---|---|---|
| `push(v)` | null | |
| `pop()` | value | AssertionError when empty |
| `peek()` | value | |
| `min()` / `max()` | value | `min()` on MinStack, `max()` on MaxStack |
| `size()` | integer | |
| `is_empty()` | boolean | |
| `to_list()` | list | bottom … top |
| `clear()` | null | |

## Complexity summary

| Structure | push | pop | peek/front | notes |
|---|---|---|---|---|
| Stack | O(1) | O(1) | O(1) | |
| Queue | O(1) | O(1) amortised | O(1) | compacts backing list |
| Deque | O(1) amortised | O(1) amortised | O(1) | ring doubles when full |
| CircularQueue | O(1) | O(1) | O(1) | fixed capacity |
| PriorityQueue | O(log n) | O(log n) | O(1) | stable ties |
| MonotonicStack | amortised O(1) | O(1) | O(1) | push returns popped values |
| MonotonicQueue | amortised O(1) | O(1) | O(1) | window helpers |
| Min/MaxStack | O(1) | O(1) | O(1) | |

## Error behaviour

Every misuse raises catchable `E2101` (`AssertionError`) with a `Candy: ...`
message:

* `pop`/`peek`/`front`/`back`/`dequeue`/`min`/`max` on an empty structure;
* `enqueue` into a full circular queue;
* `circular_queue` with a capacity below 1 or of non-integer type.

Arity and type mistakes in *your* call (`s["push"]()` with no argument,
`s["size"](1)`) are ordinary Arcana errors (`E3004`/`E3003`) and are not
catchable — they are programmer errors, exactly like the core language treats
them.
