? Standard library tour: the text module
import text

spell title_case(sentence) {
    bind words = []
    for w in text.split(sentence) {
        append(words, upper(text.substring(w, 0, 1)) + lower(text.substring(w, 1)))
    }
    return join(words, " ")
}

spell main() {
    bind raw = "   the QUICK brown   fox  "
    bind clean = text.trim(raw)
    cast "[" + clean + "]"
    cast title_case(clean)
    cast text.replace(clean, " ", "_")
    cast text.pad_left(to_text(42), 8, "0")
    cast text.pad_right("name", 10, ".") + "|"
    cast text.repeat_text("=-", 10)
    cast text.find(clean, "brown")
    cast text.starts_with(clean, "the")
    cast text.reverse_text("arcana")
}
