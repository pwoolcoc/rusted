extern crate _rusted as rusted;
use rusted::Config;

fn main() {
    let mut config = Config::default();
    if let Err(_) = rusted::run(&mut config) {
        ::std::process::exit(1);
    }
}

