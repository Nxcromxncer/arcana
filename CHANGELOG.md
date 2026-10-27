# Changelog

## 1.0.0

Arcana 1.0 completes the core language with a standard library. The 0.1 language is
unchanged: all 0.1 programs run with identical output and diagnostics.

### New: standard library (`docs/08-standard-library.md`)

Ten modules, built into the binary and loaded with `import`:

* **math**: `sqrt power floor ceil trunc clamp sign abs min max round`, trigonometry
  (`sin cos tan asin acos atan atan2`), `exp log log2 log10 hypot degrees radians`,
  `gcd lcm factorial`, `random random_int random_seed`, constants `pi e tau`.
* **text**: `trim trim_start trim_end contains starts_with ends_with replace substring
  find char_at repeat_text pad_left pad_right lines chars reverse_text code_of from_code`
  (plus `upper lower split join`).
* **list**: `append insert extend remove clear sort reverse shuffle pop sorted reversed
  unique slice concat first last index_of contains sum choice`. Verbs change the list in
  place; other functions return new values.
* **logic**: `is_number is_integer is_decimal is_text is_boolean is_list is_map is_null
  is_empty all any`.
* **convert**: `to_boolean to_list to_map to_binary to_octal to_hex from_binary from_octal
  from_hex parse_number format_number`.
* **error**: `fail warn debug assert_true assert_equal`.
* **system**: `version platform clock args env cwd write exit`.
* **file**: `read_text read_lines write_text append_text write_lines exists is_file is_dir
  size remove make_dir list_dir`.
* **time**: `now sleep format_duration`.
* **date**: dates and datetimes as maps; `today now utc_now local_offset make_date
  make_datetime parse from_timestamp to_timestamp format iso weekday weekday_name
  month_name day_of_year is_leap_year days_in_month add_days add_months add_years add_hours
  add_minutes add_seconds diff_days diff_seconds compare`.

### Changed

* New always-available built-ins: `to_text`, `to_integer`, `to_decimal`, `to_number`,
  `assert`. The 0.1 names `text`, `integer`, `decimal` keep working.
* `round(x, digits)` rounds to decimal places (returns a decimal); `round(x)` is unchanged.
* `arcana run file.arc args...` passes the arguments to the program (`system.args()`);
  `--` ends option parsing. `system.exit(code)` sets the process exit status.
* Standard modules always win over files with the same name. Misspelled module members
  get a "did you mean" suggestion (`E5003`).
* Calling an imported module as if it were a function (`import text` then `text(5)`)
  now explains that the import hides the built-in spell (`E3005`).

### New error codes

* `E2012` (`FileError`): a `file` operation failed (catchable).
* `E2101` (`AssertionError`): `assert`, `assert_true` or `assert_equal` failed (catchable).

### Internals

* Built-ins are plain function pointers (`NativeFn`) instead of a big enum, so new
  modules need no change to the interpreter core.
* Version is `1.0.0` (`arcana version`).
