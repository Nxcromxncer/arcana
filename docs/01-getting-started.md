# 1. Getting started

## Install

You need Rust (the `cargo` command). On Arch Linux: `sudo pacman -S rust`, or
get it from <https://rustup.rs>.

```
cd arcana
./install.sh          # builds a release binary and installs ~/.local/bin/arcana
```

Check that it worked:

```
which arcana          # /home/<you>/.local/bin/arcana
arcana version        # Arcana 1.0.0
```

If `arcana` is not found, add `~/.local/bin` to your `PATH`
(`export PATH="$HOME/.local/bin:$PATH"` in `~/.bashrc` or `~/.zshrc`).

Other ways to run it:

| Goal                                  | Command                                              |
|---------------------------------------|------------------------------------------------------|
| Install somewhere else                | `PREFIX=/usr/local ./install.sh`                     |
| Install with cargo                    | `cargo install --path .`                             |
| Run without installing                | `cargo run --release -- run examples/hello.arc`      |
| Uninstall                             | `rm ~/.local/bin/arcana`                             |
| Rebuild after changing the source     | `./install.sh` again                                 |

## Your first grimoire

Create `hello.arc`:

```arc
? The first ritual
spell main() {
    cast "The gates are open."
}
```

Run it:

```
$ arcana run hello.arc
The gates are open.
```

`arcana hello.arc` is a shorthand for the same thing.

What happened:

* `spell main() { ... }` declares a spell (function) called `main`.
* Arcana loads the file, then calls `main()`. **Every runnable grimoire needs a
  `main` spell**; without one you get error `E2004`.
* `cast` prints a value followed by a newline.
* `?` starts a comment that runs to the end of the line.

## The commands

```
arcana run <file.arc> [args...] [--max-steps N] [--max-depth N]
arcana check <file.arc>
arcana tokens <file.arc>
arcana version
```

### `arcana run`

Runs the grimoire. Input for `ask()` is read from the terminal, or from a pipe:

```
printf 'Mira\n41\n' | arcana run examples/inputs.arc
```

Anything after the file name is passed to the program (see
[program arguments](#program-arguments)). Two safety limits are available:

| Option            | Meaning                                                   | Default |
|-------------------|-----------------------------------------------------------|---------|
| `--max-steps N`   | Stop with `E2009` after N statements/loop iterations      | none    |
| `--max-depth N`   | Stop with `E2007` when spells nest deeper than N calls    | 1000    |

`--max-steps` is handy when you suspect an endless loop:

```
arcana run loop.arc --max-steps 100000
```

### Program arguments

Words after the grimoire's name are handed to the program, which reads them with
`system.args()`:

```arc
import system

spell main() {
    cast system.args()
}
```

```
$ arcana run args.arc Mira 3
["Mira", "3"]
$ arcana run args.arc -- --max-steps 5     # after `--`, arcana ignores the rest
["--max-steps", "5"]
```

Arcana's own options (`--max-steps`, `--max-depth`) are recognised anywhere before
a `--`. A program can end with a chosen status using `system.exit(code)`.
`examples/cli_args.arc` shows a complete command-line program.

### `arcana check`

Looks for mistakes **without running** the program: syntax errors, unknown
names (`E1001`) and names declared twice in one scope (`E3002`). Spells can call
each other regardless of the order they are written in.

```
$ arcana check examples/spells.arc
No problems found in examples/spells.arc
```

It does not detect type errors or runtime errors, because those depend on the
values the program computes.

### `arcana tokens`

Prints what the lexer sees, one token per line (`line:column KIND value`).
Useful when a syntax error is confusing.

```
$ echo 'bind x = On ? hi' > t.arc && arcana tokens t.arc
1:1    KW       bind
1:6    IDENT    x
1:8    OP       =
1:10   KW       On
1:17   NEWLINE  '\n'
2:1    EOF      None
```

Note that the comment does not appear: the lexer drops it.

## Exit codes

| Code | Meaning                                                              |
|------|----------------------------------------------------------------------|
| 0    | Success                                                              |
| 1    | Arcana diagnostic (syntax, type, runtime or import error)            |
| 2    | Usage problem or the file could not be read                          |
| 0-255 | whatever the program passes to `system.exit(code)`                  |

This makes Arcana easy to use in shell scripts:

```
arcana check app.arc && arcana run app.arc
```

Diagnostics are printed to **standard error**; everything your program `cast`s
goes to **standard output**, so you can redirect them separately.

## Using several files

Put reusable spells in another grimoire and import them. Given `lib.arc`:

```arc
bind version = "1.0"

spell double(x) {
    return x * 2
}
```

and `main.arc` next to it:

```arc
import lib
from lib import double

spell main() {
    cast lib.version
    cast lib.double(21)
    cast double(4)
}
```

```
1.0
42
8
```

Arcana looks for imported files next to the importing file first, then in every
folder listed in the `ARCANA_PATH` environment variable (separated by `:`):

```
export ARCANA_PATH="$HOME/arcana-libs:$HOME/more-libs"
```

See the [language reference](03-language-reference.md#modules-import) for the
full rules.

## The standard library

Besides your own grimoires, Arcana ships ten standard modules (`math`, `text`, `list`,
`logic`, `convert`, `error`, `system`, `file`, `time`, `date`). They need no
installation:

```arc
import math
import date

spell main() {
    cast math.sqrt(144)
    cast date.format(date.make_date(2026, 10, 27), "%A")
}
```

```
12.0
Tuesday
```

See [Standard library](08-standard-library.md) for every function.

## Suggested workflow

1. Write the grimoire in any editor (`.arc` is plain UTF-8 text).
2. `arcana check file.arc` for a fast sanity check.
3. `arcana run file.arc`.
4. If something is wrong, read the diagnostic: it shows the file, line, column,
   the offending code, and usually a `help:` line with the fix. See
   [Errors and debugging](05-errors-and-debugging.md).

Next: [2. Language tour](02-language-tour.md).
