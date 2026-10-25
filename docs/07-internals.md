# 7. Internals (for contributors)

How the Rust implementation is organised and how to extend it. You do not need any
of this to *write* Arcana programs.

## Pipeline

```
source text
   |  lexer.rs      Lexer::tokenize()           -> Vec<Token>
   v
tokens
   |  parser.rs     parse_source()              -> Program (AST, ast.rs)
   v
AST --+--> checker.rs      `arcana check`: unknown names, redeclarations
      |
      +--> interpreter.rs  `arcana run`: tree-walking evaluation
                              |-- runtime.rs      built-in spells
                              |-- values.rs       Value, Int-backed numbers, maps, ranges
                              +-- environment.rs  scopes
```

`cli.rs` wires the commands together; `main.rs` only forwards the arguments.
`lib.rs` exposes every module, so the interpreter can be embedded in other Rust code
(and is used directly by the tests).

## Module guide

| File             | Responsibility |
|------------------|----------------|
| `tokens.rs`      | `Token`, `Kind`, the keyword table. |
| `lexer.rs`       | Characters to tokens, with line/column tracking (columns count characters). Drops comments, normalises line endings, produces `E1xxx` errors. |
| `ast.rs`         | `Expr`, `Stmt`, `Param`, `Program`, `BinOp`, `UnOp`. Every node carries a `Span`. Literal nodes already hold their runtime `Value`. |
| `parser.rs`      | Recursive descent; one method per grammar rule (`parse_or`, `parse_and`, ... `parse_primary`). A `nest` counter makes newlines insignificant inside brackets. Produces `E4xxx` errors. |
| `checker.rs`     | Static scope analysis for `arcana check`. Spell bodies are checked after the surrounding block, so forward references work. |
| `values.rs`      | `Value` enum, `ArcMap` (insertion-ordered map), `RangeVal`, equality, `to_text`, Python-compatible float formatting. |
| `int.rs`         | `Int`: `i64` fast path with automatic promotion to `BigInt`; exact int/float comparison; floor division and modulo. |
| `environment.rs` | `Env`: `Rc` scope chain with a fast hasher. |
| `interpreter.rs` | Statement execution, expression evaluation, spell calls, imports, `try`/`catch`/`finally`. |
| `runtime.rs`     | Built-in spells as a `BuiltinId` enum dispatched in `call_builtin`. |
| `errors.rs`      | `ArcanaError`, diagnostic rendering, "did you mean" matching (a port of Python's `difflib`). |
| `cli.rs`         | Argument parsing, exit codes, large-stack execution thread. |

## Key design decisions

* **Values.** `Value` is a cheap-to-clone enum. Lists and maps are
  `Rc<RefCell<...>>`, which gives the shared-reference semantics the language
  promises. Reference cycles are not collected (they leak until the process exits);
  that is fine for a short-lived interpreter.
* **Integers** are arbitrary precision. `Int` keeps values that fit in an `i64`
  unboxed and promotes to `num-bigint` only when needed, so ordinary loops do not
  allocate. Invariant: a value that fits in an `i64` is always `Small`.
* **Decimals** are `f64`. `float_repr` reproduces Python's shortest round-trip
  formatting (`1e+16`, `1.5e-07`) so output is identical to the original
  implementation. Operations that would give infinity or NaN raise `E2008`, so those
  values never exist at runtime.
* **Errors** are `Result<T, Box<ArcanaError>>` everywhere (boxed to keep `Result`
  small). `ArcanaError.recoverable` decides whether `try`/`catch` can intercept it.
  Built-ins return errors *without* a span; `call_value` fills in the call site.
* **Control flow** does not use panics or exceptions. `exec` returns a `Flow`
  (`Normal`, `Break`, `Continue`, `Return(value)`), so `finally` is just ordinary
  code that runs after the body's `Result` has been computed.
* **Limits.** `tick()` counts statements and loop rounds for `--max-steps`. Spell
  calls increment a depth counter for `--max-depth`. The CLI runs the interpreter on
  a thread with a 1 GiB (lazily committed) stack, so deep recursion is reported
  as `E2007` instead of crashing. The parser also guards against absurd
  expression nesting.
* **Modules** are cached by absolute path, executed once, with a `loading` stack to
  detect cycles. Each `Spell` remembers its defining `ModuleInfo`, so errors inside
  imported code show the right file and source line.
* **Output** is buffered when stdout is not a terminal and flushed before reading
  input and at exit.

## Building and testing

```
cargo build                 # debug build
cargo build --release       # optimised, ~15-20x faster on call-heavy code
cargo test                  # all tests
cargo test --test interpreter           # one test file
cargo test fizz -- --nocapture          # tests matching a name, with output
```

| Test file                     | Covers |
|-------------------------------|--------|
| `tests/lexer.rs`              | tokens, comments, numbers, escapes, lexical errors |
| `tests/parser.rs`             | precedence trees, syntax errors, reserved words |
| `tests/interpreter.rs`        | language semantics end to end |
| `tests/diagnostics_and_cli.rs`| diagnostic format, `check`, exit codes, imports, the real binary |
| `tests/common/mod.rs`         | helpers: run source on a big-stack thread and capture output |

`diagnostics_and_cli.rs` also runs `arcana check` over every file in `examples/`, so
a broken example fails the test-suite.

## Recipes for extending the language

### Add a built-in spell

1. `runtime.rs`: add a variant to `BuiltinId`.
2. Add its name to `BUILTIN_NAMES` (the array length is part of the type, so
   update the number) and a row in the `table` inside `make_builtins` (same
   length rule): `("name", Variant, min_args, Some(max_args))`.
3. Implement it in the `match` in `call_builtin`. Return
   `Err(type_err("...").boxed())` for type errors, or
   `rt("E2xxx", "Kind", "...")` for a catchable runtime error.
4. Add a test in `tests/interpreter.rs` and a section in `docs/04-builtins.md`.

### Add a keyword

Add it to `KEYWORDS` in `tokens.rs` (and fix the array length). Keywords for
statements are then handled in `Parser::parse_statement`.

### Add a statement

1. `ast.rs`: new `Stmt` variant (and add it to `Stmt::span`).
2. `parser.rs`: parse it in `parse_statement`.
3. `interpreter.rs`: handle it in `exec`.
4. `checker.rs`: handle it in `stmt` so `arcana check` understands its scoping.

### Add an operator

Add to `BinOp` (with `symbol` and `from_symbol`), put it at the right level of the
precedence chain in `parser.rs`, then implement it in `e_binary` / `arith`.

### Add an error code

Pick the next free number in its family (`E1`/`E2`/`E3`/`E4`/`E5`) and create it with
`self.err("E2xxx", message, Some(span))` in the interpreter (add `.help(...)`,
`.note(...)`, and `.recoverable("Kind")` when `try`/`catch` should be able to catch
it). Then document it in `docs/05-errors-and-debugging.md`.

## Embedding the interpreter

```rust
use arcana::interpreter::{Interpreter, Options};

let mut it = Interpreter::new(Options::default());
match it.run_source("spell main() { cast 1 + 2 }", "inline.arc", true) {
    Ok(_) => it.flush(),
    Err(e) => eprintln!("{}", e.render()),
}
```

`Interpreter::with_io` takes any `Write` / `BufRead`, which is how the tests capture
output and provide input.

## Performance notes and ideas

The evaluator is a straightforward AST walk with `HashMap` scopes, which is already
much faster than the original Python implementation. If more speed is ever needed,
the next steps in order of payoff are: resolve variable names to slot indexes
during parsing, intern identifier strings, and compile the AST to bytecode.
