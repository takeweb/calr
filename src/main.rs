fn main() {
    if let Err(e) = calr::run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
