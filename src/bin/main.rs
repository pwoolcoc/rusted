extern crate env_logger;
extern crate _rusted as rusted;
use rusted::Config;

fn main() {
    env_logger::init().unwrap();
    let mut config = Config::default();
    if let Err(_) = rusted::run(&mut config) {
        ::std::process::exit(1);
    }
}

