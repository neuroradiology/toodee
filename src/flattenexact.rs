#![allow(missing_debug_implementations)]

/// An iterator that behaves like `core::iter::adapters::Flatten` but has the added advantage of implementing
/// `ExactSizeIterator` (we know how many cells there are per row in a `TooDee` array).
pub struct FlattenExact<I>
where
    I : Iterator + ExactSizeIterator + DoubleEndedIterator,
    I::Item : IntoIterator,
    <I::Item as IntoIterator>::IntoIter : Iterator + DoubleEndedIterator + ExactSizeIterator,
{
    iter: I,
    len_per_iter: usize,
    frontiter: Option<<I::Item as IntoIterator>::IntoIter>,
    backiter: Option<<I::Item as IntoIterator>::IntoIter>,
}

impl<I> FlattenExact<I>
where
    I : Iterator + ExactSizeIterator + DoubleEndedIterator,
    I::Item : IntoIterator,
    <I::Item as IntoIterator>::IntoIter : Iterator + DoubleEndedIterator + ExactSizeIterator,
{
    pub(super) fn new(iter: I, len_per_iter : usize) -> FlattenExact<I> {
        FlattenExact { iter, len_per_iter, frontiter: None, backiter: None }
    }
}

impl<I> Iterator for FlattenExact<I>
where
    I : Iterator + ExactSizeIterator + DoubleEndedIterator,
    I::Item : IntoIterator,
    <I::Item as IntoIterator>::IntoIter : Iterator + DoubleEndedIterator + ExactSizeIterator,
{
    type Item = <I::Item as IntoIterator>::Item;

    #[inline]
    fn next(&mut self) -> Option<<I::Item as IntoIterator>::Item> {
        loop {
            if let Some(ref mut inner) = self.frontiter {
                if let elt @ Some(_) = inner.next() {
                    return elt;
                }
            }
            match self.iter.next() {
                None => return self.backiter.as_mut()?.next(),
                Some(inner) => self.frontiter = Some(inner.into_iter()),
            }
        }
    }
    
    #[inline]
    fn nth(&mut self, mut n: usize) -> Option<<I::Item as IntoIterator>::Item> {
        
        if self.len_per_iter == 0 {
            return None;
        }
        
        if let Some(ref mut inner) = self.frontiter {
            if n < inner.len() {
                return inner.nth(n);
            } else {
                n -= inner.len();
                self.frontiter = None;
            }
        }
        
        let iter_skip = self.iter.len().min(n / self.len_per_iter);
        if let Some(inner) = self.iter.nth(iter_skip) {
            let mut tmp = inner.into_iter();
            n -= iter_skip * self.len_per_iter;
            debug_assert!(n < tmp.len());
            let ret_val = tmp.nth(n);
            self.frontiter = Some(tmp);
            ret_val
        } else {
            n -= iter_skip * self.len_per_iter;
            self.backiter.as_mut()?.nth(n)
        }
        
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut len = self.len_per_iter * self.iter.len();
        len += self.frontiter.as_ref().map_or(0, |i| i.len());
        len += self.backiter.as_ref().map_or(0, |i| i.len());
        (len, Some(len))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
    
    #[inline]
    #[allow(clippy::toplevel_ref_arg)]
    fn fold<Acc, Fold>(self, init: Acc, ref mut fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        #[inline]
        fn flatten<U: Iterator, Acc>(
            fold: &mut impl FnMut(Acc, U::Item) -> Acc,
        ) -> impl FnMut(Acc, U) -> Acc + '_ {
            move |acc, iter| iter.fold(acc, &mut *fold)
        }

        self.frontiter
            .into_iter()
            .chain(self.iter.map(IntoIterator::into_iter))
            .chain(self.backiter)
            .fold(init, flatten(fold))
    }
    
}

impl<I> DoubleEndedIterator for FlattenExact<I>
where
    I : Iterator + ExactSizeIterator + DoubleEndedIterator,
    I::Item : IntoIterator,
    <I::Item as IntoIterator>::IntoIter : Iterator + DoubleEndedIterator + ExactSizeIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<<I::Item as IntoIterator>::Item> {
        loop {
            if let Some(ref mut inner) = self.backiter {
                if let elt @ Some(_) = inner.next_back() {
                    return elt;
                }
            }
            match self.iter.next_back() {
                None => return self.frontiter.as_mut()?.next_back(),
                Some(next) => self.backiter = Some(next.into_iter()),
            }
        }
    }

    #[inline]
    fn nth_back(&mut self, mut n: usize) -> Option<<I::Item as IntoIterator>::Item> {
        
        if self.len_per_iter == 0 {
            return None;
        }
        
        if let Some(ref mut inner) = self.backiter {
            if n < inner.len() {
                return inner.nth_back(n);
            } else {
                n -= inner.len();
                self.backiter = None;
            }
        }
        
        let iter_skip = self.iter.len().min(n / self.len_per_iter);
        if let Some(inner) = self.iter.nth_back(iter_skip) {
            let mut tmp = inner.into_iter();
            n -= iter_skip * self.len_per_iter;
            debug_assert!(n < tmp.len());
            let ret_val = tmp.nth_back(n);
            self.backiter = Some(tmp);
            ret_val
        } else {
            n -= iter_skip * self.len_per_iter;
            self.frontiter.as_mut()?.nth_back(n)
        }
        
    }
    
    #[inline]
    #[allow(clippy::toplevel_ref_arg)]
    fn rfold<Acc, Fold>(self, init: Acc, ref mut fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        #[inline]
        fn flatten<U: DoubleEndedIterator, Acc>(
            fold: &mut impl FnMut(Acc, U::Item) -> Acc,
        ) -> impl FnMut(Acc, U) -> Acc + '_ {
            move |acc, iter| iter.rfold(acc, &mut *fold)
        }

        self.frontiter
            .into_iter()
            .chain(self.iter.map(IntoIterator::into_iter))
            .chain(self.backiter)
            .rfold(init, flatten(fold))
    }
    
}

impl<I> ExactSizeIterator for FlattenExact<I>
where
    I : Iterator + ExactSizeIterator + DoubleEndedIterator,
    I::Item : IntoIterator,
    <I::Item as IntoIterator>::IntoIter : Iterator + DoubleEndedIterator + ExactSizeIterator,
{}

