use crate::error::*;

use nix::sys::select::{pselect, FdSet};
use nix::sys::socket::{
    accept4, bind, listen, socket, AddressFamily, SockAddr, SockFlag, SockType,
};
use nix::sys::time::{TimeSpec, TimeValLike};
use nix::unistd::close;

use std::io;
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

#[derive(Debug)]
pub struct Listener {
    fd: RawFd,
}

impl Drop for Listener {
    fn drop(&mut self) {
        close(self.fd).unwrap();
    }
}

impl Listener {
    fn prepare_socket(fd: RawFd, path: &Path) -> Result<()> {
        let addr = SockAddr::new_unix(path)?;
        bind(fd, &addr)?;

        listen(fd, 5)?;

        Ok(())
    }

    pub fn bind<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if let Err(e) = std::fs::remove_file(path) {
            if e.kind() != io::ErrorKind::NotFound {
                bail!(e);
            }
        }

        let fd = socket(
            AddressFamily::Unix,
            SockType::Stream,
            SockFlag::SOCK_NONBLOCK | SockFlag::SOCK_CLOEXEC,
            None,
        )?;

        if let Err(e) = Self::prepare_socket(fd, path) {
            close(fd).unwrap();
            return Err(e);
        }

        Ok(Self { fd })
    }

    pub fn accept(&self, timeout: Duration) -> Result<Option<UnixStream>> {
        assert!(timeout.as_secs() <= i64::max_value() as u64);

        let mut rd_set = FdSet::new();
        rd_set.insert(self.fd);

        let timespec = TimeSpec::seconds(timeout.as_secs() as i64)
            + TimeSpec::nanoseconds(timeout.subsec_nanos() as i64);

        // TODO: See if EAGAIN needs to be handled here.
        pselect(self.fd + 1, &mut rd_set, None, None, &timespec, None)?;

        if !rd_set.contains(self.fd) {
            return Ok(None);
        }

        let fd = accept4(self.fd, SockFlag::SOCK_CLOEXEC)?;

        let uds = unsafe { UnixStream::from_raw_fd(fd) };

        Ok(Some(uds))
    }
}
