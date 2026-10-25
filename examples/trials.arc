? Error handling
spell safe_divide(a, b) {
    try {
        return a / b
    } catch err {
        cast "Caught " + err["code"] + ": " + err["message"]
        return null
    } finally {
        cast "division attempted"
    }
}

spell main() {
    cast safe_divide(10, 4)
    cast safe_divide(1, 0)
    try {
        fail("The ward has been broken")
    } catch e {
        cast e["kind"]
    }
}
