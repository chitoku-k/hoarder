use dyn_clone::DynClone;

pub trait CloneableIterator: DynClone + Iterator {}

impl<T> CloneableIterator for T
where
    T: Clone + Iterator,
{
}
