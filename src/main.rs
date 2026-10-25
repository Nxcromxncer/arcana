fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(arcana::cli::main_with_args(&args));
}
