fn main() {
    if let Err(e) = canopy_cli::run_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
