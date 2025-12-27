use std::process;

fn main() {
    if let Err(e) = vimurai::run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
