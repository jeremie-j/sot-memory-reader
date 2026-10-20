use std::fs::File;
use std::path::Path;

use memmap2::Mmap;

use super::{DumpManifest, DumpedRegion, DATA_FILE, FORMAT_VERSION, MANIFEST_FILE};
use crate::{MemError, Memory};

/// A dump loaded for reading. `memory.bin` is memory-mapped, so opening a multi-GB
/// dump is instant and only the pages actually read are loaded from disk.
pub struct DumpMem {
    manifest: DumpManifest,
    data: Mmap,
}

impl DumpMem {
    pub fn open(dir: &Path) -> Result<Self, MemError> {
        let mut manifest: DumpManifest =
            serde_json::from_reader(File::open(dir.join(MANIFEST_FILE))?)?;
        if manifest.format_version != FORMAT_VERSION {
            return Err(MemError::InvalidDump(format!(
                "format version {} (expected {FORMAT_VERSION})",
                manifest.format_version
            )));
        }

        // SAFETY: the dump file is not expected to be modified while it is mapped.
        let data = unsafe { Mmap::map(&File::open(dir.join(DATA_FILE))?)? };

        manifest.regions.sort_by_key(|r| r.start);
        for r in &manifest.regions {
            if r.data_offset + r.len() > data.len() as u64 {
                return Err(MemError::InvalidDump(format!(
                    "region {:#x}-{:#x} extends past the end of {DATA_FILE}",
                    r.start, r.end
                )));
            }
        }

        Ok(Self { manifest, data })
    }

    pub fn manifest(&self) -> &DumpManifest {
        &self.manifest
    }

    /// The region containing `addr`, found by binary search on the sorted starts.
    pub fn region_at(&self, addr: u64) -> Option<&DumpedRegion> {
        let regions = &self.manifest.regions;
        let i = regions.partition_point(|r| r.start <= addr).checked_sub(1)?;
        Some(&regions[i]).filter(|r| addr < r.end)
    }
}

impl Memory for DumpMem {
    fn read_into(&self, addr: u64, buf: &mut [u8]) -> Result<(), MemError> {
        let end = addr.checked_add(buf.len() as u64).ok_or_else(|| unreadable_at(addr, buf))?;

        // Reads spanning two adjacent regions are rejected. The kernel merges adjacent
        // mappings with identical permissions, so this only happens across a permission change.
        let region = self
            .region_at(addr)
            .filter(|r| end <= r.end)
            .ok_or_else(|| unreadable_at(addr, buf))?;
        if region.holes.iter().any(|&(s, e)| addr < e && s < end) {
            return Err(unreadable_at(addr, buf));
        }

        let offset = (region.data_offset + (addr - region.start)) as usize;
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }
}

fn unreadable_at(addr: u64, buf: &[u8]) -> MemError {
    MemError::Unreadable { addr, len: buf.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dump::writer::{write_dump, DumpInfo};
    use crate::maps::{MapRegion, Perms};

    /// Fake process: two regions, the second one has an unreadable page.
    struct FakeMem;

    const A: u64 = 0x10_0000;
    const B: u64 = 0x20_0000;

    impl Memory for FakeMem {
        fn read_into(&self, addr: u64, buf: &mut [u8]) -> Result<(), MemError> {
            let end = addr + buf.len() as u64;
            if addr < B + 0x2000 && B + 0x1000 < end {
                return Err(unreadable_at(addr, buf));
            }
            for (i, b) in buf.iter_mut().enumerate() {
                *b = (addr + i as u64) as u8;
            }
            Ok(())
        }
    }

    fn region(start: u64, len: u64) -> MapRegion {
        let perms = Perms { read: true, write: true, exec: false, shared: false };
        MapRegion { start, end: start + len, perms, offset: 0, inode: 0, path: None }
    }

    #[test]
    fn roundtrip() {
        let dir = std::env::temp_dir().join(format!("memory-dump-test-{}", std::process::id()));
        let regions = [region(B, 0x3000), region(A, 0x2000)];
        let info = DumpInfo { pid: 1, process_name: "fake".into(), module_base: Some(A) };
        write_dump(&FakeMem, &regions, info, &dir).unwrap();

        let dump = DumpMem::open(&dir).unwrap();
        assert_eq!(dump.manifest().module_base, Some(A));
        assert_eq!(dump.read::<u8>(A + 0x42).unwrap(), 0x42);
        assert_eq!(dump.read::<u32>(B + 0x2ffc).unwrap(), u32::from_le_bytes([0xfc, 0xfd, 0xfe, 0xff]));
        assert_eq!(dump.manifest().regions[1].holes, vec![(B + 0x1000, B + 0x2000)]);

        assert!(dump.read::<u8>(B + 0x1800).is_err(), "inside hole");
        assert!(dump.read::<u64>(A + 0x1ffc).is_err(), "past region end");
        assert!(dump.read::<u8>(0x1234).is_err(), "unmapped");

        std::fs::remove_dir_all(dir).unwrap();
    }
}
