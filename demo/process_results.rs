/// Trait for processing results of an iterator.
/// From Chaim freedman's answer to https://stackoverflow.com/questions/69746026/how-to-convert-an-iterator-of-results-into-a-result-of-an-iterator,
/// combined with an example from the `itertools` crate at https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.process_results.
///
//# Purpose: RYO iterator result processor
//# Categories: learning, technique
pub trait IteratorExt: Iterator {
    fn process_results<R, T, E>(
        self,
        f: impl FnOnce(ProcessResults<'_, Self, E>) -> R,
    ) -> Result<R, E>
    where
        Self: Iterator<Item = Result<T, E>>;
}

impl<I: Iterator> IteratorExt for I {
    fn process_results<R, T, E>(
        self,
        f: impl FnOnce(ProcessResults<'_, Self, E>) -> R,
    ) -> Result<R, E>
    where
        Self: Iterator<Item = Result<T, E>>,
    {
        let mut err = None;
        let p = ProcessResults {
            err: &mut err,
            iter: self,
        };
        let success = f(p);
        err.map(Err).unwrap_or(Ok(success))
    }
}

pub struct ProcessResults<'a, I, E> {
    err: &'a mut Option<E>,
    iter: I,
}

impl<T, E, I: Iterator<Item = Result<T, E>>> Iterator for ProcessResults<'_, I, E> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.iter.next() {
            Some(Ok(item)) => Some(item),
            Some(Err(err)) => {
                *self.err = Some(err);
                None
            }
            None => None,
        }
    }
}

type Item = Result<i32, &'static str>;

fn main() {
    let first_values: Vec<Item> = vec![Ok(1), Ok(0), Ok(3)];
    let second_values: Vec<Item> = vec![Ok(2), Ok(1), Err("overflow")];

    // “Lift” the iterator .max() method to work on the Ok-values.
    let first_max = first_values
        .into_iter()
        .process_results(|iter| iter.max().unwrap_or(0));
    let second_max = second_values
        .into_iter()
        .process_results(|iter| iter.max().unwrap_or(0));

    println!("First max: {:?}", first_max);
    println!("Second max: {:?}", second_max);
    assert_eq!(first_max, Ok(3));
    assert!(second_max.is_err());
}
