use error::*;

use mio::event::Evented;
use mio::unix::EventedFd;
use mio::{Poll, PollOpt, Ready, Token};

use nix::fcntl::{self, open, OFlag};
use nix::pty::{grantpt, posix_openpt, unlockpt, PtyMaster};
use nix::sys::stat::Mode;

use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;

mod pts_namer {
    use error::*;

    use nix::pty::{ptsname as unsafe_ptsname, PtyMaster};

    use std::sync::Mutex;

    struct PtsNamer;

    impl !Sync for PtsNamer {}

    impl PtsNamer {
        fn ptsname(&self, master: &PtyMaster) -> Result<String> {
            let name = unsafe { unsafe_ptsname(master)? };
            Ok(name)
        }
    }

    lazy_static! {
        static ref PTS_NAMER: Mutex<PtsNamer> = Mutex::new(PtsNamer);
    }

    pub fn ptsname(master: &PtyMaster) -> Result<String> {
        PTS_NAMER.lock().unwrap().ptsname(master)
    }
}

use self::pts_namer::ptsname;

pub fn pair() -> Result<(Master, File)> {
    let master = posix_openpt(OFlag::O_RDWR | OFlag::O_CLOEXEC | OFlag::O_NONBLOCK)?;

    grantpt(&master)?;
    unlockpt(&master)?;

    let slave_name = ptsname(&master)?;

    let slave_fd = open(
        Path::new(&slave_name),
        OFlag::O_RDWR | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?;
    let slave = unsafe { File::from_raw_fd(slave_fd) };

    Ok((Master(master), slave))
}

pub struct Master(PtyMaster);

impl Evented for Master {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.0.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.0.as_raw_fd()).deregister(poll)
    }
}

mod pty_helper {
    use std::os::unix::io::RawFd;

    extern "C" {
        pub fn tiocsctty(fd: RawFd) -> bool;
    }
}

fn set_tty<F: AsRawFd>(fd: &F) -> io::Result<()> {
    let result = unsafe { pty_helper::tiocsctty(fd.as_raw_fd()) };
    if result {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub trait CommandTty {
    fn tty(&mut self, file: File) -> io::Result<&mut Self>;
}

impl CommandTty for Command {
    fn tty(&mut self, file: File) -> io::Result<&mut Self> {
        let stdin = file.try_clone()?;
        let stdout = file.try_clone()?;
        let stderr = file.try_clone()?;

        self.stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .before_exec(move || {
                match ::nix::unistd::setsid() {
                    Ok(_) => (),
                    Err(::nix::Error::Sys(e)) => {
                        return Err(io::Error::from_raw_os_error(e as i32));
                    }
                    _ => return Err(io::Error::last_os_error()),
                }

                set_tty(&file)
            });

        Ok(self)
    }
}
