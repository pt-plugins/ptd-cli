mod shared;
mod host;

fn main() {
    if atty::is(atty::Stream::Stdin) {
        eprintln!("This process is meant to be launched by a browser via Native Messaging.");
        eprintln!("Run 'ptd status' to inspect available instances.");
        std::process::exit(1);
    }
    println!("ptd-host daemon — not yet implemented");
}
