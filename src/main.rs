use std::env;
use std::process;

use minigrep::Config;

fn main() {
    // now passing the arg iterator directly to the config build
     let config = Config::build(env::args()).unwrap_or_else(|err|{
        println!("{err}");
        process::exit(1);
    });

    if let Err(e) = minigrep::run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}


