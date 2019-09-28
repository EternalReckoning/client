use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let bootstrap = eternalreckoning_client::Bootstrap {
        args: args,
        default_config: Some("config/default.toml".to_string()),
        user_config: None,
    };

    if let Err(ref e) = eternalreckoning_client::run(bootstrap) {
        eprintln!("Application error: {}", e);
        eprintln!("Backtrace: {:?}", e.backtrace());

        process::exit(1);
    }
}