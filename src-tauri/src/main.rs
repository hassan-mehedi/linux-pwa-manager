fn main() {
    if let Err(error) = linux_pwa_manager::bootstrap() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
