use nix::fcntl;
use nix::sys::signal;
use nix::unistd;
use std::convert::TryFrom;
use std::mem;
use std::os::unix::io::RawFd;

static mut PIPE: (RawFd, RawFd) = (-1, -1);

extern "C" fn handler(signum: libc::c_int) {
    unsafe {
        let _ = unistd::write(PIPE.1, &signum.to_le_bytes());
    }
}

pub struct SignalHandler {
    fd: RawFd,
    old_sig_action: signal::SigAction,
    signal: signal::Signal,
}

impl SignalHandler {
    pub fn install(signal: signal::Signal) -> nix::Result<Self> {
        if PIPE == (-1, -1) {
            unsafe {
                PIPE = unistd::pipe2(fcntl::OFlag::O_NONBLOCK)?;
            }
        }

        let sig_action = signal::SigAction::new(
            signal::SigHandler::Handler(handler),
            signal::SaFlags::empty(),
            signal::SigSet::empty()
        );

        let old_sig_action = unsafe {
            match signal::sigaction(signal, &sig_action) {
                Ok(old_sig_action) => old_sig_action,
                Err(error) => {
                    let _ = unistd::close(PIPE.1);
                    let _ = unistd::close(PIPE.0);
                    PIPE = (-1, -1);
                    return Err(error);
                },
            }
        };

        Ok(Self {
            fd: unsafe { PIPE.0 },
            old_sig_action,
            signal,
        })
    }

    pub fn fd(&self) -> RawFd {
        self.fd
    }

    pub fn try_read(&self) -> nix::Result<signal::Signal> {
        let mut buffer = [0; mem::size_of::<libc::c_int>()];
        unistd::read(self.fd, &mut buffer[..])?;
        let signum: libc::c_int = mem::transmute(buffer);
        signal::Signal::try_from(signum)
    }
}

impl Drop for SignalHandler {
    fn drop(&mut self) {
        signal::sigaction(self.signal, &self.old_sig_action);
    }
}
