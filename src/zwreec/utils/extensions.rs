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
