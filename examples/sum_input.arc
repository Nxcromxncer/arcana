? Read whole numbers until a blank line, then report the total
spell main() {
    bind total = 0
    bind numbers = 0
    while On {
        bind line = ask("number (blank to stop): ")
        if line == "" {
            break
        }
        try {
            total = total + integer(line)
            numbers = numbers + 1
        } catch err {
            cast "skipping " + line + ": " + err["message"]
        }
    }
    cast "sum of " + text(numbers) + " numbers = " + text(total)
}
