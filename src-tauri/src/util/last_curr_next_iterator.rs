pub struct LastCurrNextIterator<T>
where
    T: Iterator,
{
    inner: T,
    curr_item: Option<T::Item>,
    next_item: Option<T::Item>,
}

impl<T> LastCurrNextIterator<T>
where
    T: Iterator,
{
    pub fn new(mut inner: T) -> LastCurrNextIterator<T> {
        LastCurrNextIterator {
            curr_item: None,
            next_item: inner.next(),
            inner,
        }
    }
}

impl<T> Iterator for LastCurrNextIterator<T>
where
    T: Iterator,
    T::Item: Copy,
    T::Item: PartialEq,
{
    type Item = (Option<T::Item>, T::Item, Option<T::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_item {
            Some(i) => {
                let last_item = self.curr_item;
                self.curr_item = Some(i);
                self.next_item = self.inner.next();

                Some((last_item, i, self.next_item))
            }
            None => None,
        }
    }
}

pub trait IntoLastCurrNextIterator<T>
where
    T: Iterator,
    Self: Sized,
{
    fn last_curr_next(self) -> LastCurrNextIterator<T>;
}

impl<T> IntoLastCurrNextIterator<T> for T
where
    T: Iterator,
{
    fn last_curr_next(self) -> LastCurrNextIterator<T> {
        LastCurrNextIterator::new(self)
    }
}
