use std::{collections::HashMap, ops::ControlFlow};

#[derive(Debug)]
pub enum TransposeError {
    InconsistentVecLengths { expected_length: usize, found_length: usize },
}

#[derive(Debug)]
pub struct Transposed<T> {
    src: Vec<std::vec::IntoIter<T>>,
}

impl<T> Iterator for Transposed<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.src.iter_mut()
            .map(|vec| vec.next())
            .collect::<Option<Vec<_>>>()
    }
}

pub trait Transpose: Iterator {
    fn transpose<T>(self) -> Result<Transposed<T>, TransposeError>
    where
        Self: Iterator<Item = Vec<T>> + Sized,
    {
        let mut compare_length = Option::None::<usize>;

        self.fold(Ok(vec![]), |rslt, vec| {
            rslt.and_then(|mut src| {
                let vec_len = vec.len();
                let cmp_len = compare_length.get_or_insert(vec_len);

                if *cmp_len == vec_len {
                    src.push(vec.into_iter());

                    Ok(src)
                } else {
                    Err(TransposeError::InconsistentVecLengths { expected_length: *cmp_len, found_length: vec_len })
                }
            })
        }).map(|src| Transposed { src })
    }
}

impl<T> Transpose for T where T: Iterator + ?Sized { }


pub trait Group {
    fn group<K, V>(self) -> HashMap<K, Vec<V>>
    where
        Self: Iterator<Item = (K, V)> + Sized,
        K: Eq + std::hash::Hash,
    {
        self.fold(HashMap::new(), |mut map, (k, v)| {
            map.entry(k).or_insert_with(|| Vec::new()).push(v);
            map
        })
    }
}

impl<T> Group for T where T: Iterator + ?Sized { }



pub trait MapIfSame: Iterator {
    fn map_if_same<K, V, Kf, Tf>(mut self, key_fn: Kf, mut transformer: Tf) -> Option<Vec<V>>
    where
        Self: Sized,
        K: Eq,
        Kf: Fn(&Self::Item) -> K,
        Tf: FnMut(&K, Self::Item) -> V,
    {
        match self.next() {
            Some(val) => {
                let key = key_fn(&val);
                let first_transformed = transformer(&key, val);

                let output = self.try_fold(vec![ first_transformed ], |mut vec, val| {
                    if key == key_fn(&val) {
                        vec.push(transformer(&key, val));
                        ControlFlow::Continue(vec)
                    } else {
                        ControlFlow::Break(())
                    }
                });

                match output {
                    ControlFlow::Continue(vec) => Some(vec),
                    ControlFlow::Break(_) => None,
                }
            }
            None => Some(vec![])
        }
    }
}

impl<T> MapIfSame for T where T: Iterator + ?Sized { }

