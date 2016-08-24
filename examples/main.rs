use std::env;
use std::io::{BufRead, BufReader};
use std::fs::File;

extern crate multi_reader;

fn count_lines<R: BufRead>(reader: R) -> usize {
    reader.lines().count()
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let files = args[2..].iter().map(|f| File::open(f).expect("File not found"));

    let mut count = 0;
    if args[1] == "chained" {
        let reader = multi_reader::MultiReader::new(files);
        count = count_lines(BufReader::new(reader));
    } else {
        for file in files {
            count += count_lines(BufReader::new(file));
        }
    }

    println!("Lines count: {}", count);
}
