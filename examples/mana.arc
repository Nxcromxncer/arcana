? Variables, arithmetic, conditions and Booleans
spell main() {
    bind mana = 80          ? current mana
    bind cost = 25
    bind remaining = mana - cost
    cast remaining          ? 55

    bind ready = mana >= 50
    cast ready              ? On

    if ready and not (remaining < 10) {
        cast "The portal opens."
    } else {
        cast "Insufficient mana."
    }
    cast "Is that all? Yes!"   ? a '?' inside text is kept
}
