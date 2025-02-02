use std::io::{BufRead, Seek};

pub trait SeekableBufRead: BufRead + Seek {}

impl<T> SeekableBufRead for T
where
    T: BufRead + Seek,
{
}
