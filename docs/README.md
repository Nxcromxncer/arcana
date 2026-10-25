# Arcana documentation

Arcana is a small, strict, fantasy-flavoured programming language. Programs are
called **grimoires** (files ending in `.arc`), functions are **spells**, and
output is made with `cast`.

```
spell main() {
    cast "The gates are open."
}
```

## Where to start

| If you want to...                                   | Read                                                   |
|-----------------------------------------------------|--------------------------------------------------------|
| Install Arcana and run your first program           | [1. Getting started](01-getting-started.md)            |
| Learn the language step by step                     | [2. Language tour](02-language-tour.md)                |
| Look up exact syntax, operators and rules           | [3. Language reference](03-language-reference.md)      |
| Look up a built-in spell (`length`, `split`, ...)   | [4. Built-in spells](04-builtins.md)                   |
| Understand an error message or catch errors         | [5. Errors and debugging](05-errors-and-debugging.md)  |
| Copy working programs for common tasks              | [6. Cookbook](06-cookbook.md)                          |
| Change the interpreter itself                       | [7. Internals](07-internals.md)                        |
| Keep a one-page reminder next to your editor        | [Cheat sheet](cheatsheet.md)                           |

## Ideas worth knowing before you start

These are the rules that most often surprise people coming from other languages.

1. **Booleans are `On` and `Off`**, capitalised. `true` and `false` do not exist.
2. **No truthiness.** `if 1 { }` is an error; conditions must be `On` or `Off`.
   Write `if x != 0 { }` or `if length(items) > 0 { }`.
3. **Comments start with `?`**, not `#`.
4. **A statement ends at the end of the line.** No semicolons.
5. **Top level holds declarations only** (`spell`, `bind`, `import`). Your code
   runs inside `spell main() { ... }`.
6. **`+` joins text with text only.** Use `text(value)` to convert first.
7. **`/` always gives a decimal** (`6 / 3` is `2.0`); use `//` for integer division.
8. **Errors come in two kinds.** Runtime errors (`E2xxx`, like dividing by zero)
   can be caught with `try`/`catch`. Programmer errors (unknown name, wrong type)
   cannot; fix the code instead.
9. **`count`, `repeat` and `times` are reserved words**, so they cannot be used
   as variable names.

## Runnable examples

Every program in the `examples/` folder is referenced from these docs and is
verified by the test-suite (`cargo test` runs `arcana check` on all of them).

```
arcana run examples/hello.arc
arcana run examples/fizzbuzz.arc
```
