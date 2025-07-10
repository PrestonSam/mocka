use std::{collections::HashMap, marker::PhantomData, ops::ControlFlow};

use itertools::Itertools;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum TransposeError {
    #[error("jagged columns")]
    JaggedColumns(Vec<usize>),

    #[error("empty input collection")]
    EmptyInputCollection,
}

pub struct Transposed<I, C>
where
    I: ExactSizeIterator,
    C: FromIterator<I::Item>,
{
    container: Vec<I>,
    _phantom: PhantomData<C>,
}

impl<IC, C> Transposed<IC, C>
where
    IC: ExactSizeIterator,
    C: FromIterator<IC::Item>
{
    fn new(row_container: Vec<IC>) -> Result<Self, TransposeError>
    {

        if row_container.is_empty() {
            return Err(TransposeError::EmptyInputCollection);
        }

        if row_container.iter().map(|i| i.len()).all_equal() {
            Ok(Self {
                container: row_container,
                _phantom: std::marker::PhantomData
            })
        } else {
            let sizes = row_container.into_iter()
                .map(|i| i.len())
                .collect();

            Err(TransposeError::JaggedColumns(sizes))
        }
    }
}

pub trait Transpose<IR, IC>
where
    IR: IntoIterator<Item = IC>,
    IC: IntoIterator,
    IC::IntoIter: ExactSizeIterator,
{
    fn transpose<C>(self) -> Result<Transposed<IC::IntoIter, C>, TransposeError>
    where
        Self: Sized,
        C: FromIterator<IC::Item>;
}

impl<IR, IC> Transpose<IR, IC> for IR
where
    IR: IntoIterator<Item = IC>,
    IC: IntoIterator,
    IC::IntoIter: ExactSizeIterator,
{
    fn transpose<C>(self) -> Result<Transposed<IC::IntoIter, C>, TransposeError>
    where
        Self: Sized,
        C: FromIterator<<IC>::Item>
    {
        let row_container = self.into_iter()
            .map(IntoIterator::into_iter)
            .collect();

        Transposed::new(row_container)
    }
}

impl<I, C> Iterator for Transposed<I, C>
where
    Self: Sized,
    I: ExactSizeIterator,
    C: FromIterator<I::Item>,
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        self.container.iter_mut()
            .map(Iterator::next)
            .collect()
    }
}



pub trait FindOk where Self: Iterator + Sized {
    fn find_ok<E, F>(self, mut predicate: F) -> Result<Option<Self::Item>, E>
    where F: FnMut(&Self::Item) -> Result<bool, E>
    {
        for item in self {
            if predicate(&item)? {
                return Ok(Some(item))
            }
        }
        Ok(None)
    }
}

impl<I> FindOk for I where I: Iterator {}







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


mod tests {
    use crate::utils::iterator::Transpose;

    #[test]
    fn test_transpose() {
        let input = [
            vec![ 'Y', ' ', 'e', 't' ],
            vec![ 'o', 's', ' ', '!' ],
            vec![ 'u', 'e', 'i', '!' ],
        ];

        let mut transposed = input.transpose::<Vec<_>>().unwrap();

        assert_eq!(transposed.next(), Some(vec![ 'Y', 'o', 'u' ]));
        assert_eq!(transposed.next(), Some(vec![ ' ', 's', 'e' ]));
        assert_eq!(transposed.next(), Some(vec![ 'e', ' ', 'i' ]));
        assert_eq!(transposed.next(), Some(vec![ 't', '!', '!' ]));
    }
}
