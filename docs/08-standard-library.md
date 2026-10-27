# 8. Standard library

Arcana 1.0 ships ten standard modules, written in Rust and built into the `arcana`
binary. There is nothing to install: just `import` them.

| Module                      | What it is for                                        |
|-----------------------------|-------------------------------------------------------|
| [`math`](#math)             | roots, powers, rounding, trigonometry, logs, random numbers |
| [`text`](#text)             | trimming, searching, replacing, slicing, padding text |
| [`list`](#list)             | sorting, searching, slicing and changing lists        |
| [`logic`](#logic)           | type checks, emptiness, `all` / `any`                 |
| [`convert`](#convert)       | conversions, number bases, number formatting          |
| [`error`](#error)           | warnings, debug output, assertions                    |
| [`system`](#system)         | version, platform, arguments, environment, exit       |
| [`file`](#file)             | reading and writing files, directories                |
| [`time`](#time)             | the clock, sleeping, durations                        |
| [`date`](#date)             | calendar dates, times and date arithmetic             |

Everything that was a built-in in 0.1 (`length`, `range`, `upper`, `append`, ...) is
**still always available**; see [Built-in spells](04-builtins.md). Several modules
re-export those same spells so you can import everything you need from one place.

## Using a module

```arc
import math                    ? use as math.sqrt(...)
import text as t               ? use as t.trim(...)
from list import sort, sum     ? use as sort(...), sum(...)

spell main() {
    cast math.sqrt(144)
    cast t.upper(t.trim("  arcana  "))
    bind scores = [30, 10, 20]
    sort(scores)
    cast scores
    cast sum(scores)
}
```

```
12.0
ARCANA
[10, 20, 30]
60
```

Rules worth remembering:

* A **standard module always wins** over a file with the same name, so a local
  `math.arc` can never replace the real `math`. Give your own grimoires different names.
* `import text` declares the name `text`, which **hides the old built-in `text(x)`**
  in that file. Use `to_text(x)` (recommended in all new code), or import the
  module under another name: `import text as t`. Calling `text(5)` after
  `import text` reports `E3005` with a note explaining exactly this.
* Misspelled names get a suggestion: `math.sqroot` gives `help: did you mean 'sqrt'?`
* Standard modules cannot be extended, and `from math import nothing` is `E5003`.

## Conventions shared by every module

These rules make the library predictable.

1. **Strict types, clear errors.** Functions never guess: `sqrt("4")` is `E3003`,
   not `2.0`. Failures that depend on data are *catchable* runtime errors
   (`E2xxx`); see [Errors and debugging](05-errors-and-debugging.md).
2. **No truthiness.** Functions that test conditions (`all`, `any`, `assert`) need
   real booleans (`On` / `Off`).
3. **Indexes start at 0, a range's end is excluded,** and out-of-range indexes are
   errors (`E2002`), never silently clamped. There are no negative indexes.
4. **Changing versus returning (lists).** A *verb* changes the list in place and
   returns `null`: `append`, `insert`, `extend`, `remove`, `clear`, `sort`,
   `reverse`, `shuffle`. `pop` removes and returns the last item. Everything else
   leaves the list alone and returns a new value: `sorted`, `reversed`, `unique`,
   `slice`, `concat`, `first`, `last`, `sum`, `index_of`. Need both? Copy first:
   `bind s = copy(items)` then `sort(s)`, or just use `sorted(items)`.
5. **Text is immutable,** so text functions always return new text.
6. **"Not found" is `null`, not an error,** for searches (`find`, `index_of`,
   `parse_number`, `env`). Operations that *require* the item raise instead
   (`remove`, `first` of an empty list).
7. **Optional arguments** are shown as `name = default`.

## Design decisions (from the library proposal)

The proposal for the standard library asked a few questions. Here is what Arcana 1.0
does, and why.

| Proposal                                   | Arcana 1.0 | Reason |
|--------------------------------------------|------------|--------|
| `print(x)`: alias for `cast`?              | **Not added.** `cast` is the only output statement. | One obvious way to print. `system.write(x)` prints without a newline; `error.warn` / `error.debug` use standard error. |
| `input(prompt)`                            | **`ask(prompt)` already exists** (plus `ask_integer`, `ask_decimal`). | No duplicate names. |
| `type_of(42) -> "number"`                  | Stays `"integer"` (and `"decimal"`). | Integers and decimals behave differently (`/`, `//`, formatting), so programs need to tell them apart. Use `logic.is_number(x)` for "either". |
| `to_text`, `to_integer`, `to_decimal`      | **Added** as the preferred names; `text`, `integer`, `decimal` still work. | Consistent `to_*` family (`to_number`, `to_boolean`, `to_list`, `to_binary`, `to_hex`) and no clash with `import text`. |
| `to_number(x)`                             | Added. Integer text gives an integer, other numeric text a decimal. | |
| `assert(condition)`                        | Added (global), with an optional message. | Raises catchable `E2101` (`AssertionError`). |
| `math.round(x)`, `math.min/max`, `abs`     | Same spells as the global ones. `round(x, digits)` is new. | `round(x)` gives an integer; with `digits` it gives a decimal. |
| `floor`, `ceil`                            | Return **integers**. | Arcana integers have unlimited size. |
| `power(x, y)`                              | Same as `x ** y`, with the same errors. | |
| `contains(text, part)`, `contains(items, v)` | Added, but `part in text` / `value in items` are the idiomatic forms. | The functions are handy with `from ... import`. |
| list functions: change the list or return a new one? | Verbs change in place, other names return new values (rule 4 above). `sort` and `reverse` change in place as in the proposal's example; `sorted` and `reversed` are the copying versions. | Explicit and consistent. |
| `is_number`, `is_text`, `is_list`, `is_null` | Added in `logic`, plus `is_integer`, `is_decimal`, `is_boolean`, `is_map`. | Clear intent when validating input. |
| `all(list)`, `any(list)`: "truthy"         | They need a **list of booleans**; other values are `E3003`. | Arcana has no truthiness. |
| `not_empty(x)`                             | **Not added**; write `not logic.is_empty(x)`. | Avoids a built-in that only negates another. |
| `parse_number(text)`: "safely"             | Returns **`null`** when the text is not a number; `to_number` raises. | Gives a real choice between "raise" and "give me null". |
| `fail`, `warn`, `debug`, `assert_equal`, `assert_true` | All in `error`. `fail` is also global. | |
| `version`, `platform`, `clock`, `args`, `exit` | All in `system`, plus `env`, `cwd`, `write`. | Only deliberate, documented capabilities: no way to run other programs. |
| (not in the proposal) `file`, `time`, `date` | Added. | A general-purpose language needs files and calendars. |

---

## `math`

```arc
import math
```

**Constants:** `math.pi`, `math.e`, `math.tau` (decimals).

| Spell | Returns | Notes |
|-------|---------|-------|
| `abs(x)` | number | keeps the type |
| `min(a, b, ...)` / `max(a, b, ...)` | number | also accept one list; same as the global spells |
| `round(x)` / `round(x, digits)` | integer / decimal | halves round away from zero; `digits` is 0 to 15 |
| `floor(x)` `ceil(x)` `trunc(x)` | integer | toward -infinity / +infinity / zero |
| `sqrt(x)` | decimal | `E2008` if `x < 0` |
| `power(x, y)` | number | same as `x ** y` |
| `clamp(x, low, high)` | number | `E2006` if `low > high` |
| `sign(x)` | integer | -1, 0 or 1 |
| `sin cos tan asin acos atan (x)`, `atan2(y, x)` | decimal | radians; `asin`/`acos` need -1..1 (`E2008`) |
| `exp(x)` | decimal | |
| `log(x)` / `log(x, base)` | decimal | natural log by default; `x` must be > 0 (`E2008`) |
| `log2(x)` `log10(x)` | decimal | |
| `hypot(x, y)` | decimal | `sqrt(x*x + y*y)` |
| `degrees(r)` `radians(d)` | decimal | |
| `gcd(a, b)` `lcm(a, b)` | integer | exact, any size |
| `factorial(n)` | integer | `0 <= n <= 20000` |
| `random()` | decimal | from 0.0 up to (not including) 1.0 |
| `random_int(low, high)` | integer | both ends included |
| `random_seed(n)` | null | makes the sequence repeatable |

```arc
import math

spell main() {
    cast math.sqrt(144)
    cast math.power(2, 5)
    cast math.clamp(120, 0, 100)
    cast math.floor(-2.1)
    cast math.round(3.14159, 2)
    cast math.gcd(12, 18)
    cast math.factorial(20)
    cast math.degrees(math.pi)
    math.random_seed(42)
    bind first = math.random_int(1, 100)
    math.random_seed(42)
    cast first == math.random_int(1, 100)
}
```

```
12.0
32
100
-3
3.14
6
2432902008176640000
180.0
On
```

Without `random_seed` the generator is seeded from the clock, so every run differs.
It is **not** suitable for cryptography.

## `text`

```arc
import text
```

Indexes count characters (not bytes), start at 0, and ends are excluded.

| Spell | Returns | Notes |
|-------|---------|-------|
| `upper(s)` `lower(s)` | text | |
| `trim(s)` `trim_start(s)` `trim_end(s)` | text | remove surrounding whitespace |
| `contains(s, part)` `starts_with(s, part)` `ends_with(s, part)` | boolean | |
| `replace(s, old, new)` | text | all occurrences; `old` cannot be empty (`E2006`) |
| `split(s)` / `split(s, sep)` | list | no separator: split on whitespace |
| `join(items, sep = "")` | text | items must all be text |
| `substring(s, start)` / `substring(s, start, end)` | text | `E2002` if out of range |
| `char_at(s, index)` | text | same as `s[index]` |
| `find(s, part)` | integer or null | index of the first match |
| `repeat_text(s, n)` | text | `n >= 0`; result is limited to 50 million characters |
| `pad_left(s, width, fill = " ")` `pad_right(...)` | text | `fill` is one character; longer text is returned unchanged |
| `lines(s)` | list | split at line breaks |
| `chars(s)` | list | one text per character |
| `reverse_text(s)` | text | |
| `code_of(ch)` | integer | Unicode code point of a one-character text |
| `from_code(n)` | text | the inverse of `code_of` |

```arc
import text

spell main() {
    cast text.upper(text.trim("  arcana  "))
    cast text.replace("a-b-c", "-", "+")
    cast text.substring("hello world", 6)
    cast text.pad_left("7", 3, "0")
    cast text.find("hello", "ll")
    cast text.find("hello", "z")
    cast text.chars("héy")
}
```

```
ARCANA
a+b+c
world
007
2
null
["h", "é", "y"]
```

## `list`

```arc
import list
```

| Spell | Returns | Changes the list? |
|-------|---------|-------------------|
| `append(items, v)` | null | yes |
| `insert(items, index, v)` | null | yes; `0 <= index <= length` |
| `extend(items, other)` | null | yes |
| `remove(items, v)` | null | yes; removes the first equal item, `E2006` if absent |
| `clear(items)` | null | yes |
| `sort(items)` / `sort(items, descending)` | null | yes; all numbers or all text (`E3003`) |
| `reverse(items)` | null | yes |
| `shuffle(items)` | null | yes (uses the random generator) |
| `pop(items)` | the removed last item | yes; `E2002` if empty |
| `sorted(items)` / `sorted(items, descending)` | new list | no |
| `reversed(items)` | new list | no |
| `unique(items)` | new list | no; keeps the first of each value |
| `slice(items, start)` / `slice(items, start, end)` | new list | no; `E2002` if out of range |
| `concat(a, b)` | new list | no |
| `first(items)` `last(items)` | item | no; `E2002` if empty |
| `index_of(items, v)` | integer or null | no |
| `contains(items, v)` | boolean | no; same as `v in items` |
| `sum(items)` | number | no; numbers only; empty list gives `0` |
| `choice(items)` | item | no; random item, `E2002` if empty |

```arc
import list

spell main() {
    bind scores = [30, 10, 20, 10]
    list.sort(scores)
    cast scores
    cast list.sorted(scores, On)
    cast list.unique(scores)
    cast list.sum(scores)
    cast list.slice(scores, 1, 3)
    cast list.index_of(scores, 99)
    list.remove(scores, 10)
    cast scores
}
```

```
[10, 10, 20, 30]
[30, 20, 10, 10]
[10, 20, 30]
70
[10, 20]
null
[10, 20, 30]
```

## `logic`

```arc
import logic
```

| Spell | Returns |
|-------|---------|
| `is_number(x)` | `On` for an integer or a decimal |
| `is_integer(x)` `is_decimal(x)` `is_text(x)` `is_boolean(x)` `is_list(x)` `is_map(x)` `is_null(x)` | boolean |
| `is_empty(x)` | `On` for empty text, list, map or range; other types are `E3003` (use `is_null`) |
| `all(items)` | `On` if every item is `On` (an empty list gives `On`) |
| `any(items)` | `On` if some item is `On` (an empty list gives `Off`) |

`all` and `any` need lists of booleans, so compare first:

```arc
import logic

spell main() {
    bind ages = [21, 34, 17]
    bind adult = []
    for a in ages { append(adult, a >= 18) }
    cast logic.all(adult)
    cast logic.any(adult)
    cast logic.is_number(2.5)
    cast logic.is_empty("")
}
```

```
Off
On
On
On
```

## `convert`

```arc
import convert
```

| Spell | Returns | Notes |
|-------|---------|-------|
| `to_boolean(x)` | boolean | numbers: `0` is `Off`; `null` is `Off`; text: `On`/`Off`, `true`/`false`, `yes`/`no`, `1`/`0` in any case, otherwise `E2011` |
| `to_list(x)` | new list | text gives characters, a map gives `[key, value]` pairs, a range gives its integers, a list gives a copy |
| `to_map(pairs)` | map | the inverse of `to_list(map)` |
| `to_binary(n)` `to_octal(n)` `to_hex(n)` | text | with a prefix: `0b1010`, `0o10`, `0xff`; negatives look like `-0b101` |
| `from_binary(s)` `from_octal(s)` `from_hex(s)` | integer | the prefix is optional; `E2011` on bad digits |
| `parse_number(s)` | number or null | **null** if the text is not a number (never raises) |
| `format_number(x, digits)` | text | exactly `digits` decimal places (0 to 100) |

(`to_text`, `to_integer`, `to_decimal` and `to_number` are global spells.)

```arc
import convert

spell main() {
    cast convert.to_binary(10)
    cast convert.to_hex(255)
    cast convert.from_hex("0xff")
    cast convert.parse_number("12")
    cast convert.parse_number("abc")
    cast convert.format_number(3.14159, 2)
    cast convert.format_number(5, 2)
    cast convert.to_list({"a": 1})
}
```

```
0b1010
0xff
255
12
null
3.14
5.00
[["a", 1]]
```

## `error`

```arc
import error
```

| Spell | Effect |
|-------|--------|
| `fail(message)` | raise a catchable `E2100` (`UserError`); also a global spell |
| `warn(message)` | write `warning: message` to **standard error** |
| `debug(value)` | write `debug: value (type)` to standard error |
| `assert_true(condition, message = ...)` | `E2101` (`AssertionError`) when the condition is `Off` |
| `assert_equal(a, b, message = ...)` | `E2101` unless `a == b` (Arcana equality) |

`warn` and `debug` never appear in the program's normal output, so redirecting
standard output stays clean. Assertions are catchable, which makes them usable
both for tests and for validating input.

```arc
import error

spell main() {
    error.assert_equal(2 + 2, 4)
    try {
        error.assert_equal([1, 2], [1, 3])
    } catch e {
        cast e["kind"]
        cast e["message"]
    }
}
```

```
AssertionError
Assertion failed: [1, 2] is not equal to [1, 3]
```

## `system`

```arc
import system
```

| Spell | Returns | Notes |
|-------|---------|-------|
| `version()` | text | the Arcana version, e.g. `"1.0.0"` |
| `platform()` | text | `"linux"`, `"macos"`, `"windows"`, ... |
| `clock()` | decimal | monotonic seconds since the program started; use it to time things |
| `args()` | list of text | the arguments after the grimoire name |
| `env(name)` | text or null | an environment variable |
| `cwd()` | text | the current directory |
| `write(x)` | null | print without a trailing newline |
| `exit(code = 0)` | never returns | stop now with status 0 to 255 |

**Program arguments.** Everything after the file name goes to the program:

```
arcana run greet.arc Mira 3            ? system.args() is ["Mira", "3"]
arcana run greet.arc -- --max-steps 5  ? after `--`, arcana ignores the rest
arcana run greet.arc --max-steps 1000 Mira   ? arcana's own options still work
```

**`exit`** ends the program immediately and quietly. `finally` blocks still run and
`catch` cannot intercept it. The status becomes the process's exit code, so shell
scripts can react to it: `arcana run job.arc || echo "job failed"`.

```arc
import system

spell main() {
    bind args = system.args()
    if length(args) == 0 {
        cast "usage: greet NAME"
        system.exit(2)
    }
    cast "Hello, " + args[0]
}
```

## `file`

```arc
import file
```

Paths are text, relative to the directory the program was started from. Every
failure is a catchable `E2012` (kind `FileError`) with the operating system's reason.
Files are read and written as UTF-8 text.

| Spell | Returns | Notes |
|-------|---------|-------|
| `read_text(path)` | text | whole file |
| `read_lines(path)` | list | one text per line, without line endings |
| `write_text(path, text)` | null | creates or replaces the file |
| `append_text(path, text)` | null | creates the file if needed |
| `write_lines(path, lines)` | null | each line ends with a newline |
| `exists(path)` `is_file(path)` `is_dir(path)` | boolean | |
| `size(path)` | integer | bytes |
| `remove(path)` | null | a file or an **empty** directory |
| `make_dir(path)` | null | creates parents too |
| `list_dir(path)` | list of text | names, sorted |

```arc
import file

spell main() {
    file.write_lines("demo_notes.txt", ["one", "two"])
    file.append_text("demo_notes.txt", "three\n")
    cast file.read_lines("demo_notes.txt")
    cast file.size("demo_notes.txt")
    file.remove("demo_notes.txt")
    cast file.exists("demo_notes.txt")
    try {
        cast file.read_text("demo_notes.txt")
    } catch e {
        cast e["code"] + " " + e["kind"]
    }
}
```

```
["one", "two", "three"]
14
Off
E2012 FileError
```

## `time`

```arc
import time
```

| Spell | Returns | Notes |
|-------|---------|-------|
| `now()` | decimal | seconds since 1970-01-01 00:00:00 UTC (the Unix timestamp) |
| `sleep(seconds)` | null | pauses the program; `0` to 1,000,000,000 |
| `format_duration(seconds)` | text | `3725` is `"1h 2m 5s"`, `90061` is `"1d 1h 1m 1s"`, `0.25` is `"250ms"` |

To measure how long something takes, use `system.clock()`:

```arc
import system
import time

spell main() {
    bind started = system.clock()
    time.sleep(0.05)
    cast system.clock() - started >= 0.05
    cast time.format_duration(3725)
}
```

```
On
1h 2m 5s
```

## `date`

```arc
import date
```

### The model

A **date** is a map with `year`, `month` and `day`. A **datetime** also has `hour`,
`minute` and `second`. They are ordinary values: print them, read fields with
`d["year"]`, compare with `==`.

```arc
import date

spell main() {
    bind d = date.make_date(2026, 10, 27)
    cast d
    cast d["month"]
    cast date.make_datetime(2026, 10, 27, 14, 30)
    cast d == date.parse("2026-10-27")
}
```

```
{"year": 2026, "month": 10, "day": 27}
10
{"year": 2026, "month": 10, "day": 27, "hour": 14, "minute": 30, "second": 0}
On
```

* The calendar is the Gregorian calendar for years **1 to 9999**; every function
  checks that dates really exist (`2026-02-30` is `E2006`).
* Weekdays are **ISO numbers: 1 = Monday ... 7 = Sunday**.
* There is no time-zone database. Values are plain wall-clock readings. `utc_now()`
  gives UTC; `now()` and `today()` apply the computer's current UTC offset (on
  Windows they are UTC). To convert between zones yourself, use
  `from_timestamp(ts, offset_seconds)` or `add_hours`.
* Functions taking a date also accept a datetime (the time is ignored where it does
  not matter, and treated as midnight for dates in `diff_seconds` / `compare`).

### Creating and reading

| Spell | Returns | Notes |
|-------|---------|-------|
| `today()` | date | local date |
| `now()` | datetime | local time |
| `utc_now()` | datetime | UTC |
| `local_offset()` | integer | seconds the local clock is ahead of UTC |
| `make_date(y, m, d)` | date | `E2006` for an impossible date |
| `make_datetime(y, m, d, h = 0, mi = 0, s = 0)` | datetime | |
| `parse(text)` | date or datetime | `YYYY-MM-DD`, `YYYY-MM-DD HH:MM[:SS]` or with `T`; `E2011` if it is not a date |
| `from_timestamp(ts, offset = 0)` | datetime | `ts` in seconds since 1970 UTC; `offset` in seconds |
| `to_timestamp(d)` | integer | treats `d` as UTC |
| `weekday(d)` | integer | 1 to 7 |
| `weekday_name(d)` / `weekday_name(1..7)` | text | `"Monday"` ... |
| `month_name(d)` / `month_name(1..12)` | text | `"January"` ... |
| `day_of_year(d)` | integer | 1 to 366 |
| `is_leap_year(y)` | boolean | |
| `days_in_month(y, m)` | integer | |

### Formatting

`format(d, pattern)` fills in `%` codes; `iso(d)` gives `2026-10-27` or
`2026-10-27T14:30:05`.

| Code | Meaning | Code | Meaning |
|------|---------|------|---------|
| `%Y` | year, 4 digits | `%B` | month name |
| `%y` | year, 2 digits | `%b` | month, 3 letters |
| `%m` | month `01`-`12` | `%A` | weekday name |
| `%d` | day `01`-`31` | `%a` | weekday, 3 letters |
| `%e` | day, space padded | `%u` | weekday `1`-`7` |
| `%H` | hour `00`-`23` | `%j` | day of year `001`-`366` |
| `%I` | hour `01`-`12` | `%p` | `AM` / `PM` |
| `%M` | minute | `%S` | second |
| `%%` | a percent sign | | |

An unknown code is `E2006`.

### Arithmetic and comparison

| Spell | Returns | Notes |
|-------|---------|-------|
| `add_days(d, n)` | same kind as `d` | `n` may be negative |
| `add_months(d, n)` `add_years(d, n)` | same kind as `d` | the day is limited to the end of the month: Jan 31 + 1 month = Feb 28 |
| `add_hours(d, n)` `add_minutes(d, n)` `add_seconds(d, n)` | datetime | a date becomes a datetime at midnight |
| `diff_days(a, b)` | integer | `a - b` in calendar days |
| `diff_seconds(a, b)` | integer | `a - b` in seconds |
| `compare(a, b)` | -1, 0 or 1 | chronological order |

Going outside the years 1 to 9999 is `E2008`.

```arc
import date

spell main() {
    bind launch = date.make_date(2026, 10, 27)
    cast date.format(launch, "%A, %d %B %Y")
    bind deadline = date.add_days(launch, 45)
    cast date.iso(deadline) + " " + date.weekday_name(deadline)
    cast date.diff_days(deadline, launch)
    cast date.iso(date.add_months(date.make_date(2026, 1, 31), 1))
    cast date.format(date.parse("2026-11-03 09:30"), "%a %b %e, %I:%M %p")
    cast date.compare(launch, deadline)
    cast date.iso(date.from_timestamp(0))
}
```

```
Tuesday, 27 October 2026
2026-12-11 Friday
45
2026-02-28
Tue Nov  3, 09:30 AM
-1
1970-01-01T00:00:00
```

### Sorting dates

`sort` works on numbers and text, so sort dates by their timestamps, or insert each
date at the right place using `compare` (see `examples/dates.arc` for more date code):

```arc
import date
import list

spell main() {
    bind days = [date.make_date(2026, 3, 1), date.make_date(2025, 12, 25), date.make_date(2026, 1, 15)]
    bind ordered = []
    for d in days {
        bind at = length(ordered)
        for i in range(length(ordered)) {
            if date.compare(d, ordered[i]) < 0 {
                at = i
                break
            }
        }
        list.insert(ordered, at, d)
    }
    for d in ordered { cast date.iso(d) }
}
```

```
2025-12-25
2026-01-15
2026-03-01
```
