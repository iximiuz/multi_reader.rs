# MultiReader - a composite reader implementation.

Like `std::io::Chain` but allows to chain more than two readers together.

# Usage

    extern crate multi_reader;
    use std::env;
    use std::io::{BufRead, BufReader};
    use std::fs::File;
    
    fn main() {
        let args: Vec<_> = env::args().collect();
        let files = args[1..].iter().map(|f| File::open(f).expect("File not found"));
        let reader = BufReader::new(multi_reader::MultiReader::new(files));
        println!("Total lines count: {}", reader.lines().count());
    }
    
# Examples

Run `cargo run --example main chained /path/to/file/a /path/to/file/b ...`.

# Tests

    cargo test
