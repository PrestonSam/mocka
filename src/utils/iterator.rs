use std::{collections::HashMap, marker::PhantomData, ops::ControlFlow};

use itertools::Itertools;
use thiserror::Error;

#[derive(Debug)]
pub enum LegacyTransposeError {
    InconsistentVecLengths { expected_length: usize, found_length: usize },
}

#[derive(Debug)]
pub struct LegacyTransposed<T> {
    src: Vec<std::vec::IntoIter<T>>,
}

impl<T> Iterator for LegacyTransposed<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.src.iter_mut()
            .map(|vec| vec.next())
            .collect::<Option<Vec<_>>>()
    }
}

pub trait LegacyTranspose: Iterator {
    fn transpose<T>(mut self) -> Result<LegacyTransposed<T>, LegacyTransposeError>
    where
        Self: Iterator<Item = Vec<T>> + Sized,
    {
        let mut compare_length = Option::None::<usize>;

        self.try_fold(vec![], |mut src, vec| {
            let vec_len = vec.len();
            let cmp_len = compare_length.get_or_insert(vec_len);

            if *cmp_len == vec_len {
                src.push(vec.into_iter());

                Ok(src)
            } else {
                Err(LegacyTransposeError::InconsistentVecLengths { expected_length: *cmp_len, found_length: vec_len })
            }
        }).map(|src| LegacyTransposed { src })
    }
}

impl<T> LegacyTranspose for T where T: Iterator + ?Sized { }




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