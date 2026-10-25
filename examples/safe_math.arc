? Catching runtime errors, raising your own with fail(), and finally
spell safe_divide(a, b) {
    try {
        return a / b
    } catch err {
        cast "Caught " + err["kind"] + " (" + err["code"] + "): " + err["message"]
        return null
    } finally {
        cast "division attempted"
    }
}

spell withdraw(balance, amount) {
    if amount > balance {
        fail("insufficient mana")
    }
    return balance - amount
}

spell main() {
    cast safe_divide(9, 3)
    cast safe_divide(1, 0)
    try {
        cast withdraw(50, 80)
    } catch e {
        cast e["kind"] + ": " + e["message"]
    }
}
