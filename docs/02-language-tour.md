# 2. Language tour

A guided walk through Arcana. Every snippet here is a complete program unless it
says otherwise; paste it into a file and `arcana run` it.

## Output and comments

```arc
? This whole line is a comment
spell main() {
    cast "Hello"          ? a comment after code
    cast 1 + 2
    cast "Is that all? Yes!"    ? a ? inside text is just a character
}
```

```
Hello
3
Is that all? Yes!
```

`cast` prints any value. Text prints without quotes; numbers, booleans, lists and
maps print in their natural form.

## Variables: `bind`

`bind` declares a variable. Assigning to it later uses plain `=`.

```arc
spell main() {
    bind mana = 80
    bind cost = 25
    mana = mana - cost
    cast mana
}
```

```
55
```

Rules:

* Declare once per scope. `bind x = 1` followed by `bind x = 2` in the same
  block is error `E3002`. To change a value, assign: `x = 2`.
* Assigning to a name that was never declared is error `E1001`.
* Names are case-sensitive and may contain letters, digits and `_`.

## Types

| Type      | Examples                         | Notes                                        |
|-----------|----------------------------------|----------------------------------------------|
| integer   | `42`, `1_000_000`, `-7`          | Any size. No overflow.                       |
| decimal   | `2.5`, `1e3`, `0.25`             | 64-bit floating point.                       |
| text      | `"hello"`, `"line\nbreak"`       | Immutable. Double quotes only.               |
| boolean   | `On`, `Off`                      | Capital letters are required.                |
| null      | `null`                           | "no value"; what a spell returns by default. |
| list      | `[1, "two", On]`                 | Ordered, mutable, any mix of types.          |
| map       | `{"mana": 100, "level": 3}`      | Ordered by insertion; keys are text or integer. |
| range     | `range(1, 4)`                    | Produced by `range(...)`; used in loops.     |
| spell     | `main`, `length`                 | Spells are values and can be passed around.  |
| grimoire  | `lib` (after `import lib`)       | An imported file.                            |

Ask a value for its type with `type_of(x)`:

```arc
spell main() {
    cast type_of(1)
    cast type_of(1.0)
    cast type_of("s")
    cast type_of(On)
    cast type_of(null)
    cast type_of([])
    cast type_of({})
}
```

```
integer
decimal
text
boolean
null
list
map
```

## Arithmetic

```arc
spell main() {
    cast 7 / 2        ? 3.5   division always gives a decimal
    cast 6 / 3        ? 2.0
    cast 7 // 2       ? 3     floor division
    cast -7 // 2      ? -4    floors toward negative infinity
    cast -7 % 3       ? 2     remainder takes the sign of the divisor
    cast 2 ** 10      ? 1024  power
    cast 2 ** 100     ? 1267650600228229401496703205376
    cast 0.1 + 0.2    ? 0.30000000000000004  (IEEE 754)
}
```

Precedence, tightest first: `**`, unary `-`, `* / // %`, `+ -`. Power is
right-associative and binds tighter than a minus on its left, so `-2 ** 2` is `-4`
and `2 ** 3 ** 2` is `512`. Use parentheses whenever it helps readability.

`+` also joins text, but **only text with text**:

```arc
spell main() {
    cast "Mana: " + text(55)      ? convert numbers with text()
}
```

`"Mana: " + 55` is error `E3003` with the help "use text(value) to convert".

## Booleans and conditions

Comparison operators: `==  !=  <  <=  >  >=`, plus `in`. Combine booleans with
`and`, `or`, `not` (which short-circuit).

```arc
spell main() {
    bind mana = 80
    bind ready = mana >= 50
    cast ready                          ? On
    cast ready and not (mana > 100)     ? On
}
```

**Conditions must be booleans.** Arcana has no "truthy" values:

```arc-error
spell main() {
    if 1 { cast "yes" }       ? E3001: Condition must be a boolean
}
```

Write the comparison you mean: `x != 0`, `length(list) > 0`, `x != null`.

Equality never converts types: `1 == 1.0` is `On`, but `On == 1` and `"1" == 1`
are `Off`. Lists and maps compare by contents.

Comparisons cannot be chained: write `a < b and b < c`, not `a < b < c`.

## `if` / `else`

```arc
spell main() {
    bind mana = 30
    if mana >= 50 {
        cast "Cast the big spell."
    } else if mana >= 10 {
        cast "Cast a small spell."
    } else {
        cast "Rest."
    }
}
```

```
Cast a small spell.
```

`else` must start on the same line as the closing `}`. Braces are always
required. Variables declared inside a block disappear when the block ends.

## Loops

```arc
spell main() {
    for i in range(1, 4) {      ? 1, 2, 3 (the end is excluded)
        cast i
    }
    for c in "ab" { cast c }    ? text yields one character at a time
    for k in {"x": 1, "y": 2} { cast k }    ? maps yield their keys

    bind n = 0
    while n < 3 {
        n = n + 1
    }
    cast n
}
```

`range(stop)`, `range(start, stop)` and `range(start, stop, step)` are available;
a negative step counts down: `range(3, 0, -1)` gives 3, 2, 1.

`break` leaves the loop and `continue` jumps to the next round:

```arc
spell main() {
    for i in range(10) {
        if i == 2 { continue }
        if i == 4 { break }
        cast i
    }
}
```

```
0
1
3
```

A `for` loop walks a *snapshot* of a list, so appending to the list inside the
loop does not make it endless.

## Spells (functions)

```arc
spell greet(name, title = "Arcanist") {
    return "Welcome, " + title + " " + name
}

spell main() {
    cast greet("Mira")
    cast greet("Orin", "Archmage")
}
```

```
Welcome, Arcanist Mira
Welcome, Archmage Orin
```

* `return` ends the spell and hands back a value; a bare `return` (or reaching the
  end) gives `null`.
* Parameters with defaults must come last. Default expressions are evaluated
  **on every call**, so `spell f(x = [])` gets a fresh list each time.
* Calling with the wrong number of arguments is error `E3004`.
* Spells can be recursive and can call spells declared later in the file.
* Spells are values, and nested spells **close over** the variables around them
  (see `examples/closures.arc`):

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
    cast a()      ? 3
    cast b()      ? 110
    cast b()      ? 120
}
```

## Lists

```arc
spell main() {
    bind spells = ["fireball", "frost", "ward"]
    append(spells, "teleport")
    spells[0] = "inferno"
    cast spells
    cast length(spells)
    cast spells[1]
    cast pop(spells)          ? removes and returns the last item
    cast "ward" in spells
}
```

```
["inferno", "frost", "ward", "teleport"]
4
frost
teleport
On
```

Indexes start at 0. Reading or writing outside the list is error `E2002`
(catchable); negative indexes are **not** allowed.

**Lists and maps are shared, not copied.** Two names can point at the same list:

```arc
spell main() {
    bind a = [1]
    bind b = a
    append(b, 2)
    cast a              ? [1, 2]
    bind c = copy(a)    ? a separate (shallow) copy
    append(c, 3)
    cast a              ? still [1, 2]
}
```

## Maps

```arc
spell main() {
    bind hero = {"mana": 100, "level": 3}
    hero["mana"] = hero["mana"] - 25
    hero["title"] = "Mage"          ? assigning a new key adds it
    cast hero
    cast "level" in hero            ? On
    cast keys(hero)
    cast values(hero)
}
```

```
{"mana": 75, "level": 3, "title": "Mage"}
On
["mana", "level", "title"]
[75, 3, "Mage"]
```

Keys must be text or integers. Reading a missing key is error `E2005`
(catchable), so test first with `key in map`.

## Text

```arc
spell main() {
    cast upper("arcana")                  ? ARCANA
    cast split("a,b,c", ",")              ? ["a", "b", "c"]
    cast join(["x", "y", "z"], "-")       ? x-y-z
    cast length("héllo")                  ? 5 (counts characters)
    cast "abc"[1]                         ? b
    cast "ell" in "hello"                 ? On (substring test)
    cast "tab\there, quote \" and \u{2728}"
}
```

Escapes: `\n` `\t` `\\` `\"` and `\u{hex}` for any Unicode code point. Text is
immutable: `s[0] = "x"` is an error.

## Handling errors: `try` / `catch` / `finally`

```arc
spell main() {
    try {
        cast 1 / 0
    } catch err {
        cast err["code"]        ? E2001
        cast err["kind"]        ? MathError
        cast err["message"]     ? Division by zero
    } finally {
        cast "always runs"
    }
}
```

* The `catch` variable is a map with `code`, `kind` and `message`.
* `finally` runs no matter what, including after `return`, `break` and `continue`.
* You can raise your own error with `fail("message")` (kind `UserError`).
* Only **runtime** errors (`E2xxx`) can be caught. Programmer mistakes such as an
  unknown name or adding text to a number are not catchable.
* `assert(condition, message)` raises a catchable `AssertionError` (`E2101`).

See `examples/safe_math.arc` and [Errors and debugging](05-errors-and-debugging.md).

## Asking for input

```arc
spell main() {
    bind name = ask("Your name: ")
    try {
        bind age = ask_integer("Your age: ")
        cast name + " will be " + text(age + 1) + " next year."
    } catch e {
        cast "That was not a number. (" + e["code"] + ")"
    }
}
```

`ask(prompt)` returns text. `ask_integer` and `ask_decimal` convert the answer and
raise a catchable `E2010` if it is not a number (or if input has ended).

## Splitting code across files

```
import lib                  ? use as lib.name
import tools.maps as m      ? file tools/maps.arc, used as m.name
from lib import double      ? use as double(...)
```

Imports go at the top of the file. A grimoire is loaded only once, even if many
files import it.

## The standard library

Ten ready-made modules cover everyday needs. They import exactly like your own files:

```arc
import math
import text
import list
import date

spell main() {
    cast math.sqrt(144)                          ? 12.0
    cast text.pad_left(to_text(7), 3, "0")       ? 007
    bind scores = [30, 10, 20]
    list.sort(scores)                            ? changes the list in place
    cast scores
    cast list.sum(scores)
    cast date.format(date.make_date(2026, 10, 27), "%A, %d %B %Y")
}
```

```
12.0
007
[10, 20, 30]
60
Tuesday, 27 October 2026
```

| Module    | For |
|-----------|-----|
| `math`    | `sqrt`, `power`, `floor`, `ceil`, `clamp`, trigonometry, `random` |
| `text`    | `trim`, `replace`, `substring`, `find`, `pad_left`, `lines` |
| `list`    | `sort`, `sorted`, `unique`, `slice`, `sum`, `first`, `remove` |
| `logic`   | `is_number`, `is_empty`, `all`, `any` |
| `convert` | `to_binary`, `to_hex`, `parse_number`, `format_number` |
| `error`   | `warn`, `debug`, `assert_equal` |
| `system`  | `args`, `exit`, `env`, `platform`, `clock` |
| `file`    | `read_text`, `write_text`, `read_lines`, `list_dir` |
| `time`    | `now`, `sleep`, `format_duration` |
| `date`    | `make_date`, `parse`, `format`, `add_days`, `diff_days` |

`import text` hides the old built-in `text(x)` in that file; use `to_text(x)` (the
recommended name) or `import text as t`.

Full details: [Standard library](08-standard-library.md).

## Where next

* Exact rules and grammar: [Language reference](03-language-reference.md)
* Every built-in: [Built-in spells](04-builtins.md)
* The standard modules: [Standard library](08-standard-library.md)
* Ready-made programs: [Cookbook](06-cookbook.md)
