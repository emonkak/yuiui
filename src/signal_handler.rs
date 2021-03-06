use nix::fcntl;
use nix::sys::signal;
use nix::unistd;
use std::os::unix::io::RawFd;

static mut PIPE: (RawFd, RawFd) = (-1, -1);

extern "C" fn handler(signum: libc::c_int) {
    unsafe {
        let _ = unistd::write(PIPE.1, &signum.to_le_bytes());
    }
}

pub fn install(signals: &[signal::Signal]) -> Result<RawFd, nix::Error> {
    unsafe {
        PIPE = unistd::pipe2(fcntl::OFlag::O_NONBLOCK)?;
    }

    let sig_action = signal::SigAction::new(
        signal::SigHandler::Handler(handler),
        signal::SaFlags::empty(),
        signal::SigSet::empty()
    );

    for signal in signals {
        if let Err(e) = unsafe { signal::sigaction(*signal, &sig_action) } {
            unsafe {
                let _ = unistd::close(PIPE.1);
                let _ = unistd::close(PIPE.0);
            }
            return Err(e);
        }
    }

    unsafe {
        Ok(PIPE.0)
    }
}
