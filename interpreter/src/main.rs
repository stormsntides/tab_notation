use std::{env, process};

use interpreter::Config;

fn main() {
    // set up a configuration based on the arguments passed in
    // or fail if no arguments were passed in
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Could not parse arguments: {}", err);
        process::exit(1);
    });

    // execute the file reading operation
    // or fail if the file cannot be read
    if let Err(e) = interpreter::run(config) {
        eprintln!("Interpreter failed:\n{}", e);
        process::exit(2);
    };
}