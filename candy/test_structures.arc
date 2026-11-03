? Candy test: structures — run with  arcana run candy/test_structures.arc
import structures

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

spell main() {
    ? ----- Stack -----
    bind s = structures.stack()
    assert(s["kind"] == "candy/stack", "stack kind")
    assert(s["is_empty"](), "stack starts empty")
    s["push"](1)
    s["push"](2)
    s["push"](3)
    assert(s["size"]() == 3, "stack size")
    assert(s["peek"]() == 3, "stack peek")
    assert(s["pop"]() == 3, "stack pop 3")
    assert(s["pop"]() == 2, "stack pop 2")
    assert(s["to_list"]() == [1], "stack to_list")
    assert(s["contains"](1), "stack contains 1")
    assert(not s["contains"](9), "stack not contains 9")
    assert(s["pop"]() == 1, "stack pop 1")
    spell pop_empty_stack() {
        s["pop"]()
    }
    expect_error(pop_empty_stack, "empty")

    ? ----- Queue (incl. compaction path) -----
    bind q = structures.queue()
    assert(q["kind"] == "candy/queue", "queue kind")
    for i in range(0, 200) {
        q["enqueue"](i)
    }
    for i in range(0, 150) {
        assert(q["dequeue"]() == i, "queue fifo order at " + to_text(i))
    }
    assert(q["size"]() == 50, "queue size after churn")
    bind want150 = []
    for i in range(150, 200) {
        append(want150, i)
    }
    assert(q["to_list"]() == want150, "queue to_list after compaction")
    assert(q["front"]() == 150, "queue front")
    assert(q["back"]() == 199, "queue back")
    assert(q["contains"](199) and not q["contains"](5), "queue contains")
    for i in range(0, 50) {
        q["dequeue"]()
    }
    assert(q["is_empty"](), "queue empty after drain")
    spell dequeue_empty() {
        q["dequeue"]()
    }
    spell front_empty() {
        q["front"]()
    }
    expect_error(dequeue_empty, "empty")
    expect_error(front_empty, "empty")

    ? ----- Deque (ring wrap-around and growth) -----
    bind d = structures.deque()
    assert(d["kind"] == "candy/deque", "deque kind")
    for i in range(0, 6) {
        d["push_back"](i)
    }
    d["push_front"](-1)
    d["push_front"](-2)
    assert(d["size"]() == 8, "deque size")
    assert(d["to_list"]() == [-2, -1, 0, 1, 2, 3, 4, 5], "deque order after wrap")
    assert(d["front"]() == -2 and d["back"]() == 5, "deque ends")
    d["push_back"](6)
    d["push_front"](-3)
    assert(d["size"]() == 10, "deque grew past capacity")
    assert(d["to_list"]() == [-3, -2, -1, 0, 1, 2, 3, 4, 5, 6], "deque order after grow")
    assert(d["pop_front"]() == -3, "deque pop_front")
    assert(d["pop_back"]() == 6, "deque pop_back")
    assert(d["contains"](0) and not d["contains"](6), "deque contains")
    while not d["is_empty"]() {
        d["pop_front"]()
    }
    spell pop_front_empty() {
        d["pop_front"]()
    }
    spell pop_back_empty() {
        d["pop_back"]()
    }
    expect_error(pop_front_empty, "empty")
    expect_error(pop_back_empty, "empty")

    ? ----- CircularQueue -----
    bind c = structures.circular_queue(3)
    assert(c["kind"] == "candy/circular-queue", "circular kind")
    assert(c["capacity"]() == 3 and c["is_empty"]() and not c["is_full"](), "circular state")
    c["enqueue"]("a")
    c["enqueue"]("b")
    c["enqueue"]("c")
    assert(c["is_full"](), "circular full")
    spell enqueue_full() {
        c["enqueue"]("x")
    }
    expect_error(enqueue_full, "full")
    assert(c["dequeue"]() == "a", "circular dequeue a")
    c["enqueue"]("d")
    assert(c["to_list"]() == ["b", "c", "d"], "circular wraps around")
    assert(c["dequeue"]() == "b" and c["dequeue"]() == "c" and c["dequeue"]() == "d", "circular drain")
    assert(c["is_empty"](), "circular empty")
    bind saw_bad_capacity = Off
    try {
        structures.circular_queue(0)
    } catch e {
        saw_bad_capacity = On
        assert("capacity" in e["message"], "capacity error message")
    }
    assert(saw_bad_capacity, "capacity 0 rejected")

    ? ----- PriorityQueue -----
    bind pq = structures.priority_queue()
    assert(pq["kind"] == "candy/priority-queue", "pq kind")
    pq["push"]("low", 5)
    pq["push"]("top", 1)
    pq["push"]("mid", 3)
    pq["push"]("also-low", 5)
    assert(pq["peek"]() == "top", "pq peek min")
    assert(pq["size"]() == 4, "pq size")
    assert(pq["to_list"]() == [["top", 1], ["mid", 3], ["low", 5], ["also-low", 5]],
           "pq to_list pop order, stable on ties")
    assert(pq["pop"]() == "top", "pq pop top")
    assert(pq["pop"]() == "mid", "pq pop mid")
    assert(pq["pop"]() == "low", "pq stable tie: earlier push leaves first")
    assert(pq["pop"]() == "also-low", "pq last")
    assert(pq["is_empty"](), "pq empty")
    spell pq_pop_empty() {
        pq["pop"]()
    }
    expect_error(pq_pop_empty, "empty")
    bind pqmax = structures.priority_queue(On)
    pqmax["push"](1, 10)
    pqmax["push"](2, 20)
    pqmax["push"](3, 15)
    assert(pqmax["pop"]() == 2, "max priority queue")
    assert(pqmax["pop"]() == 3, "max priority queue second")

    ? ----- MonotonicStack (next greater element) -----
    bind ms = structures.monotonic_stack()
    assert(ms["kind"] == "candy/monotonic-stack", "ms kind")
    assert(ms["push"](4) == [], "4 pops nothing")
    assert(ms["push"](5) == [4], "5 pops 4: nge(4) = 5")
    assert(ms["push"](2) == [], "2 pops nothing")
    assert(ms["push"](25) == [2, 5], "25 pops 2 then 5")
    assert(ms["to_list"]() == [25], "ms contents")
    assert(ms["peek"]() == 25 and ms["pop"]() == 25 and ms["is_empty"](), "ms drain")
    bind mi = structures.monotonic_stack(On)
    assert(mi["push"](9) == [], "increasing push 9")
    assert(mi["push"](3) == [], "increasing push 3")
    assert(mi["push"](7) == [3], "increasing: 7 pops 3")
    assert(mi["to_list"]() == [9, 7], "increasing contents")

    ? ----- MonotonicQueue (sliding window maximum) -----
    bind mq = structures.monotonic_queue()
    assert(mq["kind"] == "candy/monotonic-queue", "mq kind")
    bind a = [1, 3, -1, -3, 5, 3, 6, 7]
    bind w = 3
    bind want = [3, 3, 5, 5, 6, 7]
    bind got = []
    for i in range(0, length(a)) {
        mq["push"](a[i])
        if i >= w {
            mq["expire"](a[i - w])
        }
        if i >= w - 1 {
            append(got, mq["front"]())
        }
    }
    assert(got == want, "sliding window max: " + to_text(got))
    bind mq_min = structures.monotonic_queue(On)
    for i in range(0, 4) {
        mq_min["push"](a[i])
    }
    assert(mq_min["front"]() == -3, "min variant front")

    ? ----- MinStack / MaxStack -----
    bind mns = structures.min_stack()
    assert(mns["kind"] == "candy/min-stack", "min stack kind")
    mns["push"](5)
    mns["push"](2)
    mns["push"](7)
    assert(mns["min"]() == 2, "min is 2")
    assert(mns["pop"]() == 7, "pop 7")
    assert(mns["min"]() == 2, "min still 2")
    assert(mns["pop"]() == 2, "pop 2")
    assert(mns["min"]() == 5, "min now 5")
    assert(mns["to_list"]() == [5], "min stack to_list")
    assert(mns["pop"]() == 5, "min stack drains")
    bind mxs = structures.max_stack()
    mxs["push"](5)
    mxs["push"](9)
    mxs["push"](-1)
    assert(mxs["max"]() == 9, "max is 9")
    assert(mxs["pop"]() == -1, "pop -1")
    assert(mxs["max"]() == 9, "max still 9")
    assert(mxs["pop"]() == 9, "pop 9")
    assert(mxs["max"]() == 5, "max now 5")
    spell min_of_empty() {
        mns["min"]()
    }
    expect_error(min_of_empty, "empty")

    cast "ALL STRUCTURES TESTS PASSED"
}
