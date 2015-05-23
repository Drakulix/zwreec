use std::collections::VecDeque;
use std::str::Chars;
use std::any::Any;

/// An iterator that filters the elements of `iter` with `predicate`
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct QueuedScan<I, St, F> where
   I: Iterator,
   F: FnMut(&mut St, Option<I::Item>, &mut VecDeque<I::Item>) -> bool
 {
    iter: I,
    f: F,
    state: St,
    queue: VecDeque<I::Item>,
}

impl<I, St, F> Iterator for QueuedScan<I, St, F> where
    I: Iterator,
    F: FnMut(&mut St, Option<I::Item>, &mut VecDeque<I::Item>) -> bool,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
        loop {
            let elem: Option<I::Item> = self.queue.pop_front();
            match elem {
                Some(item) => return Some(item),
                None => {
                    if !((self.f)(&mut self.state, self.iter.next(), &mut self.queue)) {
                        return None;
                    }
                },
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None) //cannot know any bound
    }
}

pub trait QueuedScanExtension {
    fn queued_scan<St, F>(mut self, initial_state: St, f: F) -> QueuedScan<Self, St, F>
        where Self: Sized+Iterator, F: FnMut(&mut St, Option<Self::Item>, &mut VecDeque<Self::Item>) -> bool;
}

impl<A: Sized+Iterator> QueuedScanExtension for A {
    fn queued_scan<St, F>(mut self, initial_state: St, f: F) -> QueuedScan<A, St, F>
        where Self: Sized+Iterator, F: FnMut(&mut St, Option<A::Item>, &mut VecDeque<A::Item>) -> bool
    {
        QueuedScan {iter: self, f: f, state: initial_state, queue: VecDeque::<A::Item>::new()}
    }
}
