# 6. Cookbook

Complete programs for common tasks. Each one lives in `examples/` and can be run
with `arcana run examples/<name>.arc`.

## FizzBuzz: loops and strict conditions

`examples/fizzbuzz.arc`

```arc
spell main() {
    for n in range(1, 16) {
        if n % 15 == 0 {
            cast "FizzBuzz"
        } else if n % 3 == 0 {
            cast "Fizz"
        } else if n % 5 == 0 {
            cast "Buzz"
        } else {
            cast n
        }
    }
}
```

Output starts `1`, `2`, `Fizz`, `4`, `Buzz`, ... and ends with `FizzBuzz`.

## Inventory: lists, maps and text together

`examples/inventory.arc`

```arc
spell describe(item) {
    return upper(item["name"]) + " x" + text(item["qty"])
}

spell main() {
    bind inventory = [
        {"name": "potion", "qty": 3},
        {"name": "scroll", "qty": 12},
        {"name": "wand", "qty": 1},
    ]

    bind total = 0
    for item in inventory {
        cast describe(item)
        total = total + item["qty"]
    }
    cast "Total items: " + text(total)

    bind names = []
    for item in inventory { append(names, item["name"]) }
    cast join(names, ", ")

    bind stock = {}
    for item in inventory { stock[item["name"]] = item["qty"] }
    cast stock
    cast "wand" in stock
    cast keys(stock)
}
```

```
POTION x3
SCROLL x12
WAND x1
Total items: 16
potion, scroll, wand
{"potion": 3, "scroll": 12, "wand": 1}
On
["potion", "scroll", "wand"]
```

## Counting words with a map

`examples/wordcount.arc`

```arc
spell main() {
    bind sentence = "the wand and the staff and the scroll"
    bind counts = {}
    for word in split(sentence) {
        if word in counts {
            counts[word] = counts[word] + 1
        } else {
            counts[word] = 1
        }
    }
    for word in keys(counts) {
        cast word + ": " + text(counts[word])
    }
}
```

```
the: 3
wand: 1
and: 2
staff: 1
scroll: 1
```

Pattern: test with `key in map` before reading, because reading a missing key is
an error.

## Closures and default arguments

`examples/closures.arc`

```arc
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
    cast a()
    cast b()
    cast b()
}
```

```
3
110
120
```

Each call to `make_counter` creates its own `value`, so `a` and `b` are independent.

## Big numbers and text reversal

`examples/factorial.arc`

```arc
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
```

```
15511210043330985984000000
1267650600228229401496703205376
anacra
```

## A guessing game: input, `try`/`catch`, `while`, `break`

`examples/guess.arc`

```arc
spell main() {
    bind secret = 7
    bind tries = 0
    while On {
        try {
            bind guess = ask_integer("Guess the number (1-10): ")
            tries = tries + 1
            if guess == secret {
                cast "Correct! It took " + text(tries) + " tries."
                break
            } else if guess < secret {
                cast "Too low."
            } else {
                cast "Too high."
            }
        } catch err {
            cast "That was not a whole number (" + err["code"] + ")."
        }
    }
}
```

Try it without typing, by piping answers:

```
printf 'abc\n3\n9\n7\n' | arcana run examples/guess.arc
```

If the input runs out before a correct guess, `ask_integer` raises `E2010`, which
this loop catches too, so the loop would never end. When reading from a pipe,
always provide the right answer, or use `--max-steps`.

## Reading numbers until a blank line

`examples/sum_input.arc`

```arc
spell main() {
    bind total = 0
    bind numbers = 0
    while On {
        bind line = ask("number (blank to stop): ")
        if line == "" {
            break
        }
        try {
            total = total + integer(line)
            numbers = numbers + 1
        } catch err {
            cast "skipping " + line + ": " + err["message"]
        }
    }
    cast "sum of " + text(numbers) + " numbers = " + text(total)
}
```

```
printf '10\n20\nabc\n5\n\n' | arcana run examples/sum_input.arc
```

ends with `skipping abc: Cannot convert "abc" to integer` and
`sum of 3 numbers = 35`.

## Safe division and your own errors

`examples/safe_math.arc`

```arc
spell safe_divide(a, b) {
    try {
        return a / b
    } catch err {
        cast "Caught " + err["kind"] + " (" + err["code"] + "): " + err["message"]
        return null
    } finally {
        cast "division attempted"
    }
}

spell withdraw(balance, amount) {
    if amount > balance {
        fail("insufficient mana")
    }
    return balance - amount
}

spell main() {
    cast safe_divide(9, 3)
    cast safe_divide(1, 0)
    try {
        cast withdraw(50, 80)
    } catch e {
        cast e["kind"] + ": " + e["message"]
    }
}
```

```
division attempted
3.0
Caught MathError (E2001): Division by zero
division attempted
null
UserError: insufficient mana
```

Note that `finally` ran in both calls even though each `try` block ended with
`return`.

## Sharing code between files

`examples/lib.arc` and `examples/imports.arc`

```arc
? lib.arc
bind version = "1.0"

spell double(x) {
    return x * 2
}
```

```arc
? imports.arc (next to lib.arc)
import lib
from lib import double

spell main() {
    cast lib.version
    cast lib.double(21)
    cast double(4)
}
```

```
1.0
42
8
```

## Small recipes

Each snippet goes inside `spell main() { ... }`.

**Sum a list**

```
bind total = 0
for n in [4, 8, 15] { total = total + n }
```

**Largest item without `max`**

```
bind best = items[0]
for x in items { if x > best { best = x } }
```

**Last item of a list** (no negative indexes)

```
bind last = items[length(items) - 1]
```

**Repeat text n times**

```
bind line = ""
for i in range(10) { line = line + "-" }
```

**Check for an empty list or text**

```
if length(items) == 0 { cast "nothing here" }
```

**Clamp a number to a range**

```
bind clamped = min(max(x, 0), 100)
```

**Loop with an index**

```
for i in range(length(items)) {
    cast text(i) + ": " + text(items[i])
}
```

**Safe lookup with a default**

```
bind value = 0
if "mana" in hero { value = hero["mana"] }
```

**Floor, round and convert**

```
cast 7 // 2             ? 3    integer division
cast round(2.5)         ? 3
cast integer("42") + 1  ? 43
cast decimal(1) / 3     ? 0.3333333333333333
```
