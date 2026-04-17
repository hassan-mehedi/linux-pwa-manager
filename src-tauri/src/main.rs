fn main() {
    if let Err(error) = webapp_manager::bootstrap() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
