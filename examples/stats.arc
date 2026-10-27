? Standard library tour: math, list and convert together
import math
import list
import convert

spell mean(items) {
    return list.sum(items) / length(items)
}

spell main() {
    bind scores = [72, 95, 88, 61, 95, 79]

    cast "count:   " + to_text(length(scores))
    cast "sum:     " + to_text(list.sum(scores))
    cast "mean:    " + convert.format_number(mean(scores), 2)
    cast "min/max: " + to_text(min(scores)) + " / " + to_text(max(scores))
    cast "sorted:  " + to_text(list.sorted(scores, On))
    cast "unique:  " + to_text(list.unique(scores))

    bind variance = 0.0
    for s in scores {
        variance = variance + math.power(s - mean(scores), 2)
    }
    cast "std dev: " + convert.format_number(math.sqrt(variance / length(scores)), 3)
    cast "hex of the top score: " + convert.to_hex(max(scores))
}
