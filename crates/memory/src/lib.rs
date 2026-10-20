//! Reading another process's memory, live (`ProcMem`, Linux only) or from a dump (`DumpMem`).

pub mod dump;
pub mod maps;
#[cfg(target_os = "linux")]
pub mod process;

pub use dump::reader::DumpMem;
#[cfg(target_os = "linux")]
pub use process::ProcMem;

#[derive(Debug, thiserror::Error)]
pub enum MemError {
    #[error("{len} bytes at {addr:#x} are not readable")]
    Unreadable { addr: u64, len: usize },
    #[error("invalid dump: {0}")]
    InvalidDump(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// A readable address space. Implemented by the live process and by dumps, so
/// everything built on top can run against either.
pub trait Memory: Send + Sync {
    /// Fill `buf` with the bytes at `addr`. Either the whole buffer is read or an error is returned.
    fn read_into(&self, addr: u64, buf: &mut [u8]) -> Result<(), MemError>;

    fn read<T: bytemuck::Pod>(&self, addr: u64) -> Result<T, MemError>
    where
        Self: Sized,
    {
        let mut value = T::zeroed();
        self.read_into(addr, bytemuck::bytes_of_mut(&mut value))?;
        Ok(value)
    }

    fn read_vec(&self, addr: u64, len: usize) -> Result<Vec<u8>, MemError>
    where
        Self: Sized,
    {
        let mut buf = vec![0; len];
        self.read_into(addr, &mut buf)?;
        Ok(buf)
    }
}
