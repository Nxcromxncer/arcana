mod common;
use common::*;

// ---- global built-ins added in 1.0 ----------------------------------------------------------

#[test]
fn conversion_aliases_and_to_number() {
    assert_eq!(
        main_body("cast to_text(42) + \"!\"\ncast to_integer(4.0)\ncast to_decimal(\"3.5\")\ncast to_number(\"25\")\ncast to_number(\"2.5\")\ncast text(1)"),
        "42!\n4\n3.5\n25\n2.5\n1\n"
    );
    assert_eq!(body_error("cast to_number(\"abc\")").code, "E2011");
    assert_eq!(body_error("cast to_number([1])").code, "E3003");
}

#[test]
fn round_with_digits() {
    assert_eq!(main_body("cast round(2.5)\ncast round(-2.5)\ncast round(3.14159, 2)\ncast round(2.675, 2)\ncast round(5, 2)"), "3\n-3\n3.14\n2.68\n5.0\n");
    assert_eq!(body_error("cast round(1.5, 20)").code, "E2006");
}

#[test]
fn assert_builtin() {
    assert_eq!(main_body("assert(1 + 1 == 2)"), "");
    assert_eq!(main_body("try { assert(Off) } catch e { cast e[\"code\"] + \" \" + e[\"kind\"] + \" \" + e[\"message\"] }"), "E2101 AssertionError Assertion failed\n");
    assert_eq!(main_body("try { assert(1 > 2, \"math is broken\") } catch e { cast e[\"message\"] }"), "math is broken\n");
    assert_eq!(body_error("assert(1)").code, "E3003");
    assert_eq!(body_error("assert(Off)").code, "E2101");
}

// ---- modules in general -----------------------------------------------------------------------

#[test]
fn std_modules_import_in_all_forms() {
    let src = "import math\nimport math as m\nfrom math import sqrt, pi\nfrom text import upper as _x\nspell main() {}\n";
    // `as` is only valid for whole-module imports, so the last line is a syntax error
    assert!(run_with(src, "", 100, None).is_err());
    let ok = "import math\nimport text as t\nfrom math import sqrt, pi\nspell main() {\n cast math.sqrt(16)\n cast t.upper(\"a\")\n cast sqrt(9)\n cast pi > 3\n cast type_of(math)\n cast math\n}\n";
    assert_eq!(run(ok), "4.0\nA\n3.0\nOn\ngrimoire\n<grimoire math>\n");
}

#[test]
fn std_module_name_suggestions() {
    let e = error_of("import math\nspell main() { cast math.sqroot(4) }");
    assert_eq!(e.code, "E5003");
    assert!(e.help.unwrap().contains("did you mean 'sqrt'?"));
    let e = error_of("from list import sortt\nspell main() { }");
    assert!(e.help.unwrap().contains("'sort'"));
    assert_eq!(error_of("import mathh\nspell main() { }").code, "E5001");
}

#[test]
fn builtin_functions_are_values_with_correct_arity_errors() {
    let e = std_error("cast math.sqrt()");
    assert_eq!(e.code, "E3004");
    assert!(std_body("cast math.sqrt").starts_with("<built-in spell sqrt>"));
}

// ---- math ----------------------------------------------------------------------------------------

#[test]
fn math_basics() {
    assert_eq!(
        std_body("cast math.sqrt(144)\ncast math.power(2, 5)\ncast math.power(2, -2)\ncast math.clamp(120, 0, 100)\ncast math.clamp(-5, 0, 100)\ncast math.clamp(5, 0, 100)\ncast math.abs(-7)\ncast math.min(3, 1, 2)\ncast math.max([4, 9])"),
        "12.0\n32\n0.25\n100\n0\n5\n7\n1\n9\n"
    );
    assert_eq!(std_body("cast math.floor(2.7)\ncast math.floor(-2.1)\ncast math.ceil(2.1)\ncast math.ceil(-2.9)\ncast math.trunc(-2.9)\ncast math.floor(7)\ncast math.sign(-3)\ncast math.sign(0)\ncast math.sign(2.5)"), "2\n-3\n3\n-2\n-2\n7\n-1\n0\n1\n");
    assert_eq!(std_body("cast math.gcd(12, 18)\ncast math.lcm(4, 6)\ncast math.factorial(20)\ncast math.factorial(0)"), "6\n12\n2432902008176640000\n1\n");
    assert_eq!(std_body("cast math.pi\ncast math.e\ncast math.tau"), "3.141592653589793\n2.718281828459045\n6.283185307179586\n");
}

#[test]
fn math_trig_and_logs() {
    assert_eq!(
        std_body("cast math.sin(0)\ncast math.cos(0)\ncast math.log(8, 2)\ncast math.log10(1000)\ncast math.log2(8)\ncast math.exp(0)\ncast math.hypot(3, 4)\ncast math.degrees(math.pi)\ncast math.radians(180) == math.pi\ncast math.atan2(1, 1) * 4 == math.pi"),
        "0.0\n1.0\n3.0\n3.0\n3.0\n1.0\n5.0\n180.0\nOn\nOn\n"
    );
}

#[test]
fn math_errors() {
    assert_eq!(std_error("cast math.sqrt(0 - 1)").code, "E2008");
    assert_eq!(std_error("cast math.log(0)").code, "E2008");
    assert_eq!(std_error("cast math.asin(2)").code, "E2008");
    assert_eq!(std_error("cast math.factorial(-1)").code, "E2006");
    assert_eq!(std_error("cast math.clamp(1, 5, 2)").code, "E2006");
    assert_eq!(std_error("cast math.power(0, -1)").code, "E2001");
    assert_eq!(std_error("cast math.sqrt(\"4\")").code, "E3003");
    assert_eq!(std_error("cast math.power(10.0, 400)").code, "E2008");
    assert_eq!(std_error("cast math.log(5, 1)").code, "E2006");
}

#[test]
fn math_random() {
    assert_eq!(
        std_body("math.random_seed(7)\nbind a = math.random_int(1, 1000)\nbind b = math.random()\nmath.random_seed(7)\ncast a == math.random_int(1, 1000)\ncast b == math.random()\ncast b >= 0.0 and b < 1.0"),
        "On\nOn\nOn\n"
    );
    assert_eq!(std_body("bind ok = On\nfor i in range(200) {\n bind r = math.random_int(3, 5)\n if r < 3 or r > 5 { ok = Off }\n}\ncast ok\ncast math.random_int(4, 4)"), "On\n4\n");
    assert_eq!(std_error("cast math.random_int(5, 1)").code, "E2006");
}

// ---- text ----------------------------------------------------------------------------------------

#[test]
fn text_functions() {
    assert_eq!(
        std_body("cast t.upper(t.trim(\"  arcana  \"))\ncast \"[\" + t.trim_start(\"  a \") + \"]\"\ncast \"[\" + t.trim_end(\" a  \") + \"]\"\ncast t.replace(\"a-b-c\", \"-\", \"+\")\ncast t.contains(\"hello\", \"ell\")\ncast t.starts_with(\"arcana\", \"arc\")\ncast t.ends_with(\"arcana\", \"ana\")"),
        "ARCANA\n[a ]\n[ a]\na+b+c\nOn\nOn\nOn\n"
    );
    assert_eq!(
        std_body("cast t.substring(\"hello world\", 6)\ncast t.substring(\"hello\", 1, 3)\ncast t.repeat_text(\"ab\", 3)\ncast t.char_at(\"arcana\", 2)\ncast t.find(\"hello\", \"ll\")\ncast t.find(\"hello\", \"z\")\ncast t.find(\"héllo\", \"l\")"),
        "world\nel\nababab\nc\n2\nnull\n2\n"
    );
    assert_eq!(
        std_body("cast t.pad_left(\"7\", 3, \"0\")\ncast t.pad_right(\"ab\", 5, \".\")\ncast t.pad_left(\"abcdef\", 3)\ncast t.reverse_text(\"abc\")\ncast t.lines(\"a\\nb\\n\")\ncast t.chars(\"héy\")\ncast t.code_of(\"A\")\ncast t.from_code(10024)\ncast t.split(\"a,b\", \",\")\ncast t.join([\"x\", \"y\"], \"-\")"),
        "007\nab...\nabcdef\ncba\n[\"a\", \"b\"]\n[\"h\", \"é\", \"y\"]\n65\n✨\n[\"a\", \"b\"]\nx-y\n"
    );
}

#[test]
fn text_errors() {
    assert_eq!(std_error("cast t.substring(\"abc\", 2, 9)").code, "E2002");
    assert_eq!(std_error("cast t.substring(\"abc\", 3, 1)").code, "E2002");
    assert_eq!(std_error("cast t.char_at(\"abc\", 3)").code, "E2002");
    assert_eq!(std_error("cast t.replace(\"abc\", \"\", \"x\")").code, "E2006");
    assert_eq!(std_error("cast t.repeat_text(\"a\", 0 - 1)").code, "E2006");
    assert_eq!(std_error("cast t.repeat_text(\"abc\", 100000000)").code, "E2008");
    assert_eq!(std_error("cast t.pad_left(\"a\", 3, \"xy\")").code, "E2006");
    assert_eq!(std_error("cast t.upper(5)").code, "E3003");
    assert_eq!(std_error("cast t.from_code(1114112)").code, "E2006");
}

// ---- list ------------------------------------------------------------------------------------------

#[test]
fn list_in_place_versus_new() {
    // verbs change the list and return null; adjectives return a new list
    assert_eq!(
        std_body("bind s = [30, 10, 20]\ncast l.sort(s)\ncast s\ncast l.sorted([3, 1, 2])\ncast l.reversed([1, 2, 3])\nbind r = [1, 2, 3]\nl.reverse(r)\ncast r\nbind o = [3, 1]\nbind n = l.sorted(o)\ncast o\ncast n"),
        "null\n[10, 20, 30]\n[1, 2, 3]\n[3, 2, 1]\n[3, 2, 1]\n[3, 1]\n[1, 3]\n"
    );
    assert_eq!(std_body("bind s = [1, 3, 2]\nl.sort(s, On)\ncast s\nbind w = [\"pear\", \"apple\"]\nl.sort(w)\ncast w\ncast l.sorted([2.5, 1, 3])"), "[3, 2, 1]\n[\"apple\", \"pear\"]\n[1, 2.5, 3]\n");
}

#[test]
fn list_functions() {
    assert_eq!(
        std_body("bind s = [10, 10, 20, 30]\ncast l.unique(s)\ncast l.sum(s)\ncast l.first(s)\ncast l.last(s)\ncast l.index_of(s, 20)\ncast l.index_of(s, 99)\ncast l.slice(s, 1, 3)\ncast l.slice(s, 2)\ncast l.contains(s, 30)\ncast l.concat([1], [2, 3])\ncast s"),
        "[10, 20, 30]\n70\n10\n30\n2\nnull\n[10, 20]\n[20, 30]\nOn\n[1, 2, 3]\n[10, 10, 20, 30]\n"
    );
    assert_eq!(
        std_body("bind s = [10, 20, 30]\nl.remove(s, 20)\nl.insert(s, 0, 5)\nl.extend(s, [1, 2])\ncast s\ncast l.pop(s)\nl.append(s, 9)\ncast s\nl.clear(s)\ncast s\ncast l.unique([1, 1.0, \"a\", \"a\", [1], [1]])\ncast l.sum([1, 2.5])\ncast l.sum([])"),
        "[5, 10, 30, 1, 2]\n2\n[5, 10, 30, 1, 9]\n[]\n[1, \"a\", [1]]\n3.5\n0\n"
    );
}

#[test]
fn list_random() {
    assert_eq!(
        std_body("math.random_seed(1)\nbind xs = [1, 2, 3, 4, 5]\nl.shuffle(xs)\ncast l.sum(xs)\ncast length(xs)\ncast l.choice([7]) \ncast l.sorted(xs)"),
        "15\n5\n7\n[1, 2, 3, 4, 5]\n"
    );
}

#[test]
fn list_errors() {
    assert_eq!(std_error("cast l.first([])").code, "E2002");
    assert_eq!(std_error("cast l.last([])").code, "E2002");
    assert_eq!(std_error("l.remove([1], 5)").code, "E2006");
    assert_eq!(std_error("l.sort([1, \"a\"])").code, "E3003");
    assert_eq!(std_error("cast l.sum([\"a\", \"b\"])").code, "E3003");
    assert_eq!(std_error("cast l.slice([1, 2], 1, 5)").code, "E2002");
    assert_eq!(std_error("cast l.slice([1, 2], 2, 1)").code, "E2002");
    assert_eq!(std_error("l.insert([1], 5, 0)").code, "E2002");
    assert_eq!(std_error("cast l.choice([])").code, "E2002");
    assert_eq!(std_error("l.sort(5)").code, "E3003");
}

// ---- logic -------------------------------------------------------------------------------------------

#[test]
fn logic_functions() {
    assert_eq!(
        std_body("cast logic.is_number(1.5)\ncast logic.is_number(\"1\")\ncast logic.is_integer(1)\ncast logic.is_decimal(1)\ncast logic.is_text(\"a\")\ncast logic.is_boolean(On)\ncast logic.is_list([])\ncast logic.is_map({})\ncast logic.is_null(null)\ncast logic.is_null(0)"),
        "On\nOff\nOn\nOff\nOn\nOn\nOn\nOn\nOn\nOff\n"
    );
    assert_eq!(
        std_body("cast logic.is_empty(\"\")\ncast logic.is_empty([1])\ncast logic.is_empty({})\ncast logic.is_empty(range(0))\ncast logic.all([On, On])\ncast logic.all([On, Off])\ncast logic.all([])\ncast logic.any([Off, On])\ncast logic.any([])"),
        "On\nOff\nOn\nOn\nOn\nOff\nOn\nOn\nOff\n"
    );
    let e = std_error("cast logic.all([1, 2])");
    assert_eq!(e.code, "E3003");
    assert!(e.help.unwrap().contains("truthiness"));
    assert_eq!(std_error("cast logic.is_empty(5)").code, "E3003");
}

// ---- convert -------------------------------------------------------------------------------------------

#[test]
fn convert_functions() {
    assert_eq!(
        std_body("cast c.to_boolean(\"yes\")\ncast c.to_boolean(\"Off\")\ncast c.to_boolean(0)\ncast c.to_boolean(2.5)\ncast c.to_boolean(null)\ncast c.to_boolean(On)"),
        "On\nOff\nOff\nOn\nOff\nOn\n"
    );
    assert_eq!(
        std_body("cast c.to_list(\"ab\")\ncast c.to_list({\"a\": 1, \"b\": 2})\ncast c.to_list(range(3))\ncast c.to_map([[\"a\", 1], [2, \"b\"]])\nbind src = [1]\nbind dup = c.to_list(src)\nappend(dup, 2)\ncast src"),
        "[\"a\", \"b\"]\n[[\"a\", 1], [\"b\", 2]]\n[0, 1, 2]\n{\"a\": 1, 2: \"b\"}\n[1]\n"
    );
    assert_eq!(
        std_body("cast c.to_binary(10)\ncast c.to_binary(0 - 5)\ncast c.to_hex(255)\ncast c.to_octal(8)\ncast c.from_binary(\"0b1010\")\ncast c.from_binary(\"101\")\ncast c.from_hex(\"FF\")\ncast c.from_hex(\"0xff\")\ncast c.from_octal(\"17\")\ncast c.from_hex(\"-0x10\")"),
        "0b1010\n-0b101\n0xff\n0o10\n10\n5\n255\n255\n15\n-16\n"
    );
    assert_eq!(
        std_body("cast c.parse_number(\"12\")\ncast c.parse_number(\" 1.5 \")\ncast c.parse_number(\"abc\")\ncast c.format_number(3.14159, 2)\ncast c.format_number(5, 2)\ncast c.format_number(2.5, 0)\ncast c.format_number(1e15, 1)"),
        "12\n1.5\nnull\n3.14\n5.00\n2\n1000000000000000.0\n"
    );
}

#[test]
fn convert_errors() {
    assert_eq!(std_error("cast c.to_boolean(\"maybe\")").code, "E2011");
    assert_eq!(std_error("cast c.to_boolean([1])").code, "E3003");
    assert_eq!(std_error("cast c.to_list(5)").code, "E3003");
    assert_eq!(std_error("cast c.to_map([1, 2])").code, "E3003");
    assert_eq!(std_error("cast c.from_hex(\"xyz\")").code, "E2011");
    assert_eq!(std_error("cast c.from_binary(\"\")").code, "E2011");
    assert_eq!(std_error("cast c.to_binary(1.5)").code, "E3003");
    assert_eq!(std_error("cast c.format_number(1.5, 500)").code, "E2006");
}

// ---- error ----------------------------------------------------------------------------------------------

#[test]
fn error_module_writes_to_stderr() {
    let src = "import error\nspell main() {\n cast \"out\"\n error.warn(\"careful\")\n error.debug([1, \"a\"])\n}\n";
    let (out, err) = run_full(src, "", &[]);
    assert_eq!(out.unwrap(), "out\n");
    assert_eq!(err, "warning: careful\ndebug: [1, \"a\"] (list)\n");
}

#[test]
fn error_module_assertions() {
    assert_eq!(std_body("err.assert_equal([1, 2], [1, 2])\nerr.assert_true(1 < 2)\ncast \"ok\""), "ok\n");
    assert_eq!(
        std_body("try { err.assert_equal(1, 2) } catch e { cast e[\"kind\"] + \": \" + e[\"message\"] }\ntry { err.assert_true(Off, \"nope\") } catch e { cast e[\"message\"] }\ntry { err.fail(\"x\") } catch e { cast e[\"code\"] }"),
        "AssertionError: Assertion failed: 1 is not equal to 2\nnope\nE2100\n"
    );
    assert_eq!(std_error("err.assert_equal(\"a\", \"b\")").code, "E2101");
}

// ---- system --------------------------------------------------------------------------------------------

#[test]
fn system_basics() {
    assert_eq!(std_body("cast system.version()\ncast type_of(system.platform())\ncast type_of(system.clock())\ncast system.clock() >= 0.0\ncast system.args()\ncast system.env(\"ARCANA_SURELY_UNSET_VARIABLE\")\ncast type_of(system.cwd())"), "1.0.0\ntext\ndecimal\nOn\n[]\nnull\ntext\n");
    let src = "import system\nspell main() {\n cast system.args()\n cast length(system.args())\n system.write(\"no newline\")\n system.write(1)\n}\n";
    let (out, _) = run_full(src, "", &["alpha", "--beta", "3"]);
    assert_eq!(out.unwrap(), "[\"alpha\", \"--beta\", \"3\"]\n3\nno newline1");
}

#[test]
fn system_exit_stops_the_program_quietly() {
    let src = "import system\nspell main() {\n cast \"before\"\n try {\n  system.exit(3)\n } catch e {\n  cast \"never\"\n } finally {\n  cast \"cleanup\"\n }\n cast \"after\"\n}\n";
    let (r, _) = run_full(src, "", &[]);
    let e = r.unwrap_err();
    assert_eq!(e.exit_code, Some(3));
    let (r, _) = run_full("import system\nspell main() { system.exit() }", "", &[]);
    assert_eq!(r.unwrap_err().exit_code, Some(0));
    assert_eq!(std_error("system.exit(256)").code, "E2006");
}

// ---- file ------------------------------------------------------------------------------------------------

fn tmp(name: &str) -> String {
    let d = std::env::temp_dir().join(format!("arcana-std-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.to_string_lossy().replace('\\', "/")
}

#[test]
fn file_round_trip() {
    let d = tmp("file");
    let body = format!(
        "bind dir = \"{d}\"\nbind p = dir + \"/notes.txt\"\ncast file.exists(p)\nfile.write_text(p, \"one\\ntwo\\n\")\nfile.append_text(p, \"three\\n\")\ncast file.read_text(p)\ncast file.read_lines(p)\ncast file.size(p)\ncast file.is_file(p)\ncast file.is_dir(dir)\nfile.write_lines(dir + \"/b.txt\", [\"x\", \"y\"])\ncast file.read_text(dir + \"/b.txt\")\nfile.make_dir(dir + \"/sub/deep\")\ncast file.list_dir(dir)\nfile.remove(p)\ncast file.exists(p)\nfile.remove(dir + \"/sub/deep\")\ncast file.is_dir(dir + \"/sub/deep\")"
    );
    assert_eq!(
        std_body(&body),
        "Off\none\ntwo\nthree\n\n[\"one\", \"two\", \"three\"]\n14\nOn\nOn\nx\ny\n\n[\"b.txt\", \"notes.txt\", \"sub\"]\nOff\nOff\n"
    );
}

#[test]
fn file_errors_are_catchable() {
    let d = tmp("fileerr");
    let body = format!("try {{ cast file.read_text(\"{d}/missing.txt\") }} catch e {{ cast e[\"code\"] + \" \" + e[\"kind\"] }}\ntry {{ file.list_dir(\"{d}/nope\") }} catch e {{ cast e[\"code\"] }}");
    assert_eq!(std_body(&body), "E2012 FileError\nE2012\n");
    let e = std_error(&format!("cast file.read_text(\"{d}/missing.txt\")"));
    assert_eq!(e.code, "E2012");
    assert!(e.message.contains("No such file or directory"), "{}", e.message);
    assert_eq!(std_error("file.write_lines(\"x\", [1])").code, "E3003");
}

// ---- time ------------------------------------------------------------------------------------------------

#[test]
fn time_functions() {
    assert_eq!(
        std_body("cast type_of(time.now())\ncast time.now() > 1.7e9\nbind t0 = system.clock()\ntime.sleep(0.02)\ncast system.clock() - t0 >= 0.02\ncast time.format_duration(3725)\ncast time.format_duration(90061)\ncast time.format_duration(0.25)\ncast time.format_duration(60)\ncast time.format_duration(0)"),
        "decimal\nOn\nOn\n1h 2m 5s\n1d 1h 1m 1s\n250ms\n1m\n0ms\n"
    );
    assert_eq!(std_error("time.sleep(0 - 1)").code, "E2006");
    assert_eq!(std_error("cast time.format_duration(0 - 1)").code, "E2006");
}

// ---- date --------------------------------------------------------------------------------------------------

#[test]
fn date_construction_and_fields() {
    assert_eq!(
        std_body("bind d = date.make_date(2026, 10, 27)\ncast d\ncast d[\"year\"]\ncast date.make_datetime(2026, 10, 27, 14, 30)\ncast date.iso(d)\ncast date.iso(date.make_datetime(2026, 1, 2, 3, 4, 5))\ncast d == date.make_date(2026, 10, 27)"),
        "{\"year\": 2026, \"month\": 10, \"day\": 27}\n2026\n{\"year\": 2026, \"month\": 10, \"day\": 27, \"hour\": 14, \"minute\": 30, \"second\": 0}\n2026-10-27\n2026-01-02T03:04:05\nOn\n"
    );
    assert_eq!(
        std_body("cast date.iso(date.parse(\"2026-10-27\"))\ncast date.iso(date.parse(\"2026-10-27 14:30\"))\ncast date.iso(date.parse(\"2026-10-27T14:30:05Z\"))\ncast date.iso(date.parse(\"2026-1-5\"))"),
        "2026-10-27\n2026-10-27T14:30:00\n2026-10-27T14:30:05\n2026-01-05\n"
    );
}

#[test]
fn date_calendar_facts() {
    assert_eq!(
        std_body("bind d = date.make_date(2026, 10, 27)\ncast date.weekday(d)\ncast date.weekday_name(d)\ncast date.weekday_name(1)\ncast date.month_name(d)\ncast date.month_name(3)\ncast date.day_of_year(d)\ncast date.day_of_year(date.make_date(2024, 12, 31))\ncast date.is_leap_year(2000)\ncast date.is_leap_year(2100)\ncast date.days_in_month(2024, 2)\ncast date.days_in_month(2023, 2)\ncast date.weekday(date.from_timestamp(0))"),
        "2\nTuesday\nMonday\nOctober\nMarch\n300\n366\nOn\nOff\n29\n28\n4\n"
    );
}

#[test]
fn date_formatting() {
    assert_eq!(
        std_body("bind d = date.make_datetime(2026, 3, 5, 14, 7, 9)\ncast date.format(d, \"%Y-%m-%d %H:%M:%S\")\ncast date.format(d, \"%A, %d %B %Y\")\ncast date.format(d, \"%a %b %e %y\")\ncast date.format(d, \"%I:%M %p\")\ncast date.format(d, \"day %j, weekday %u, 100%%\")\ncast date.format(date.make_datetime(2026, 1, 1, 0, 5), \"%I:%M %p\")"),
        "2026-03-05 14:07:09\nThursday, 05 March 2026\nThu Mar  5 26\n02:07 PM\nday 064, weekday 4, 100%\n12:05 AM\n"
    );
    assert_eq!(std_error("cast date.format(date.make_date(2026, 1, 1), \"%Q\")").code, "E2006");
}

#[test]
fn date_arithmetic() {
    assert_eq!(
        std_body("bind d = date.make_date(2026, 1, 31)\ncast date.iso(date.add_days(d, 30))\ncast date.iso(date.add_days(d, 0 - 31))\ncast date.iso(date.add_months(d, 1))\ncast date.iso(date.add_months(d, 13))\ncast date.iso(date.add_months(d, 0 - 2))\ncast date.iso(date.add_years(date.make_date(2024, 2, 29), 1))\ncast date.iso(date.add_hours(d, 25))\ncast date.iso(date.add_minutes(date.make_datetime(2026, 1, 1, 23, 50), 15))\ncast date.iso(date.add_seconds(d, 1))"),
        "2026-03-02\n2025-12-31\n2026-02-28\n2027-02-28\n2025-11-30\n2025-02-28\n2026-02-01T01:00:00\n2026-01-02T00:05:00\n2026-01-31T00:00:01\n"
    );
    assert_eq!(
        std_body("bind a = date.make_date(2026, 12, 25)\nbind b = date.make_date(2026, 10, 27)\ncast date.diff_days(a, b)\ncast date.diff_days(b, a)\ncast date.diff_seconds(date.make_datetime(2026, 1, 1, 1), date.make_date(2026, 1, 1))\ncast date.compare(b, a)\ncast date.compare(a, b)\ncast date.compare(a, a)\ncast date.compare(date.make_date(2026, 1, 1), date.make_datetime(2026, 1, 1, 0, 0, 1))"),
        "59\n-59\n3600\n-1\n1\n0\n-1\n"
    );
}

#[test]
fn date_timestamps() {
    assert_eq!(
        std_body("cast date.iso(date.from_timestamp(0))\ncast date.to_timestamp(date.make_datetime(2026, 10, 27, 12, 0, 0))\ncast date.iso(date.from_timestamp(1793102400))\ncast date.iso(date.from_timestamp(1.9))\ncast date.iso(date.from_timestamp(0, 3600))\ncast date.to_timestamp(date.make_date(1969, 12, 31))\ncast date.iso(date.from_timestamp(0 - 1))"),
        "1970-01-01T00:00:00\n1793102400\n2026-10-27T12:00:00\n1970-01-01T00:00:01\n1970-01-01T01:00:00\n-86400\n1969-12-31T23:59:59\n"
    );
    let now = std_body("bind ts = time.now()\nbind d = date.from_timestamp(ts)\ncast date.compare(date.utc_now(), d) >= 0\ncast d[\"year\"] >= 2025\ncast type_of(date.local_offset())\ncast date.today()[\"year\"] >= 2025\ncast date.now()[\"hour\"] >= 0");
    assert_eq!(now, "On\nOn\ninteger\nOn\nOn\n");
}

#[test]
fn date_errors() {
    assert_eq!(std_error("date.make_date(2026, 2, 30)").code, "E2006");
    assert_eq!(std_error("date.make_date(2026, 13, 1)").code, "E2006");
    assert_eq!(std_error("date.make_date(0, 1, 1)").code, "E2006");
    assert_eq!(std_error("date.make_datetime(2026, 1, 1, 24)").code, "E2006");
    assert_eq!(std_error("date.parse(\"yesterday\")").code, "E2011");
    assert_eq!(std_error("date.parse(\"2026-02-30\")").code, "E2006");
    assert_eq!(std_error("date.weekday(5)").code, "E3003");
    assert_eq!(std_error("date.weekday({\"year\": 2026})").code, "E3003");
    assert_eq!(std_error("date.weekday({\"year\": 2026, \"month\": \"x\", \"day\": 1})").code, "E3003");
    assert_eq!(std_error("date.add_years(date.make_date(9999, 1, 1), 1)").code, "E2008");
    assert_eq!(std_error("date.add_days(date.make_date(1, 1, 1), 0 - 1)").code, "E2008");
    assert_eq!(std_error("date.weekday_name(8)").code, "E2006");
    let caught = std_body("try { date.make_date(2026, 2, 30) } catch e { cast e[\"kind\"] + \": \" + e[\"message\"] }");
    assert_eq!(caught, "ValueError: Invalid date: day 30 must be between 1 and 28 for 2026-02\n");
}

// ---- standard modules versus files ------------------------------------------------------------------------

#[test]
fn standard_module_wins_over_a_file_of_the_same_name() {
    let d = tmp("shadow");
    std::fs::write(format!("{}/math.arc", d), "spell sqrt(x) { return \"mine\" }\n").unwrap();
    let main = format!("{}/main.arc", d);
    std::fs::write(&main, "import math\nspell main() { cast math.sqrt(4) }\n").unwrap();
    let o = std::process::Command::new(env!("CARGO_BIN_EXE_arcana")).args(["run", &main]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "2.0\n");
}

// ---- CLI: program arguments and exit codes ----------------------------------------------------------------

fn arcana(args: &[&str]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_arcana")).args(args).output().unwrap()
}

#[test]
fn cli_passes_arguments_to_the_program() {
    let d = tmp("args");
    let f = format!("{}/a.arc", d);
    std::fs::write(&f, "import system\nspell main() {\n cast system.args()\n}\n").unwrap();
    let out = |args: &[&str]| String::from_utf8_lossy(&arcana(args).stdout).into_owned();
    assert_eq!(out(&["run", &f]), "[]\n");
    assert_eq!(out(&["run", &f, "one", "two"]), "[\"one\", \"two\"]\n");
    assert_eq!(out(&["run", &f, "--", "--max-steps", "5"]), "[\"--max-steps\", \"5\"]\n");
    assert_eq!(out(&["run", &f, "x", "--unknown-flag"]), "[\"x\", \"--unknown-flag\"]\n");
    assert_eq!(out(&[&f, "short"]), "[\"short\"]\n");
    // arcana's own options still work after the file name
    let o = arcana(&["run", &f, "--max-steps", "1"]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn cli_exit_code_from_the_program() {
    let d = tmp("exit");
    let f = format!("{}/e.arc", d);
    std::fs::write(&f, "import system\nspell main() {\n cast \"bye\"\n system.exit(7)\n}\n").unwrap();
    let o = arcana(&["run", &f]);
    assert_eq!(o.status.code(), Some(7));
    assert_eq!(String::from_utf8_lossy(&o.stdout), "bye\n");
    assert_eq!(String::from_utf8_lossy(&o.stderr), "");
}

#[test]
fn cli_stdin_and_stderr_streams() {
    let d = tmp("streams");
    let f = format!("{}/s.arc", d);
    std::fs::write(&f, "import error\nspell main() {\n bind n = ask(\"name? \")\n cast \"hi \" + n\n error.warn(\"done\")\n}\n").unwrap();
    use std::io::Write;
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_arcana"))
        .args(["run", &f])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"Mira\n").unwrap();
    let o = child.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&o.stdout), "name? hi Mira\n");
    assert_eq!(String::from_utf8_lossy(&o.stderr), "warning: done\n");
}
