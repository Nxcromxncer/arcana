? Read 6 whole numbers from the user, then remove the biggest one.
? (If the biggest appears twice, only the first copy is removed.)
import list

spell main() {
    bind numbers = []
    cast "enter 6 whole numbers"
    for i in range(1, 7) {
        append(numbers, ask_integer("number " + to_text(i) + " of 6: "))
    }
    cast "your list: " + to_text(numbers)

    bind biggest = max(numbers)
    list.remove(numbers, biggest)

    cast "removed the biggest (" + to_text(biggest) + ")"
    cast "list now: " + to_text(numbers)
}
