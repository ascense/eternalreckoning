use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let bootstrap = eternalreckoning_subscription::Bootstrap {
        args: args,
        config: Some("config/subscription.toml".to_string()),
    };

    if let Err(ref e) = eternalreckoning_subscription::run(bootstrap) {
        log::error!("Application error: {}", e);

        eprintln!("Application error: {}", e);
        eprintln!("Backtrace: {:?}", e.backtrace());

        process::exit(1);
    }
}