extern crate crypto;
extern crate getopts;

use std::process::exit;
use std::{env, fs, io};

use self::getopts::Options;

use holo_configure;

use crypto::{digest::Digest, sha2};

const DETAIL: &str = "

    Produces a JSON file containing the seed entropy to produce the
HoloPort's Holochain and ZeroTier public/private keys.  This file is
*unencrypted*, as it is required to be read on start-up of the
HoloPort!

";

fn print_usage(program: &str, opts: Options) {
    let brief = opts.short_usage(program);
    eprintln!("{}\n{}", opts.usage(&brief), &DETAIL);
}

fn fail(message: &str, program: &str, opts: Options) -> ! {
    eprintln!("{}\n", message);
    print_usage(program, opts);
    exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("", "email", "User's email address", "EMAIL");
    opts.optopt(
        "",
        "password",
        "Password to authorize HoloPort configurations",
        "PASSWORD",
    );
    opts.optopt(
        "",
        "from",
        "Generate seed from entropy in the provided file",
        "FILE",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            fail(&f.to_string(), &program, opts);
        }
    };

    // Collect the email address; must not be empty.
    let email = matches.opt_str("email").unwrap_or_else(|| {
        let mut input = String::new();
        while {
            eprint!("email:    ");
            match io::stdin().read_line(&mut input) {
                Err(e) => {
                    eprintln!("{:?}", e);
                    false
                }
                Ok(n) => n == 0,
            }
        } {}
        input.trim().to_owned()
    });
    if email.len() == 0 {
        fail("Failed to read non-empty email address", &program, opts);
    }

    // Collect the password from args or command-line; empty password is allowed.
    let password = matches.opt_str("password").unwrap_or_else(|| {
        let mut input = String::new();
        while {
            eprint!("password: ");
            match io::stdin().read_line(&mut input) {
                Err(e) => {
                    eprintln!("{:?}", e);
                    false
                }
                Ok(n) => n == 0,
            }
        } {}
        input.trim().to_owned()
    });
    if password.len() == 0 {
        eprintln!("WARNING: Proceeding with empty password");
    }

    // Collect seed optionally from entropy in file
    let seed_maybe = match matches.opt_str("from") {
        None => None,
        Some(filename) => {
            let entropy = match fs::read_to_string(&filename) {
                Ok(string) => string,
                Err(e) => fail(
                    &format!("Failed to read entropy from {:?}: {}", &filename, e),
                    &program,
                    opts,
                ),
            };
            let mut seed = [0u8; 32];
            let mut hasher = sha2::Sha256::new();
            hasher.input_str(&entropy);
            hasher.result(&mut seed);
            Some(seed)
        }
    };

    // Using the email address as salt, extend the password into a seed for a
    // public/private signing keypair, used to authenticate configuration
    // requests to the HoloPort. Only a holder of of the same email and password
    // (and optional name) can generate the corresponding private key, and sign
    // a request. If optional seed entropy is not provided, a random seed will
    // be computed.
    eprintln!("Generating HoloPort Configuration for email: {}", &email);
    match holo_configure::Config::new(email, password, seed_maybe) {
        Ok(c) => println!("{}", serde_json::to_string_pretty(&c).unwrap()),
        Err(e) => fail(
            &format!("Failed to generate HoloPort configuration: {}", e),
            &program,
            opts,
        ),
    }
}
