# 3. Language reference

The precise rules of Arcana 0.1. For a friendlier introduction read the
[language tour](02-language-tour.md) first.

Contents: [Source files](#source-files) - [Lexical structure](#lexical-structure) -
[Values and types](#values-and-types) - [Expressions](#expressions) -
[Statements](#statements) - [Spells](#spells) - [Scope](#scope) -
[Modules](#modules-import) - [Program execution](#program-execution) -
[Grammar summary](#grammar-summary)

## Source files

* Grimoires are UTF-8 text files, conventionally named `*.arc`.
* `\r\n` and `\r` line endings are treated as `\n`; a leading byte-order mark is ignored.
* A file contains declarations only: `import`, `spell` and `bind` (see
  [Program execution](#program-execution)).

## Lexical structure

### Comments

`?` starts a comment that runs to the end of the line. Inside a text literal a
`?` is an ordinary character.

### Whitespace and statement ends

Spaces and tabs separate tokens. **A newline ends a statement.** A newline is
ignored (the statement continues) when it appears:

* inside `( ... )`, `[ ... ]` or the `{ ... }` of a map literal;
* right after a binary operator (`+ - * / // % ** == != < <= > >= in and or`);
* right after `=` in `bind` and in assignments.

```arc
spell main() {
    bind xs = [
        1,
        2,
    ]
    bind total = 1 +
        2
    cast xs
    cast total
}
```

Trailing commas are allowed in lists, maps and argument lists. Two statements on
one line are an error (`E4001`): put each on its own line.

The `{` of a block must be on the same line as its `if`/`while`/`for`/`spell`/`try`
header, and `else`, `catch`, `finally` must follow the closing `}` on the same line.

### Identifiers

Start with a letter or `_`, continue with letters, digits or `_`. Unicode letters
are allowed. Identifiers are case-sensitive.

### Reserved words

```
spell  bind  cast  if  else  while  for  in  return  break  continue
try  catch  finally  import  from  as  On  Off  null  and  or  not
count  repeat  times            (reserved for future use)
```

A reserved word cannot be used as a name (`E4006`). `true` and `false` are **not**
reserved; they are ordinary names that happen to be undefined, so using them gives
`E1001` with a hint about `On`/`Off`.

### Numbers

| Form                 | Type    | Examples                    |
|----------------------|---------|-----------------------------|
| digits               | integer | `0`, `42`, `1_000_000`      |
| digits `.` digits    | decimal | `2.5`, `0.001`              |
| with exponent        | decimal | `1e3`, `2.5e-1`, `6E+2`     |

* `_` may separate digits (`1_000`) but not start, end, or double up.
* A decimal point needs digits on both sides: `.5` and `5.` are invalid.
* A number followed directly by a letter (`5x`) is `E1005`.
* There are no negative literals; `-5` is the unary minus applied to `5`.
* A decimal literal too large for 64-bit floating point is `E1005`.

### Text literals

Double quotes only. Escapes:

| Escape      | Meaning                                  |
|-------------|------------------------------------------|
| `\n`        | newline                                  |
| `\t`        | tab                                      |
| `\\`        | backslash                                |
| `\"`        | double quote                             |
| `\u{2728}`  | the Unicode code point with that hex value |

Text cannot span lines (`E1003`), and unknown escapes are `E1004`. Single quotes,
`#`, `;`, `!`, `&` and `|` are rejected with a hint about the Arcana spelling.

### Operators and punctuation

```
+  -  *  /  //  %  **     ==  !=  <  <=  >  >=
=  (  )  [  ]  {  }  ,  :  .
and  or  not  in
```

## Values and types

| Type     | Written as            | Truth in conditions |
|----------|-----------------------|---------------------|
| integer  | `42`                  | not allowed         |
| decimal  | `2.5`                 | not allowed         |
| text     | `"hi"`                | not allowed         |
| boolean  | `On` / `Off`          | the only allowed    |
| null     | `null`                | not allowed         |
| list     | `[1, 2]`              | not allowed         |
| map      | `{"a": 1}`            | not allowed         |
| range    | `range(1, 4)`         | not allowed         |
| spell    | `main`, `length`      | not allowed         |
| grimoire | `lib`                 | not allowed         |

* **integer**: arbitrary precision. `2 ** 100` is exact.
* **decimal**: IEEE 754 double. Results that overflow to infinity or are not real
  numbers raise `E2008`. NaN and infinity cannot be produced.
* **text**: sequence of Unicode characters; indexing and `length` count characters,
  not bytes. Immutable.
* **list**: ordered and mutable. Names share lists; use `copy(x)` for a shallow copy.
* **map**: mutable, remembers insertion order. Keys must be text or integer
  (`E3003` otherwise). Key `1` and key `"1"` are different.
* **range**: an immutable arithmetic sequence. Supports `length`, indexing, `in`
  and `for`. Prints as `range(start, stop, step)`.
* **spell**: user-defined (`<spell name>`) or built-in (`<built-in spell name>`).

### Equality

`==` and `!=` never convert types:

* integer and decimal compare by numeric value (`1 == 1.0` is `On`);
* booleans only equal booleans; `null` only equals `null`;
* texts compare by characters; lists element by element; maps by key set and values;
  ranges as sequences;
* spells and grimoires compare by identity;
* any two values of different types are not equal (`On == 1` is `Off`).

### Ordering

`<  <=  >  >=` work on two numbers (integer/decimal may be mixed) or two texts
(compared by Unicode code point). Anything else is `E3003`. Comparisons cannot be
chained (`E4003`).

## Expressions

### Precedence

From loosest to tightest binding:

| Level | Operators                          | Associativity |
|-------|------------------------------------|---------------|
| 1     | `or`                               | left          |
| 2     | `and`                              | left          |
| 3     | `not`                              | prefix        |
| 4     | `==  !=  <  <=  >  >=  in`         | none (no chaining) |
| 5     | `+  -`                             | left          |
| 6     | `*  /  //  %`                      | left          |
| 7     | unary `-`                          | prefix        |
| 8     | `**`                               | right         |
| 9     | call `f(x)`, index `x[i]`, member `m.name` | left  |

Consequences: `-2 ** 2` is `-4`; `2 ** 3 ** 2` is `512`; `2 ** -1` is `0.5`;
`not a == b` means `not (a == b)`; `a or b and c` means `a or (b and c)`.

### Arithmetic

| Operator | integer op integer       | involving a decimal    |
|----------|--------------------------|------------------------|
| `+ - *`  | integer                  | decimal                |
| `/`      | **decimal**              | decimal                |
| `//`     | integer, floor           | decimal, floor         |
| `%`      | integer, sign of divisor | decimal, sign of divisor |
| `**`     | integer (decimal if the exponent is negative) | decimal |

* `/`, `//`, `%` by zero: `E2001` (catchable).
* `0 ** negative`: `E2001`. Negative base with a fractional exponent
  (`(-8) ** 0.5`): `E2008`. Results that are too large: `E2008`.
* Integer powers whose result would exceed about a million bits are `E2008`
  instead of exhausting memory.
* `+` on two texts joins them. Any other `+` needs two numbers.

### Logic

`and` / `or` / `not` need booleans (`E3001` otherwise) and `and` / `or`
short-circuit: the right side is not evaluated when the left decides the result.

### Membership: `in`

| Right side | Left side        | Meaning                    |
|------------|------------------|----------------------------|
| list       | any              | some item `==` the value   |
| map        | text or integer  | the key exists             |
| text       | text             | substring test             |
| range      | integer          | the value is in the range  |

### Calls, indexing, members

* `f(a, b)` calls a spell. Arguments are evaluated left to right before the call.
* `x[i]` indexes a list, text (one-character text), range, or map (by key).
  List/text/range indexes must be integers from `0` to `length - 1` (`E2002`
  otherwise, catchable). A missing map key is `E2005` (catchable).
* `m.name` reads a name from an imported grimoire (`E5003` if absent). Nothing
  else has members.

### Literals

* List `[a, b, c]`, map `{k: v, ...}` (duplicate keys: the last value wins and
  the key keeps its first position).

## Statements

| Statement                           | Effect                                           |
|-------------------------------------|--------------------------------------------------|
| `bind name = expr`                  | declare a variable in the current scope          |
| `name = expr`                       | assign to an existing variable                   |
| `list[i] = expr`, `map[k] = expr`   | change an element / add or change a key          |
| `cast expr`                         | print the value and a newline                    |
| `if c { } else if c { } else { }`   | conditional (conditions must be booleans)        |
| `while c { }`                       | loop while the condition is `On`                 |
| `for x in coll { }`                 | loop over a list, text, map (keys) or range      |
| `break` / `continue`                | leave / skip to the next round of the nearest loop; only legal inside a loop |
| `return` / `return expr`            | leave the spell                                  |
| `try { } catch e { } finally { }`   | handle errors (needs `catch` or `finally`)       |
| `spell name(params) { }`            | declare a spell                                  |
| `import ...`                        | top level only                                   |
| `f(args)`                           | a call used as a statement                       |

A bare expression that is not a call (like `1 + 2`) is `E4004`: "this expression
has no effect". Assignment targets must be a name or an index expression.

`break` and `continue` inside a spell declared **within** a loop do not see the
outer loop; they need their own (`E2003`).

### `try` / `catch` / `finally`

1. The `try` block runs.
2. If it raises a **catchable** error and a `catch` clause exists, the clause runs
   with its variable bound to a map `{"code", "kind", "message"}`.
3. `finally` runs last, always, even after `return`, `break`, `continue` or an
   uncaught error. A `return` inside `finally` overrides everything.

A `try` with only `finally` lets errors continue to propagate afterwards.
Catchable errors are exactly the `E2xxx` runtime errors listed in
[Errors and debugging](05-errors-and-debugging.md#which-errors-can-be-caught).

### `for` semantics

The collection is evaluated once. Lists are iterated as a snapshot (changing the
list inside the loop does not change the iteration), maps yield their keys in
insertion order, text yields one-character texts, ranges yield integers. The loop
variable is a fresh binding per round, scoped to the loop body.

## Spells

```
spell name(required, optional = default_expr) {
    ...
}
```

* Required parameters must come before optional ones; names must be unique (`E3002`).
* Default expressions are evaluated at **call time**, in the spell's own scope, so
  a default may refer to earlier parameters.
* Calling with too few or too many arguments is `E3004`.
* The result of a spell without `return` is `null`.
* Recursion is limited by `--max-depth` (default 1000 nested calls, `E2007`).
* Spells are first-class values: store them in variables, lists and maps, pass and
  return them.
* Nested spells capture the scope they were declared in (closures) and can modify
  the variables there.
* Built-in spells check their own argument counts and types, reported as `E3004`
  and `E3003`.

## Scope

* Scopes are **lexical and block based**: every `{ }` block, every loop round and
  every spell call starts a new scope that can see the scopes around it.
* A name must be declared with `bind` (or as a parameter / loop variable / catch
  variable) before use; there are no implicit globals.
* `bind` always creates a variable in the *current* scope, and may *shadow* a
  variable from an outer scope. Redeclaring in the same scope is `E3002`.
* Assignment (`x = ...`) updates the nearest existing variable with that name.
* Built-in spell names live in the outermost scope; you may shadow them with
  `bind length = 3`, but you may not assign to them (`E3002`).
* Top-level spells can be called before they are declared in the file, because
  `main` only runs after the whole file has been loaded. Spells declared *inside*
  another spell exist only from the point where their declaration runs.

## Modules: `import`

```arc
import lib                 ? lib.arc          -> use lib.name
import pkg.deep as d       ? pkg/deep.arc     -> use d.name
from lib import a, b       ? names copied into this file's scope
```

* `a.b.c` maps to the path `a/b/c.arc`.
* The file is looked up next to the importing file first, then in every directory
  of the `ARCANA_PATH` environment variable. Not found: `E5001`; the diagnostic
  lists the places searched.
* A module's top-level code (its `bind`s and `spell`s) runs once, the first time
  it is imported; later imports reuse the same module. Its own `main` is **not**
  called.
* `import m` declares the name `m` (or the alias after `as`); `from m import x`
  declares `x`. These names follow the normal rule against redeclaration (`E3002`).
* Circular imports are `E5002`. Importing a name the module does not define is
  `E5003`.
* Imports are allowed only at the top level of a file (`E4001` elsewhere).
* Errors raised inside an imported file are reported with **that** file's name,
  line and source text.

## Program execution

1. The file is lexed and parsed. Any error here stops everything (`E1xxx`, `E4xxx`).
2. The top-level declarations run in order: imports are loaded, `bind`s are
   evaluated, `spell`s are created.
3. `main()` is called with no arguments. If it does not exist, `E2004`.
4. An uncaught error prints a diagnostic to standard error and ends the process
   with exit code 1.

Top level may contain only `import`, `spell` and `bind`. Anything else is `E4002`
("put this statement inside spell main").

## Grammar summary

An informal EBNF. `NL` is a newline; `{ x }` means zero or more, `[ x ]` optional.

```
grimoire    = { import | spell | bind } ;
import      = "import" dotted [ "as" NAME ]
            | "from" dotted "import" NAME { "," NAME } ;
dotted      = NAME { "." NAME } ;
spell       = "spell" NAME "(" [ param { "," param } ] ")" block ;
param       = NAME [ "=" expr ] ;
block       = "{" { statement NL } "}" ;
statement   = bind | assign | "cast" expr | if | while | for
            | "return" [ expr ] | "break" | "continue" | try | spell | call ;
bind        = "bind" NAME "=" expr ;
assign      = ( NAME | postfix "[" expr "]" ) "=" expr ;
if          = "if" expr block { "else" "if" expr block } [ "else" block ] ;
while       = "while" expr block ;
for         = "for" NAME "in" expr block ;
try         = "try" block [ "catch" NAME block ] [ "finally" block ] ;

expr        = or ;
or          = and { "or" and } ;
and         = not { "and" not } ;
not         = "not" not | comparison ;
comparison  = additive [ ( "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" ) additive ] ;
additive    = term { ( "+" | "-" ) term } ;
term        = unary { ( "*" | "/" | "//" | "%" ) unary } ;
unary       = "-" unary | power ;
power       = postfix [ "**" unary ] ;
postfix     = primary { "(" [ args ] ")" | "[" expr "]" | "." NAME } ;
primary     = INTEGER | DECIMAL | TEXT | "On" | "Off" | "null" | NAME
            | "(" expr ")" | "[" [ args ] "]" | "{" [ pair { "," pair } ] "}" ;
args        = expr { "," expr } ;
pair        = expr ":" expr ;
```

(A trailing comma is accepted in `args` and map pairs.)
