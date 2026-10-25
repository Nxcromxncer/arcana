? A number-guessing loop: ask_integer, try/catch, while, break
spell main() {
    bind secret = 7
    bind tries = 0
    while On {
        try {
            bind guess = ask_integer("Guess the number (1-10): ")
            tries = tries + 1
            if guess == secret {
                cast "Correct! It took " + text(tries) + " tries."
                break
            } else if guess < secret {
                cast "Too low."
            } else {
                cast "Too high."
            }
        } catch err {
            cast "That was not a whole number (" + err["code"] + ")."
        }
    }
}
