? Lists, maps and text built-ins working together
spell describe(item) {
    return upper(item["name"]) + " x" + text(item["qty"])
}

spell main() {
    bind inventory = [
        {"name": "potion", "qty": 3},
        {"name": "scroll", "qty": 12},
        {"name": "wand", "qty": 1},
    ]

    bind total = 0
    for item in inventory {
        cast describe(item)
        total = total + item["qty"]
    }
    cast "Total items: " + text(total)

    bind names = []
    for item in inventory { append(names, item["name"]) }
    cast join(names, ", ")

    bind stock = {}
    for item in inventory { stock[item["name"]] = item["qty"] }
    cast stock
    cast "wand" in stock
    cast keys(stock)
}
