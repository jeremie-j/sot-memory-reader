//! Live process access through `/proc/<pid>/mem`.

use std::fs::{self, File};
use std::os::unix::fs::FileExt;

use crate::maps::{read_maps, MapRegion};
use crate::{MemError, Memory};

pub struct ProcMem {
    pid: u32,
    mem: File,
}

impl ProcMem {
    /// Needs ptrace access to `pid`: same user plus `kernel.yama.ptrace_scope=0`,
    /// or `CAP_SYS_PTRACE` on this binary.
    pub fn open(pid: u32) -> Result<Self, MemError> {
        let mem = File::open(format!("/proc/{pid}/mem"))?;
        Ok(Self { pid, mem })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_alive(&self) -> bool {
        fs::metadata(format!("/proc/{}", self.pid)).is_ok()
    }

    pub fn regions(&self) -> Result<Vec<MapRegion>, MemError> {
        read_maps(self.pid)
    }

    /// First process whose `comm` equals `name`. The kernel truncates `comm` to 15 bytes,
    /// so `name` is truncated the same way before comparing.
    pub fn find_pid(name: &str) -> Result<Option<u32>, MemError> {
        for entry in fs::read_dir("/proc")? {
            let entry = entry?;
            let Some(pid) = entry.file_name().to_str().and_then(|s| s.parse().ok()) else {
                continue;
            };
            // The process may exit between read_dir and here; just skip it.
            let Ok(cmdline) = fs::read(entry.path().join("cmdline")) else {
                continue;
            };
            let Some(file) = cmdline
                .split(|&b| b == 0)
                .next()
                .and_then(|arg0| arg0.rsplit(|&b| b == b'/' || b == b'\\').next())
            else {
                continue;
            };
            if file.eq_ignore_ascii_case(name.as_bytes()) {
                return Ok(Some(pid));
            }
        }
        Ok(None)
    }
}

impl Memory for ProcMem {
    fn read_into(&self, addr: u64, buf: &mut [u8]) -> Result<(), MemError> {
        // pread: no shared cursor, so &self is enough and ProcMem can be shared across threads.
        // Unmapped pages fail with EIO; kernel addresses (> i64::MAX) fail with EINVAL.
        self.mem
            .read_exact_at(buf, addr)
            .map_err(|_| MemError::Unreadable {
                addr,
                len: buf.len(),
            })
    }
}

/// Stops a process with SIGSTOP and resumes it with SIGCONT when dropped,
/// so the memory does not change while it is being dumped.
pub struct FreezeGuard {
    pid: u32,
}

impl FreezeGuard {
    pub fn freeze(pid: u32) -> Result<Self, MemError> {
        signal(pid, libc::SIGSTOP)?;
        Ok(Self { pid })
    }
}

impl Drop for FreezeGuard {
    fn drop(&mut self) {
        if let Err(e) = signal(self.pid, libc::SIGCONT) {
            eprintln!(
                "failed to resume pid {}: {e}. Run `kill -CONT {}`",
                self.pid, self.pid
            );
        }
    }
}

fn signal(pid: u32, sig: libc::c_int) -> Result<(), MemError> {
    // SAFETY: kill(2) has no memory-safety preconditions.
    if unsafe { libc::kill(pid as libc::pid_t, sig) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().into())
    }
}
