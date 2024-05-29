pub trait Transpose: Iterator {
    fn transpose<T>(mut self) -> Vec<Vec<T>> // TODO in future make this all iterator-y
    where
        Self: Iterator<Item = Vec<T>> + Sized,
        T: core::fmt::Debug,
    {
        match self.next() {
            Some(vec) => {
                let mut out_vec: Vec<Vec<T>> = (0..vec.len()).map(|_| vec![]).collect();
                let this_iter = std::iter::once(vec).chain(self);
                
                for vec in this_iter {
                    for (vec, val) in out_vec.iter_mut().zip(vec) {
                        vec.push(val);
                    }
                }

                out_vec
                
            }
            None => Vec::new()
        }
    }
}

impl<T> Transpose for T where T: Iterator + ?Sized { }
