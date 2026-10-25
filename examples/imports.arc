? Imports: run with `arcana run examples/imports.arc` (lib.arc sits next to this file)
import lib
from lib import double

spell main() {
    cast lib.version
    cast lib.double(21)
    cast double(4)
}
