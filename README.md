# Arcana 0.1 (Rust)

A fantasy-inspired general-purpose programming language with a tree-walking
interpreter, now written in Rust. It is a faithful port of the Python
implementation: same language, same diagnostics, same exit codes, and about
15-20x faster on call- and loop-heavy programs.

## Build and run

```
cargo build --release              # binary: target/release/arcana
./install.sh                       # or: cargo install --path .

arcana run hello.arc               # run a grimoire
arcana hello.arc                   # shorthand for the same thing
arcana check examples/spells.arc   # static checks, no execution
arcana tokens examples/mana.arc    # print the token stream
arcana version
cargo test                         # full test-suite
```

Without installing: `cargo run --release -- run examples/hello.arc`.

Exit codes: `0` ok, `1` Arcana diagnostic, `2` usage/file problem.
`arcana run` accepts `--max-steps N` and `--max-depth N` (resource limits).
Imports are searched next to the importing file, then in the folders listed in
the `ARCANA_PATH` environment variable.

## Language at a glance

```
? Comments start with a question mark and run to the end of the line.
spell greet(name, title = "Arcanist") {
    return "Welcome, " + title + " " + name
}

spell main() {                       ? entry point: called after the grimoire loads
    bind mana = 80                   ? declare (mutable); redeclaring in the same scope is E3002
    mana = mana - 25                 ? reassign an existing binding
    bind ready = mana >= 50          ? booleans are On and Off
    if ready and not (mana > 100) {  ? conditions must be booleans (no truthiness)
        cast greet("Mira")           ? cast = output
    } else if mana == 0 {
        cast "Empty"
    } else {
        cast "Insufficient mana."
    }
    for i in range(1, 4) { cast i }  ? range(a, b) excludes b; range(a, b, step) supports negatives
    while Off { }
    try { cast 1 / 0 } catch err { cast err["code"] } finally { cast "done" }
}
```

| Topic | Rule |
|---|---|
| Comments | `?` to end of line (a `?` inside a text literal is ordinary) |
| Booleans | `On` / `Off` (reserved, case-sensitive); `true`/`false` produce a helpful E1001 |
| Types | integer (arbitrary precision), decimal (IEEE 754), text, boolean, null, list, map, range, spell, grimoire |
| Equality | no coercion; `1 == 1.0` is `On`, `On == 1` is `Off` |
| Arithmetic | `/` always gives a decimal; `//` floors; `%` takes the sign of the divisor; `+` joins text with text only |
| Scope | lexical, block scoped, closures; no implicit globals |
| Top level | only `spell`, `bind`, `import`; then `main()` runs |
| Input | `ask(prompt)` returns text; `ask_integer`, `ask_decimal` raise a catchable E2010 on bad input |
| Errors | runtime errors (E2xxx) are catchable with `try/catch`; programmer errors (unknown name, type mismatch) are not |
| Imports | `import lib`, `import a.b as x`, `from lib import f`; use `lib.name` for members |
| Reserved for later | `count`, `repeat`, `times` |

Built-ins: `ask ask_integer ask_decimal text integer decimal round abs min max length range append pop keys values copy upper lower split join type_of fail`.

## Documentation

Full guides live in [`docs/`](docs/README.md):

| Guide | |
|---|---|
| [Getting started](docs/01-getting-started.md) | install, first program, CLI |
| [Language tour](docs/02-language-tour.md) | learn Arcana step by step |
| [Language reference](docs/03-language-reference.md) | exact syntax and rules |
| [Built-in spells](docs/04-builtins.md) | every built-in |
| [Errors and debugging](docs/05-errors-and-debugging.md) | error codes, `try`/`catch` |
| [Cookbook](docs/06-cookbook.md) | runnable example programs |
| [Internals](docs/07-internals.md) | for contributors |
| [Cheat sheet](docs/cheatsheet.md) | one page |

## Source layout

```
src/
  main.rs         binary entry point (arcana run ...)
  cli.rs          run / check / tokens / version, option parsing, big-stack thread
  lexer.rs        tokens.rs      source -> tokens
  parser.rs       ast.rs         recursive descent parser -> AST
  checker.rs                     `arcana check` (unknown names, redeclarations)
  interpreter.rs                 tree-walking evaluator, imports, try/catch
  runtime.rs                     built-in spells
  values.rs       environment.rs runtime values, scopes
  int.rs                         arbitrary-precision integers with an i64 fast path
  errors.rs                      diagnostics (E1xxx-E6xxx), "did you mean" suggestions
tests/            lexer, parser, interpreter semantics, diagnostics, CLI, imports
examples/         runnable programs
```

## Notes on the port

- Integers are arbitrary precision (`num-bigint`) behind an `i64` fast path, so
  `2 ** 100` works and small arithmetic does not allocate.
- Decimals print exactly like Python's `repr(float)` (`1e+16`, `1.5e-07`, `0.1`).
- "Did you mean ...?" uses a port of Python's `difflib` matcher, so suggestions
  match the original implementation.
- Deep recursion is safe: programs run on a large-stack thread, and
  `--max-depth` (default 1000) reports E2007 instead of crashing.
- Not in v0.1: file I/O and permission wards (E6xxx reserved), gradual typing,
  Python interop, runes, Academy.
