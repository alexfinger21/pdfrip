use std::sync::Arc;

pub trait Producer {
    fn next(&mut self) -> Option<Arc<Vec<u8>>>;
    /// Used for measuring progress. Reflects the number of passwords this producer can produce
    fn size(&self) -> usize;
}

pub mod dictionary {

    use std::{
        fs,
        io::{BufRead, BufReader},
        sync::Arc,
    };

    use super::Producer;

    pub struct LineProducer {
        inner: Box<dyn BufRead>,
        size: usize,
    }

    impl LineProducer {
        pub fn from(path: &str) -> Self {
            // TODO: This will be slow on large files, so we might want to skip this 
            // depending on the filesize.
            let lines = fs::read(&path)
                .unwrap()
                .iter()
                .filter(|x| {
                    if let Some(y) = char::from_u32(**x as u32) {
                        y == '\n'
                    } else {
                        false
                    }
                })
                .count();
            let file = fs::File::open(path).unwrap();
            let reader = BufReader::new(file);

            Self {
                inner: Box::from(reader),
                size: lines,
            }
        }
    }

    impl Producer for LineProducer {
        fn next(&mut self) -> Option<std::sync::Arc<Vec<u8>>> {
            let mut buffer = String::new();
            match self.inner.read_line(&mut buffer) {
                Ok(_) => Some(Arc::from(buffer.into_bytes())),
                Err(err) => {
                    debug!("Unable to read from reader: {}", err);
                    None
                }
            }
        }

        fn size(&self) -> usize {
            self.size
        }
    }
}

pub mod number_ranges {
    use std::sync::Arc;

    use super::Producer;

    pub struct RangeProducer {
        padding_len: usize,
        inner: Box<dyn Iterator<Item = usize>>,
        size: usize,
    }

    impl RangeProducer {
        pub fn new(padding_len: usize, lower_bound: usize, upper_bound: usize) -> Self {
            Self {
                padding_len,
                inner: Box::from(lower_bound..upper_bound),
                size: upper_bound - lower_bound,
            }
        }
    }

    impl Producer for RangeProducer {
        fn next(&mut self) -> Option<Arc<Vec<u8>>> {
            let next = self.inner.next();
            match next {
                Some(number) => {
                    let data =
                        format!("{:0>width$}", number, width = self.padding_len).into_bytes();
                    Some(Arc::new(data))
                }
                None => None,
            }
        }

        fn size(&self) -> usize {
            self.size
        }
    }
}
