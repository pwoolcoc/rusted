extern crate _rusted as rusted;

fn main() {
    let config = rusted::Config {
        prompt: "".into(),
    };
    if let Err(_) = rusted::run(&config) {
        ::std::process::exit(1);
    }
}

