? Integers have arbitrary precision: no overflow, ever
spell factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

spell reverse(s) {
    bind out = ""
    for ch in s {
        out = ch + out
    }
    return out
}

spell main() {
    cast factorial(25)
    cast 2 ** 100
    cast reverse("arcana")
}
