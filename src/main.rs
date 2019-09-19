extern crate getopts;

use std::env;
use std::process::exit;

use getopts::Options;

const DETAIL: &str = "Produces:

    Produces a JSON file containing the seed entropy to produce the
HoloPort's Holochain and ZeroTier public/private keys.  This file is *unencrypted*,
as it is required to be read on start-up of the HoloPort!

    Treat it as precious.";

fn print_usage(program: &str, opts: Options) {
    let brief = opts.short_usage(program);
    println!("{}\n{}", opts.usage(&brief), DETAIL);
}

fn fail(message: &str, program: &str, opts: Options) -> ! {
    println!("{}\n", message);
    print_usage(program, opts);
    exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("", "email", "User's email address", "EMAIL");
    opts.reqopt("", "password", "Password to authorize HoloPort configurations", "PASSWORD");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            fail(&f.to_string(), &program, opts);
        }
    };

    let email = matches.opt_str("email").unwrap();
    let password = matches.opt_str("password").unwrap();

    println!("email: {}, password: {}", email, password);
}
