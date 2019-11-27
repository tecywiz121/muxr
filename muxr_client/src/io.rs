use std::io::{Read, Write};
use std::sync::Arc;

pub fn split<T>(item: T) -> (ReadHalf<T>, WriteHalf<T>) {
    let inner = Arc::new(item);
    (ReadHalf(inner.clone()), WriteHalf(inner))
}

#[derive(Debug)]
pub struct ReadHalf<T>(Arc<T>);

impl<T> Read for ReadHalf<T>
where
    for<'a> &'a T: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        (&*self.0).read(buf)
    }
}

#[derive(Debug)]
pub struct WriteHalf<T>(Arc<T>);

impl<T> Write for WriteHalf<T>
where
    for<'a> &'a T: Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        (&*self.0).write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        (&*self.0).flush()
    }
}
