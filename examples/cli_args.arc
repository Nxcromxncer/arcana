? Command-line arguments and exit codes:  arcana run examples/cli_args.arc Mira 3
import system

spell main() {
    bind args = system.args()
    if length(args) == 0 {
        cast "usage: arcana run cli_args.arc NAME [TIMES]"
        system.exit(2)
    }
    bind rounds = 1
    if length(args) > 1 {
        rounds = to_integer(args[1])
    }
    for i in range(rounds) {
        cast "Hello, " + args[0] + "! (" + to_text(i + 1) + "/" + to_text(rounds) + ")"
    }
}
