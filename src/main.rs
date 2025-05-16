// rsql.rs - Nathanael "NateNateNate" Thevarajah
// <natenatenat3@protonmail.com> - Refer to the license for more
// information.

mod client;
mod engine;

use client::connect;

use clap::Parser;

fn main() {
    let args = connect::Args::parse();
    let _ = args.connect();
}
