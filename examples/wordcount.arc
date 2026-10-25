? Count words with a map, then show them in first-seen order
spell main() {
    bind sentence = "the wand and the staff and the scroll"
    bind counts = {}
    for word in split(sentence) {
        if word in counts {
            counts[word] = counts[word] + 1
        } else {
            counts[word] = 1
        }
    }
    for word in keys(counts) {
        cast word + ": " + text(counts[word])
    }
}
