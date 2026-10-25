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
| `text(x)` `integer(x)` `decimal(x)` `round(x)` | convert |
| `abs(x)` `min(...)` `max(...)` | numbers (or text for min/max); also take one list |
| `length(x)` | text, list, map, range |
| `range(stop)` / `range(a, b)` / `range(a, b, step)` | lazy integer sequence, stop excluded |
| `append(list, v)` `pop(list)` `copy(x)` | list operations |
| `keys(map)` `values(map)` | lists in insertion order |
| `upper(s)` `lower(s)` `split(s)` `split(s, sep)` `join(list, sep)` | text |
| `type_of(x)` | `"integer"` `"decimal"` `"text"` `"boolean"` `"null"` `"list"` `"map"` `"range"` `"spell"` `"grimoire"` |

## Precedence (loosest to tightest)

`or` < `and` < `not` < comparisons / `in` < `+ -` < `* / // %` < unary `-` < `**` < call, `[ ]`, `.`

## Commands

```
arcana run file.arc [--max-steps N] [--max-depth N]     arcana check file.arc
arcana tokens file.arc                                  arcana version
```

Exit codes: 0 ok, 1 Arcana error, 2 usage/file problem. Imports are searched next
to the importing file, then in `ARCANA_PATH`.

## Reserved words

`spell bind cast if else while for in return break continue try catch finally
import from as On Off null and or not` and `count repeat times`.

## Error families

`E1xxx` names / lexical - `E2xxx` runtime (catchable) - `E3xxx` types -
`E4xxx` syntax - `E5xxx` imports. See `docs/05-errors-and-debugging.md`.
