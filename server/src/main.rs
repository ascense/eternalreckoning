use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let bootstrap = eternalreckoning_server::Bootstrap {
        args: args,
        config: Some(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/config/default.toml"
            ).to_string()
        ),
    };

    if let Err(ref e) = eternalreckoning_server::run(bootstrap) {
        eprintln!("Application error: {}", e);
        eprintln!("Backtrace: {:?}", e.backtrace());

        process::exit(1);
    }
}