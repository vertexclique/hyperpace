//! Process entry point for the Hyperpace desktop application; see `hyperpace_app::run` for
//! everything the app itself does.

fn main() {
    if let Err(error) = hyperpace_app::run() {
        // `run` installs the tracing subscriber as its first step, so it is already active by
        // the time it can return an error.
        tracing::error!(%error, "hyperpace failed to start");
        std::process::exit(1);
    }
}
