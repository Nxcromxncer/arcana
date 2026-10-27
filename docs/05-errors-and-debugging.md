# 5. Errors and debugging

Arcana tries to make every mistake easy to fix. Each error has a stable code, an
exact location, the offending line with a marker, and usually a `help:` line that
says what to do.

## Reading a diagnostic

```
error E1001: Unknown name 'Mana'
  --> game.arc:3:10
  |
3 |     cast Mana
  |          ^^^^
help: did you mean 'mana'?
note: names are case-sensitive
```

| Part                     | Meaning                                                          |
|--------------------------|------------------------------------------------------------------|
| `error E1001`            | the error **code** (look it up below) and a short message        |
| `--> game.arc:3:10`      | file, **line 3**, **column 10** (columns count characters)      |
| the quoted line and `^^^^` | the source line, with the exact span underlined                |
| `help:`                  | a suggested fix                                                  |
| `note:`                  | extra context                                                    |

Errors raised inside an imported grimoire name *that* file, with its own source
line. Errors with no single location (such as `E2004`, no `main`) show only the file.

Code families:

| Family  | Category   | When it happens                                   |
|---------|------------|---------------------------------------------------|
| `E1xxx` | lexical / name | bad characters, bad numbers or text, unknown names |
| `E2xxx` | runtime    | something went wrong while the program ran        |
| `E3xxx` | type       | wrong kind of value, wrong argument count         |
| `E4xxx` | syntax     | the code does not follow the grammar              |
| `E5xxx` | import     | problems loading another grimoire                 |
| `E6xxx` | permission | reserved for a later version (not used yet)       |

## Which errors can be caught?

Only **runtime** errors that depend on the data a program sees can be caught with
`try`/`catch`. Mistakes in the code itself cannot, because catching them would only
hide a bug.

| Code    | Kind              | Catchable | Typical cause                                |
|---------|-------------------|-----------|----------------------------------------------|
| `E2001` | `MathError`       | yes       | division by zero, `0 ** -1`                  |
| `E2002` | `IndexError`      | yes       | list/text/range index out of range, `pop([])`, `first([])`, a bad `slice` / `substring` |
| `E2005` | `KeyError`        | yes       | reading a map key that is not there          |
| `E2006` | `ValueError`      | yes       | `range(1, 5, 0)`, an impossible date, `clamp(1, 5, 2)`, `remove` of a missing item |
| `E2008` | `MathError`       | yes       | result too large, `sqrt(-1)`, `log(0)`, a date outside years 1-9999 |
| `E2010` | `InputError`      | yes       | `ask_integer` got non-numeric text; no input left |
| `E2012` | `FileError`       | yes       | a `file` function failed (missing file, no permission, ...) |
| `E2011` | `ConversionError` / `ValueError` | yes | `integer("x")`, `integer(3.5)`, `split(s, "")`, `date.parse("nope")`, `from_hex("zz")` |
| `E2100` | `UserError`       | yes       | you called `fail("...")`                     |
| `E2101` | `AssertionError`  | yes       | `assert`, `assert_true` or `assert_equal` failed |
| `E2004`, `E2007`, `E2009` | - | no        | no `main`, call depth, step limit            |
| (`system.exit`)  | -  | no        | ends the program on purpose; `finally` still runs |
| every `E1xxx`, `E3xxx`, `E4xxx`, `E5xxx` | - | no | fix the code                       |

In a `catch` clause the variable is a map:

```arc
spell main() {
    try {
        cast [1, 2, 3][9]
    } catch e {
        cast e["code"]        ? E2002
        cast e["kind"]        ? IndexError
        cast e["message"]     ? Index 9 is out of range for a list of length 3
    }
}
```

Decide by `kind` (or `code`) what to do:

```arc
spell parse_or_default(s, fallback) {
    try {
        return integer(s)
    } catch e {
        if e["kind"] == "ConversionError" {
            return fallback
        }
        fail("unexpected: " + e["message"])     ? re-raise anything else
    }
}

spell main() {
    cast parse_or_default("12", 0)     ? 12
    cast parse_or_default("abc", 0)    ? 0
}
```

## Error catalogue

Every example below is a real program whose output was captured from Arcana.

### E1xxx: lexical errors and unknown names

| Code    | Message                          | Cause and fix |
|---------|----------------------------------|---------------|
| `E1001` | `Unknown name 'x'`               | A name was used (or assigned) before being declared. Check spelling and case; declare with `bind x = ...`. The help suggests close matches, and explains `true`/`false` (use `On`/`Off`). |
| `E1002` | `Unexpected character '!'`       | A character Arcana does not use. Hints: `!` -> `not` / `!=`; `#` -> comments are `?`; `;` -> end statements with a newline; `'` -> use double quotes; `&` -> `and`; `\|` -> `or`. |
| `E1003` | `Unterminated text`              | A text literal has no closing `"` on its line. |
| `E1004` | `Invalid escape sequence '\q'`   | Only `\n \t \\ \" \u{hex}` are valid. Also reported for a bad `\u{...}` code point. |
| `E1005` | `Invalid number '5x'`            | Number glued to letters, bad `_` placement, or too large. |

```
error E1002: Unexpected character '!'
  --> E1002.arc:2:16
  |
2 |     bind x = 5 ! 2
  |                ^
help: use 'not' for negation and '!=' for inequality
```

### E2xxx: runtime errors

| Code    | Message (example)                                   | Cause and fix |
|---------|-----------------------------------------------------|---------------|
| `E2001` | `Division by zero` / `Zero cannot be raised to a negative power` | Check the divisor first: `if b != 0 { ... }`. |
| `E2002` | `Index 5 is out of range for a list of length 2`    | Valid indexes are `0` to `length - 1`; negatives are invalid. Also `Cannot pop from an empty list`. |
| `E2003` | `'break' can only be used inside a loop`            | `break`/`continue` must be inside `for`/`while` (not in a spell declared inside the loop). Detected before the program runs. |
| `E2004` | `No 'main' spell found`                             | Define `spell main() { ... }`. |
| `E2005` | `Key "b" not found in map`                          | Test with `"b" in map` first. |
| `E2006` | `range() step cannot be zero`                       | Use a positive or negative step. |
| `E2007` | `Maximum call depth (1000) exceeded`                | Runaway recursion; add a base case. Raise the limit with `--max-depth N` if the depth is legitimate. |
| `E2008` | `Numeric overflow: the result is too large` / `The result is not a real number` / `Number is too large to be a decimal` | Reduce the magnitude; avoid roots of negative numbers. |
| `E2009` | `Step limit of N exceeded`                          | The program ran past `--max-steps`. |
| `E2010` | `"abc" is not a valid integer` / `No input available (end of input reached)` | Catch it around `ask_integer`/`ask_decimal`/`ask`. |
| `E2011` | `Cannot convert "x" to integer` / `... without losing its fraction` / `split() separator cannot be empty` | Validate the text, or use `round(x)` for decimals. |
| `E2012` | `Cannot read 'x.txt': No such file or directory`    | A `file` function failed; the message carries the operating system's reason. Check the path with `file.exists`, or catch the error. |
| `E2100` | *your message*                                      | Raised by `fail(...)`. |
| `E2101` | `Assertion failed` / `Assertion failed: 1 is not equal to 2` | An `assert*` call found something wrong; fix the data or the code it checks. |

```
error E2002: Index 5 is out of range for a list of length 2
  --> E2002.arc:3:12
  |
3 |     cast xs[5]
  |            ^
help: valid indexes run from 0 to length - 1
```

### E3xxx: type errors

| Code    | Message (example)                                          | Cause and fix |
|---------|------------------------------------------------------------|---------------|
| `E3001` | `Condition must be a boolean (On or Off), but got integer` | Use a comparison: `x != 0`, `length(x) > 0`, `x != null`. Also raised for `and`, `or`, `not` operands. |
| `E3002` | `'a' is already declared in this scope`                    | Assign (`a = ...`) instead of `bind` again. Also: duplicate parameter names, and assigning to a built-in (`Cannot assign to built-in 'length'`). |
| `E3003` | `Cannot use '+' with text and integer`                     | Wrong type for an operation: arithmetic on non-numbers, comparing unlike types, indexing a non-collection, bad map key, looping over a non-collection, wrong argument type to a built-in. Convert explicitly with `text()`, `integer()`, `decimal()`. |
| `E3004` | `Spell 'add' expects 2 arguments but got 1`                | Match the parameter list; the help shows the signature. Note `main` takes no arguments. |
| `E3005` | `Cannot call a value of type integer`                      | Only spells can be called. If the type is `grimoire`, you probably wrote `text(5)` after `import text`: the import hides the built-in, so use `to_text(5)` or `import text as t`. |

```
error E3003: Cannot use '+' with text and integer
  --> E3003.arc:2:14
  |
2 |     cast "a" + 1
  |              ^
help: use text(value) to convert a value to text before joining
```

### E4xxx: syntax errors

| Code    | Message (example)                                           | Cause and fix |
|---------|-------------------------------------------------------------|---------------|
| `E4001` | `Expected ')' but found '}'` and similar                    | General grammar problem: missing bracket, missing `{`, unclosed block, two statements on one line, `else`/`catch`/`finally` without their partner, import not at the top. |
| `E4002` | `Only declarations are allowed at the top level of a grimoire` | Move the statement into `spell main() { ... }`. A stray `return` at top level also reports this. |
| `E4003` | `Comparisons cannot be chained`                             | `a < b < c` -> `a < b and b < c`. |
| `E4004` | `This expression has no effect` / `Cannot assign to this expression` | A statement that is not a call, `bind`, assignment or `cast`; or an assignment to something other than a name or element. |
| `E4005` | `'return' can only be used inside a spell`                  | Reserved: a top-level `return` is reported as `E4002` today. |
| `E4006` | `'On' is a reserved word and cannot be used as a name`      | Pick another name. Includes the future-reserved `count`, `repeat`, `times`. |

```
error E4001: Expected ')' but found '}'
  --> E4001.arc:3:1
  |
3 | }
  | ^
help: close the parenthesis with ')'
```

Tip: for a missing bracket, the error is often reported on the line *after* the
real mistake, because that is where the parser noticed. Look upward.

### E5xxx: import errors

| Code    | Message                                  | Cause and fix |
|---------|------------------------------------------|---------------|
| `E5001` | `Cannot find grimoire 'nothere'`         | The `.arc` file is not next to the importing file or on `ARCANA_PATH`. The `note:` lists every folder searched. |
| `E5002` | `Cyclic import of 'a2'`                  | Two grimoires import each other. Move the shared code into a third file. |
| `E5003` | `Grimoire 'ok' has no name 'zzz'`        | Misspelled name in `from m import ...` or `m.name`. For close names the help says `did you mean 'sqrt'?`. |

## Debugging toolkit

* **`arcana check file.arc`** finds syntax errors, unknown names and duplicate
  declarations in one pass without running anything, and reports *all* name
  problems at once (the interpreter stops at the first).
* **`error.debug(x)`** prints `debug: value (type)` to standard error, so it never mixes
  with your program's output. `error.warn("...")` does the same for warnings, and
  `assert(condition, "message")` documents what you expect to be true.
* **`cast` is your print debugger.** `cast type_of(x)` and `cast x` quickly
  reveal surprises. Lists and maps print with quotes around nested text, so
  `["1"]` and `[1]` look different.
* **`arcana tokens file.arc`** shows exactly how the lexer split your code.
* **`--max-steps N`** turns an endless loop into a clear `E2009` error instead of a
  hang. Pick a large N, for example `--max-steps 1000000`.
* **`--max-depth N`** controls recursion. A deliberately deep (but finite)
  recursion needs a higher value.
* **Redirect streams.** Diagnostics go to stderr, program output to stdout:
  `arcana run app.arc > out.txt` keeps the error on screen.

## Common mistakes

| You wrote                          | Problem                          | Write instead                      |
|------------------------------------|----------------------------------|------------------------------------|
| `if x { ... }` with a number/text  | `E3001` no truthiness            | `if x != 0 { ... }`                |
| `true` / `false`                   | `E1001` not defined              | `On` / `Off`                       |
| `"Score: " + 10`                   | `E3003`                          | `"Score: " + text(10)`             |
| `# comment`                        | `E1002`                          | `? comment`                        |
| `bind x = 1` twice                 | `E3002`                          | `x = 2` for the second            |
| `bind count = 0`                   | `E4006` reserved                 | `bind total = 0`                   |
| `cast "hi"` at top level           | `E4002`                          | put it inside `spell main()`       |
| `} \n else {`                      | `else` on its own line           | `} else {`                         |
| `xs[-1]`                           | `E2002` no negative indexes      | `xs[length(xs) - 1]`               |
| `1 < x < 10`                       | `E4003`                          | `1 < x and x < 10`                 |
| `for i in 5 { }`                   | `E3003`                          | `for i in range(5) { }`            |
| `print(x)`                         | `E1001`                          | `cast x`                           |
