// Similar to the qfs.elf program, but with NeFS/Neutron specific functionalities
// Callable within the kernel. QFS functionalities are callable within arcboot

use clap::Parser;
use std::fs::File;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long)]
    name: String,

    /// Number of times to greet
    #[clap(short, long, default_value_t = 1)]
    count: u8,
}

/// The nefs.elf program
fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name)
    }

    let filepath = args.name.clone();

    let mut f = File::open(filepath).expect("Couldnt open file");

    // parse the header
}

fn get_help_message() -> String {
    "
    cli
        -r <file> \t[read a file]
        -b        \t[build a struct]
        -o <file> \t[output an in memory struct to a file]
    "
    .to_string()
}
