// This file and the `bin` module exist only to get the RLS working on the binaries
// Unfortunately this means that a simple `cargo build`/`cargo run` fails because there are
// multiple main functions
mod bin;
mod lib;

fn main() {
    println!("Nothing to see here, execute individual tutorials with `cargo run --bin <tutorial>`");
}
