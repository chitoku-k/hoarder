pub trait CloneableIterator: Iterator {
    fn clone_box<'a>(&self) -> Box<dyn CloneableIterator<Item = Self::Item> + 'a>
    where
        Self: 'a;
}

impl<T> CloneableIterator for T
where
    T: Clone + Iterator,
{
    fn clone_box<'a>(&self) -> Box<dyn CloneableIterator<Item = Self::Item> + 'a>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}
