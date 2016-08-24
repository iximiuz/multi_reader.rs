//! A composite reader implementation.
//!
//! Like `io::Chain` but allows to chain more than two readers together.
//!
//! # Use
//! ```rust
//! extern crate multi_reader;
//! use std::env;
//! use std::io::{BufRead, BufReader};
//! use std::fs::File;
//!
//! fn main() {
//!     let args: Vec<_> = env::args().collect();
//!     let files = args[1..].iter().map(|f| File::open(f).expect("File not found"));
//!     let reader = BufReader::new(multi_reader::MultiReader::new(files));
//!     println!("Total lines count: {}", reader.lines().count());
//! }
//! ```

#![crate_name = "multi_reader"]

use std::io;
use std::io::Read;

pub struct MultiReader<R, I> {
    readers: I,
    current: Option<R>,
}

impl<R: Read, I: Iterator<Item = R>> MultiReader<R, I> {
    pub fn new(mut readers: I) -> MultiReader<R, I> {
        let current = readers.next();
        MultiReader {
            readers: readers,
            current: current,
        }
    }
}

impl<R: Read, I: Iterator<Item = R>> Read for MultiReader<R, I> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.current {
                Some(ref mut r) => {
                    let n = try!(r.read(buf));
                    if n > 0 {
                        return Ok(n);
                    }
                }
                None => return Ok(0),
            }
            self.current = self.readers.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Read;
    use super::MultiReader;

    #[test]
    fn test_single_reader() {
        let source = io::repeat(0).take(10);
        let mut m = MultiReader::new(vec![source].into_iter());
        let mut buf = [0; 4];
        for amount in &[4, 4, 2, 0] {
            assert_eq!(m.read(&mut buf).unwrap(), *amount);
        }
    }

    #[test]
    fn test_several_readers() {
        let mut sources = Vec::new();
        for i in 0..20 {
            let s = io::repeat(i).take(3);
            sources.push(s);
        }
        let mut m = MultiReader::new(sources.into_iter());

        for i in 0..40 {
            let mut buf = [0; 2];
            let read = 2 - (i % 2);
            assert_eq!(m.read(&mut buf).unwrap(), read);
            for b in &buf[..read] {
                assert_eq!(*b, (i / 2) as u8);
            }
        }
        assert_eq!(m.read(&mut [0; 10]).unwrap(), 0);
    }

    #[test]
    fn test_first_readers_is_empty() {}

    #[test]
    fn test_middle_readers_is_empty() {}

    #[test]
    fn test_last_readers_is_empty() {}

    #[test]
    fn test_only_empty_readers() {}

    #[test]
    fn test_err_during_read() {
        struct MaybeErrReader<R> {
            reader: R,
            read_no: i32,
            fail_at: i32,
        }

        impl<R: Read> MaybeErrReader<R> {
            fn good(reader: R) -> MaybeErrReader<R> {
                MaybeErrReader {
                    reader: reader,
                    read_no: 0,
                    fail_at: -1,
                }
            }

            fn broken(reader: R, fail_at: i32) -> MaybeErrReader<R> {
                MaybeErrReader {
                    reader: reader,
                    read_no: 0,
                    fail_at: fail_at,
                }
            }
        }

        impl<R: Read> Read for MaybeErrReader<R> {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.read_no += 1;
                if self.read_no == self.fail_at + 1 {
                    Err(io::Error::new(io::ErrorKind::Other, "I'm broken"))
                } else {
                    self.reader.read(buf)
                }
            }
        }

        let s0 = MaybeErrReader::good(io::repeat(0).take(10));
        let s1 = MaybeErrReader::broken(io::repeat(0).take(2048), 1);
        let s2 = MaybeErrReader::good(io::repeat(0).take(10));
        let mut m = MultiReader::new(vec![s0, s1, s2].into_iter());

        assert_eq!(m.read(&mut [0; 1024]).unwrap(), 10);
        assert_eq!(m.read(&mut [0; 1024]).unwrap(), 1024);
        assert_eq!(m.read(&mut [0; 1024]).map_err(|e| e.to_string()).unwrap_err(),
                   "I'm broken".to_string());
    }
}
