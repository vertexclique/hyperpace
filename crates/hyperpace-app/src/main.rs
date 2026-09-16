//! Process entry point for the Hyperpace desktop application; see `hyperpace_app::run` for
//! everything the app itself does.

fn main() {
    if let Err(error) = hyperpace_app::run() {
        eprintln!("hyperpace: {error}");
        std::process::exit(1);
    }
}
