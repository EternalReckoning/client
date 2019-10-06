use std::process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let bootstrap = eternalreckoning_client::Bootstrap {
        args: args,
        config: Some("config/client.toml".to_string()),
    };

    if let Err(ref e) = eternalreckoning_client::run(bootstrap) {
        log::error!("Application error: {}", e);

        eprintln!("Application error: {}", e);
        eprintln!("Backtrace: {:?}", e.backtrace());

        process::exit(1);
    }
}