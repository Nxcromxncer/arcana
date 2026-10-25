? Functions, recursion, loops, collections
spell add(a, b) {
    return a + b
}

spell fib(n) {
    if n < 2 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}

spell greet(name, title = "Arcanist") {
    return "Welcome, " + title + " " + name
}

spell main() {
    cast add(5, 7)
    cast greet("Mira")
    cast greet("Orin", "Archmage")

    for i in range(1, 6) {
        cast fib(i)
    }

    bind spells = ["fireball", "frost", "ward"]
    append(spells, "teleport")
    for s in spells {
        cast upper(s)
    }

    bind grimoire = {"mana": 100, "level": 3}
    grimoire["mana"] = grimoire["mana"] - 25
    cast grimoire
    cast "level" in grimoire

    bind total = 0
    bind n = 0
    while On {
        n = n + 1
        if n % 2 == 0 {
            continue
        }
        if n > 9 {
            break
        }
        total = total + n
    }
    cast total
}
