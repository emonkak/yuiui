extern crate keytray;

use std::env;

use keytray::app;
use keytray::config::Config;
use keytray::context::Context;

fn main() {
    let args = env::args().collect();
    let config = Config::parse(args);
    let context = Context::new(config).unwrap();

    app::run(context);
}
