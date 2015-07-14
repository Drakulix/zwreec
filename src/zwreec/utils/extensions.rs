//! Extensions to existing types used in the zwreec compiler
//!
//! Currently this contains only extensions to the Iterator trait yielding new Iterator types
//! that wrap certain functionality, that is used to filter/construct/lookahead all objects
//! passed and iterated upon in the compiler, making it possible to have most of the compiler
//! chain to be lazy_evaluated and even multi-threaded.

use std::iter::Peekable;
use std::sync::mpsc;
use std::thread;

/// An iterator that performs a lookahead of 1, utilizing the existing Peakable Iterator
/// Can be stacked to perform an even greater lookahead
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Peeking<I, A> where
    A: Clone,
    I: Iterator<Item=A>,
{
    iter: Peekable<I>,
}

impl<I, A> Iterator for Peeking<I, A> where
    A: Clone,
    I: Iterator<Item=A>
{
    type Item = (A, Option<A>);

    #[inline]
    fn next(&mut self) -> Option<(A, Option<A>)> {
        match self.iter.next() {
            Some(item) => Some((item,
                match self.iter.peek() {
                    Some(ref item) => Some((*item).clone()),
                    None => None,
                }
            )),
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An extension for sized Iterators that specifies a function that returns a Peeking Iterator
pub trait PeekingExt {

    /// Consumes this Iterator and returns a Peeking Iterator
    fn peeking(self) -> Peeking<Self, Self::Item>
        where Self: Sized+Iterator,
              Self::Item: Clone;
}

impl<A: Clone, I: Sized+Iterator<Item=A>> PeekingExt for I {
    fn peeking(self) -> Peeking<Self, A>
        where Self: Sized+Iterator
    {
         Peeking { iter: self.peekable() }
    }
}

/// An iterator to maintain state while iterating another iterator and allows filtering
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct FilteringScan<I, St, F> {
    iter: I,
    f: F,
    state: St,
}

impl<B, I, St, F> Iterator for FilteringScan<I, St, F> where
    I: Iterator,
    F: FnMut(&mut St, I::Item) -> Option<B>,
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        loop {
            match self.iter.next() {
                Some(next) => match (self.f)(&mut self.state, next) {
                    Some(result) => return Some(result),
                    None => continue,
                },
                None => return None,
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper) // can't know a lower bound, due to the scan function
    }
}

/// An extension to sized Iterators that specifies a function that returns a FilteringScan Iterator
pub trait FilteringScanExt {

    /// Consumes this Iterator and returns a FilteringScan Iterator
    fn scan_filter<St, B, F>(self, initial_state: St, f: F) -> FilteringScan<Self, St, F>
        where Self: Sized+Iterator, F: FnMut(&mut St, Self::Item) -> Option<B>;
}

impl<I: Sized+Iterator> FilteringScanExt for I {
    fn scan_filter<St, B, F>(self, initial_state: St, f: F) -> FilteringScan<Self, St, F>
        where Self: Sized+Iterator, F: FnMut(&mut St, I::Item) -> Option<B>,
    {
        FilteringScan{iter: self, f: f, state: initial_state}
    }
}

/// The result of a parsing operation
#[derive(Clone)]
pub enum ParseResult {
    /// Continue parsing
    Continue,

    /// Halt parsing because an error occured
    Halt,

    /// End parsing because the last Token was processed
    End,
}

/// An iterator to maintain state while iterating another iterator and allows more complex parsing
// (allowing holding the current iterator value, continue if its goes empty and filter the output)
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Parser<I, A, St, F> where
    A: Clone,
    I: Iterator<Item=A>,
{
    iter: I,
    f: F,
    state: St,

    iter_state: ParseResult,
    current_elem: Option<A>,
}

impl<B, I, St, F, A> Iterator for Parser<I, A, St, F> where
    A: Clone,
    I: Iterator<Item=A>,
    F: FnMut(&mut St, Option<I::Item>) -> (ParseResult, Option<B>),
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        loop {
            match self.iter_state {
                ParseResult::Continue => self.current_elem = self.iter.next(),
                ParseResult::Halt => {},
                ParseResult::End => return None,
            };

            let (new_state, result) = (self.f) (&mut self.state, self.current_elem.clone());

            self.iter_state = new_state;

            match result {
                Some(result) => return Some(result),
                None => continue,
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None) // can't know a lower or upper bound, due to the parse function
    }
}

/// An extension to sized Iterators that specifies a function that returns a Parser Iterator
pub trait ParserExt {

    /// Consumes this Iterator and returns a FilteringScan Iterator
    fn parsing<St, B, F>(self, initial_state: St, f: F) -> Parser<Self, Self::Item, St, F>
        where Self: Sized+Iterator, Self::Item: Clone, F: Fn(&mut St, Option<Self::Item>) -> (ParseResult, Option<B>);
}

impl<A:Clone, I: Sized+Iterator<Item=A>>ParserExt for I {
    fn parsing<St, B, F>(self, initial_state: St, f: F) -> Parser<Self, A, St, F>
        where F: FnMut(&mut St, Option<A>) -> (ParseResult, Option<B>),
    {
        Parser{iter: self, f: f, state: initial_state, iter_state: ParseResult::Continue, current_elem: None}
    }
}


/// An iterator to maintain state while iterating another iterator and allows to hold the currently active
/// object to modify it (/construct it), until the next element is created
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Constructor<I, St, B, F>
    where
    I: Iterator,
    B: Clone,
    F: FnMut(&mut St, &mut Option<B>, I::Item) -> Option<B> {
        iter: I,
        state: St,
        f: F,
        current_elem: Option<B>,
}

impl<I, St, B, F> Iterator for Constructor<I, St, B, F> where
    I: Iterator,
    B: Clone,
    F: FnMut(&mut St, &mut Option<B>, I::Item) -> Option<B>,
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<B> {
        while let Some(elem) =  self.iter.next() {
            match (self.f)(&mut self.state, &mut self.current_elem, elem) {
                Some(elem) => {
                    let return_val = self.current_elem.clone();
                    self.current_elem = Some(elem);
                    match return_val {
                        Some(value) => return Some(value),
                        None => continue,
                    }
                },
                None => continue,
            }
        }

        let result = self.current_elem.clone();
        self.current_elem = None;
        result
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper) // can't know a lower bound, due to the waiting function
    }
}

/// An extension to sized Iterators that specifies a function that returns a Constructor Iterator
pub trait ConstructorExt {

    /// Consumes this Iterator and returns a Constructor Iterator
    fn construct_state<St, B, F>(self, state: St, f: F) -> Constructor<Self, St, B, F>
        where Self: Sized+Iterator, B: Clone, F: Fn(&mut St, &mut Option<B>, Self::Item) -> Option<B>;
}

impl<I: Sized+Iterator>ConstructorExt for I {
    fn construct_state<St, B, F>(self, state: St, f: F) -> Constructor<Self, St, B, F>
        where Self: Sized+Iterator, B: Clone, F: Fn(&mut St, &mut Option<B>, I::Item) -> Option<B>,
    {
        Constructor{iter: self, state: state, f: f, current_elem: None}
    }
}

/// An Iterator that performs all iteration in a seperate therad and caches them
pub struct Cached<A: Send> {
    rx: mpsc::Receiver<A>
}

impl<A: Send> Iterator for Cached<A> {
    type Item = A;

    fn next(&mut self) -> Option<A> { self.rx.recv().ok() }
}

/// A function to create a cached Iterator by passing the old Iterator by closure.
/// If the underlying iterator does not support Send you can construct it in the closure
pub fn cached<A: Send + 'static, I: Sized + Iterator<Item=A>, F: FnOnce() -> I + Send + 'static>(constructor: F) -> (Cached<A>, thread::JoinHandle<()>)
{
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        for item in constructor() {
            let _ = tx.send(item);
        }
    });
    (Cached { rx: rx }, handle)
}

// ================================
// Test functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peeking_test() {
        let test = vec!["first", "second", "third"];
        let mut index = 0;
        for (elem, peek) in test.into_iter().peeking() {
            match elem {
                "first" => {
                    assert_eq!(peek, Some("second"));
                    index += 1;
                },
                "second" => {
                    assert_eq!(peek, Some("third"));
                    index += 1;
                },
                "third" => {
                    assert_eq!(peek, None);
                    index += 1;
                },
                _ => assert!(false),
            }
        }
        assert_eq!(index, 3);
    }

    #[test]
    fn filtering_scan_test() {
        let test = vec![1, 3, 1, 3];
        let index = 0;
        let mut sum = 0;
        for elem in test.into_iter().scan_filter(index, |index, x| {
            *index += 1;
            if (*index % 2) == 1 {
                Some(x)
            } else {
                None
            }
        }) {
            sum += elem;
        }
        assert_eq!(sum, 2);
    }

    #[test]
    fn parsing_test() {

        struct ParseState {
            one_halt_test: u32,
            two_counter: u32,
            one_more_test: bool,
        }

        let mut result = vec![];

        let test = vec![1, 2, 2, 3];
        for elem in test.into_iter().parsing(
            ParseState {
                one_halt_test: 3,
                two_counter: 1,
                one_more_test: true,
            },
            |state, elem| {
                match elem {
                    Some(1) => {
                        state.one_halt_test -= 1;
                        if state.one_halt_test == 0 {
                            (ParseResult::Continue, Some(1))
                        } else {
                            (ParseResult::Halt, None)
                        }
                    },
                    Some(2) => {
                        state.two_counter += 1;
                        (ParseResult::Continue, Some(state.two_counter))
                    },
                    Some(3) => {
                        (ParseResult::Continue, Some(4))
                    },
                    Some(_) => { panic!("found a bug in rust") },
                    None => {
                        if state.one_more_test {
                            state.one_more_test = false;
                            (ParseResult::Continue, Some(5))
                        } else {
                            (ParseResult::End, None)
                        }
                    }
                }
            }
        ) {
            result.push(elem);
        }

        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn construct_test() {
        use std::cell::Cell;

        let test: Vec<u8> = vec![3, 3, 3, 2, 4, 5, 2];

        let result : Vec<u8> = test.into_iter().construct_state(0, |ref mut _state, ref mut value, i| {
            if value.is_none() {
                return Some(Cell::new(i)); //first value
            }

            let old_val = value.as_ref().unwrap().get();
            value.as_ref().unwrap().set(old_val + i);

            if value.as_ref().unwrap().get() >= 5 {
                Some(Cell::new(0)) //new value
            } else {
                None //continue constucting
            }
        }).map(|x| { x.get() % 5 }).collect();

        assert_eq!(result, vec![1, 0, 4, 2]);
    }
}
