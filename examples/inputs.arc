? Input: ask() returns text, ask_integer()/ask_decimal() convert
spell main() {
    bind name = ask("Your name: ")
    try {
        bind age = ask_integer("Your age: ")
        cast name + " will be " + text(age + 1) + " next year."
    } catch e {
        cast "That was not a number. (" + e["code"] + ")"
    }
}
