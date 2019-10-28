use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let bootstrap = eternalreckoning_lobby::Bootstrap {
        args: args,
        config: Some("config/lobby.toml".to_string()),
    };

    if let Err(ref e) = eternalreckoning_lobby::run(bootstrap) {
        log::error!("Application error: {}", e);

        eprintln!("Application error: {}", e);
        eprintln!("Backtrace: {:?}", e.backtrace());

        process::exit(1);
    }
}