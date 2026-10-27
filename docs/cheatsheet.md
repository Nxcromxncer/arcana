# Arcana cheat sheet

```
? comment                              spell main() { ... }   <- program entry point

bind x = 10                            ? declare            x = x + 1   ? assign
cast "hello"                           ? print

Types   integer 42   decimal 2.5   text "hi"   boolean On / Off   null
        list [1, 2]  map {"a": 1}  range range(1, 4)
Escapes \n \t \\ \" \u{2728}           Numbers 1_000  1e3  (no .5 or 5.)

Math    + - * /  //  %  **            / is decimal, // floors, % has the divisor's sign
Compare == != < <= > >=  in           no chaining: a < b and b < c
Logic   and  or  not                  operands must be On/Off (no truthiness)
Text    "a" + "b"   text(5)   "ell" in "hello"   "abc"[1]   length("abc")

if cond { } else if cond { } else { }
while cond { }
for item in list_or_text_or_map_or_range { }
break    continue    return value

spell add(a, b = 1) { return a + b }   ? defaults allowed (last params only)

try { } catch err { err["code"]  err["kind"]  err["message"] } finally { }
fail("message")                        ? raise your own error (E2100)

import lib        import a.b as x        from lib import name        ? top of file only
```

## Built-ins

| Spell | Does |
|---|---|
| `ask(p)` `ask_integer(p)` `ask_decimal(p)` | read input (prompt optional) |
| `to_text(x)` `to_integer(x)` `to_decimal(x)` `to_number(x)` `round(x)` / `round(x, digits)` | convert (`text`, `integer`, `decimal` still work) |
| `abs(x)` `min(...)` `max(...)` | numbers (or text for min/max); also take one list |
| `length(x)` | text, list, map, range |
| `range(stop)` / `range(a, b)` / `range(a, b, step)` | lazy integer sequence, stop excluded |
| `append(list, v)` `pop(list)` `copy(x)` | list operations |
| `keys(map)` `values(map)` | lists in insertion order |
| `upper(s)` `lower(s)` `split(s)` `split(s, sep)` `join(list, sep)` | text |
| `assert(cond)` / `assert(cond, msg)` | raise `E2101` when `Off` |
| `type_of(x)` | `"integer"` `"decimal"` `"text"` `"boolean"` `"null"` `"list"` `"map"` `"range"` `"spell"` `"grimoire"` |

## Standard modules (`import math`, or `from math import sqrt`)

| Module | Highlights |
|---|---|
| `math` | `pi e tau`, `sqrt power floor ceil trunc clamp sign gcd lcm factorial`, `sin cos tan asin acos atan atan2 exp log log2 log10 hypot degrees radians`, `random random_int random_seed` |
| `text` | `trim trim_start trim_end contains starts_with ends_with replace substring find char_at repeat_text pad_left pad_right lines chars reverse_text code_of from_code` (+ `upper lower split join`) |
| `list` | in place: `append insert extend remove clear sort reverse shuffle`; `pop`; new value: `sorted reversed unique slice concat first last index_of contains sum choice` |
| `logic` | `is_number is_integer is_decimal is_text is_boolean is_list is_map is_null is_empty all any` |
| `convert` | `to_boolean to_list to_map to_binary to_octal to_hex from_binary from_octal from_hex parse_number format_number` |
| `error` | `fail warn debug assert_true assert_equal` |
| `system` | `version platform clock args env cwd write exit` |
| `file` | `read_text read_lines write_text append_text write_lines exists is_file is_dir size remove make_dir list_dir` |
| `time` | `now sleep format_duration` |
| `date` | `today now utc_now local_offset make_date make_datetime parse from_timestamp to_timestamp format iso weekday weekday_name month_name day_of_year is_leap_year days_in_month add_days add_months add_years add_hours add_minutes add_seconds diff_days diff_seconds compare` |

Verbs change a list in place and return `null`; other list functions return new values.
`import text` hides the old `text(x)`: use `to_text(x)`. See `docs/08-standard-library.md`.

## Precedence (loosest to tightest)

`or` < `and` < `not` < comparisons / `in` < `+ -` < `* / // %` < unary `-` < `**` < call, `[ ]`, `.`

## Commands

```
arcana run file.arc [args...] [--max-steps N] [--max-depth N]     arcana check file.arc
arcana tokens file.arc                                  arcana version
```

Exit codes: 0 ok, 1 Arcana error, 2 usage/file problem, or your own via `system.exit(n)`.
Words after the file name go to `system.args()`. Standard modules win over files; other
imports are searched next to the importing file, then in `ARCANA_PATH`.

## Reserved words

`spell bind cast if else while for in return break continue try catch finally
import from as On Off null and or not` and `count repeat times`.

## Error families

`E1xxx` names / lexical - `E2xxx` runtime (catchable) - `E3xxx` types -
`E4xxx` syntax - `E5xxx` imports. New in 1.0: `E2012` file error, `E2101` failed assertion.
See `docs/05-errors-and-debugging.md`.
