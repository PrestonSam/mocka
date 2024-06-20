use std::{collections::HashMap, ops::ControlFlow};


pub trait Transpose: Iterator {
    fn transpose<T>(mut self) -> Vec<Vec<T>> // TODO in future make this all iterator-y
    where
        Self: Iterator<Item = Vec<T>> + Sized,
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

