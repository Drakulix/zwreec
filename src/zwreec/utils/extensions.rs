use std::iter::Peekable;

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

pub trait PeekingExt {
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

pub trait FilteringScanExt {
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
