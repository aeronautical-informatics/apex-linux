use std::io::ErrorKind;
use std::os::unix::prelude::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use nix::libc::{c_uint, syscall, SYS_pidfd_open};
use nix::unistd::Pid;
use polling::{Event, Poller};

use crate::error::{ResultExt, SystemError, TypedError, TypedResult};

#[derive(Debug)]
pub enum PidWaitError {
    Timeout,
    Err(TypedError),
}

impl From<TypedError> for PidWaitError {
    fn from(e: TypedError) -> Self {
        Self::Err(e)
    }
}

#[derive(Debug)]
pub struct PidFd(OwnedFd);

impl PidFd {
    pub fn wait_exited_timeout(&self, timeout: Duration) -> Result<(), PidWaitError> {
        let now = Instant::now();

        let poller = Poller::new()
            .map_err(anyhow::Error::from)
            .typ(SystemError::Panic)?;
        poller
            .add(self.0.as_raw_fd(), Event::readable(0))
            .map_err(anyhow::Error::from)
            .typ(SystemError::Panic)?;

        loop {
            let poll_res = poller.wait(
                Vec::new().as_mut(),
                Some(timeout.saturating_sub(now.elapsed())),
            );
            match poll_res {
                Ok(0) => return Err(PidWaitError::Timeout),
                Ok(_) => return Ok(()),
                Err(e) => {
                    if e.kind() != ErrorKind::Interrupted {
                        return Err(e)
                            .map_err(anyhow::Error::from)
                            .typ(SystemError::Panic)
                            .map_err(PidWaitError::Err);
                    }
                }
            }
            poller
                .modify(self.0.as_raw_fd(), Event::readable(0))
                .map_err(anyhow::Error::from)
                .typ(SystemError::Panic)?;
        }
    }
}

impl AsRawFd for PidFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl TryFrom<Pid> for PidFd {
    type Error = TypedError;

    fn try_from(value: Pid) -> TypedResult<Self> {
        let pidfd: i32 = unsafe {
            syscall(SYS_pidfd_open, value.as_raw(), 0 as c_uint)
                .try_into()
                .typ(SystemError::Panic)?
        };
        if pidfd < 0 {
            return Err(anyhow!("Error getting pidfd from {value}. {pidfd}"))
                .typ(SystemError::Panic);
        }
        Ok(PidFd(unsafe { OwnedFd::from_raw_fd(pidfd) }))
    }
}
