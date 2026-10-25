? Closures and default arguments
spell make_counter(start = 0, step = 1) {
    bind value = start
    spell next() {
        value = value + step
        return value
    }
    return next
}

spell main() {
    bind a = make_counter()
    bind b = make_counter(100, 10)
    a()
    a()
    cast a()      ? 3
    cast b()      ? 110
    cast b()      ? 120
}
