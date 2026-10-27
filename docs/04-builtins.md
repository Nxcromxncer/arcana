# 4. Built-in spells

Built-ins are ordinary spell values that are **always in scope**: no `import`
needed. Calling one with the wrong number of arguments is `E3004`; the wrong
argument type is `E3003` (unless noted). The error always points at the call in
your code.

Arcana 1.0 also has ten [standard modules](08-standard-library.md) (`math`, `text`,
`list`, `date`, ...) that you `import` when you need more.

| Group      | Spells                                                        |
|------------|---------------------------------------------------------------|
| Input      | [`ask`](#ask) [`ask_integer`](#ask_integer) [`ask_decimal`](#ask_decimal) |
| Conversion | [`to_text`](#text) [`to_integer`](#integer) [`to_decimal`](#decimal) [`to_number`](#to_number) [`round`](#round) |
| Numbers    | [`abs`](#abs) [`min`](#min--max) [`max`](#min--max)          |
| Collections| [`length`](#length) [`range`](#range) [`append`](#append) [`pop`](#pop) [`keys`](#keys) [`values`](#values) [`copy`](#copy) |
| Text       | [`upper`](#upper--lower) [`lower`](#upper--lower) [`split`](#split) [`join`](#join) |
| Other      | [`type_of`](#type_of) [`fail`](#fail) [`assert`](#assert)    |

You may shadow a built-in with your own `bind` (for example `bind length = 3`),
but you cannot assign to a built-in name.

> **1.0 naming.** The conversion spells are now spelled `to_text`, `to_integer`,
> `to_decimal` (and the new `to_number`). The 0.1 names `text`, `integer` and
> `decimal` keep working forever, but prefer the `to_` names: `import text` hides
> the old `text(x)`, while `to_text(x)` is never affected.

---

## Input

### `ask`
`ask(prompt = "")` returns **text**.

Prints the prompt (without a newline), reads one line from standard input, and
returns it without the line ending.

```arc
spell main() {
    bind name = ask("Your name: ")
    cast "Hello, " + name
}
```

Errors: `E3003` if the prompt is not text; `E2010` (kind `InputError`, catchable)
if input has ended.

### `ask_integer`
`ask_integer(prompt = "")` returns an **integer**.

Like `ask`, then converts the answer. Surrounding spaces are ignored. Raises the
catchable `E2010` if the answer is not a whole number.

### `ask_decimal`
`ask_decimal(prompt = "")` returns a **decimal**.

Like `ask_integer` but accepts anything like `3.14`, `42`, `1e3`. `nan` and `inf`
are rejected with `E2010`.

---

## Conversion

### `text`
`to_text(x)` (also spelled `text(x)`) returns **text**. Converts any value to the text
`cast` would print.

```arc
spell main() {
    cast to_text(42) + "!"        ? 42!
    cast to_text([1, "a", On])    ? [1, "a", On]
}
```

Nested text inside lists and maps is shown with quotes; top-level text is returned
unchanged.

### `integer`
`to_integer(x)` (also spelled `integer(x)`) returns an **integer**.

| Argument           | Result                                                         |
|--------------------|----------------------------------------------------------------|
| integer            | unchanged                                                      |
| decimal with no fraction (`7.0`) | `7`                                              |
| decimal with a fraction (`3.5`)  | error `E2011` (use `round`)                      |
| text (`"  42 "`)   | `42`; spaces around are ignored, `_` between digits allowed    |
| text that is not a whole number  | error `E2011` (kind `ConversionError`)           |
| anything else      | `E3003`                                                        |

### `decimal`
`to_decimal(x)` (also spelled `decimal(x)`) returns a **decimal**. Accepts integers (`decimal(3)` is `3.0`),
decimals and numeric text (`"2.5"`). Bad text is `E2011`; an integer too large for
a decimal is `E2008`.

### `to_number`
`to_number(x)` returns an **integer or a decimal**. A number is returned unchanged;
text such as `"25"` gives the integer `25` and `"2.5"` gives the decimal `2.5`. Text
that is not a number is `E2011` (catchable); use `convert.parse_number` to get `null`
instead of an error.

### `round`
`round(x)` returns an **integer**. Rounds to the nearest whole number, with halves
rounded **away from zero**: `round(2.5)` is `3`, `round(-2.5)` is `-3`,
`round(2.4)` is `2`. An integer is returned unchanged. Non-numbers are `E3003`.

`round(x, digits)` rounds to `digits` decimal places (0 to 15, else `E2006`) and
always returns a **decimal**: `round(3.14159, 2)` is `3.14`, `round(5, 2)` is `5.0`.

---

## Numbers

### `abs`
`abs(x)` returns the absolute value, keeping the type (`abs(-7)` is `7`,
`abs(-2.5)` is `2.5`).

### `min` / `max`
`min(a, b, ...)`, `max(a, b, ...)`, or `min(list)`, `max(list)`.

Take any number of numbers, or any number of texts, or a single list of either.
All values must be numbers or all must be text (`E3003` otherwise, also for an
empty list). When values tie, the first one wins.

```arc
spell main() {
    cast min(3, 1, 2)            ? 1
    cast max([4, 9, 2])          ? 9
    cast min("pear", "apple")    ? apple
}
```

---

## Collections

### `length`
`length(x)` returns an **integer**: characters in text, items in a list, keys in a
map, or elements in a range.

```arc
spell main() {
    cast length("héllo")             ? 5
    cast length([1, 2])              ? 2
    cast length({"a": 1})            ? 1
    cast length(range(0, 10, 3))     ? 4
}
```

### `range`
`range(stop)`, `range(start, stop)`, `range(start, stop, step)`.

Integers only (`E3003` otherwise). The sequence starts at `start` (default 0),
advances by `step` (default 1) and stops **before** `stop`. A negative step counts
down. A step of zero is `E2006` (catchable). A range is lazy, so `range(1000000000)`
costs nothing until you loop over it.

```arc
spell main() {
    cast range(4)              ? range(0, 4, 1)
    cast range(5, 0, -2)       ? range(5, 0, -2)   (yields 5, 3, 1)
    cast length(range(1, 10, 4))   ? 3
    cast range(10, 0, -3)[1]       ? 7
    cast 3 in range(5)             ? On
}
```

### `append`
`append(list, value)` returns `null`. Adds `value` to the end of the list, in place.
The first argument must be a list.

### `pop`
`pop(list)` removes the last item and returns it. An empty list is `E2002`
(catchable); check `length(list) > 0` first.

### `keys`
`keys(map)` returns a **list** of the map's keys, in insertion order.

### `values`
`values(map)` returns a **list** of the map's values, in the same order as `keys`.

### `copy`
`copy(x)` returns a shallow copy of a list or map (the items themselves are not
duplicated). Any other value is returned unchanged.

```arc
spell main() {
    bind a = [1, 2]
    bind b = copy(a)
    append(b, 3)
    cast a      ? [1, 2]
    cast b      ? [1, 2, 3]
}
```

---

## Text

### `upper` / `lower`
`upper(s)` and `lower(s)` return the text converted to upper or lower case.

### `split`
`split(s)` splits on runs of whitespace and ignores leading/trailing space:
`split("a b   c")` gives `["a", "b", "c"]`.

`split(s, separator)` splits at every occurrence of a non-empty separator and keeps
empty pieces: `split("a,b,,c", ",")` gives `["a", "b", "", "c"]`. An empty
separator is `E2011` (kind `ValueError`, catchable).

### `join`
`join(list, separator = "")` returns **text**. Every item of the list must be text
(`E3003` otherwise; convert items with `text(x)` first).

```arc
spell main() {
    cast join(["x", "y", "z"], "-")    ? x-y-z
    cast join(["x", "y"])              ? xy
}
```

---

## Other

### `type_of`
`type_of(x)` returns the type name as text: `integer`, `decimal`, `text`,
`boolean`, `null`, `list`, `map`, `range`, `spell` (user-defined *and* built-in
spells) or `grimoire`.

### `fail`
`fail(message)` raises a catchable `E2100` error of kind `UserError`. The message
is converted with `to_text()`. If nothing catches it, the program stops and prints
the diagnostic.

```arc
spell withdraw(balance, amount) {
    if amount > balance {
        fail("insufficient mana")
    }
    return balance - amount
}

spell main() {
    try {
        cast withdraw(50, 80)
    } catch e {
        cast e["kind"] + ": " + e["message"]     ? UserError: insufficient mana
    }
}
```

### `assert`
`assert(condition)` or `assert(condition, message)` does nothing when the condition
is `On`. When it is `Off` it raises a catchable `E2101` error of kind
`AssertionError` with the message (default `Assertion failed`). The condition must be
a boolean (`E3003` otherwise): there is no truthiness.

```arc
spell main() {
    assert(1 + 1 == 2)
    try {
        assert(length([]) > 0, "the list must not be empty")
    } catch e {
        cast e["kind"] + ": " + e["message"]
    }
}
```

```
AssertionError: the list must not be empty
```

## More spells: the standard modules

Sorting (`sort`, `sorted`), searching (`find`, `index_of`), `trim`, `replace`, `sqrt`,
`floor`, file access, dates and much more live in the
[standard library](08-standard-library.md).
