? Standard library tour: reading and writing files
import file
import list
import system

spell main() {
    bind dir = "arcana_demo_files"
    file.make_dir(dir)
    bind path = dir + "/todo.txt"

    file.write_lines(path, ["write docs", "ship 1.0"])
    file.append_text(path, "celebrate\n")

    cast file.read_lines(path)
    cast "size: " + to_text(file.size(path)) + " bytes"
    cast file.list_dir(dir)

    try {
        file.read_text(dir + "/missing.txt")
    } catch e {
        cast e["kind"] + " (" + e["code"] + ")"
    }

    file.remove(path)
    file.remove(dir)
    cast file.exists(dir)
}
