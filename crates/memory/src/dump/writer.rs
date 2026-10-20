use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{DumpManifest, DumpedRegion, DATA_FILE, FORMAT_VERSION, MANIFEST_FILE};
use crate::maps::MapRegion;
use crate::{MemError, Memory};

const CHUNK: u64 = 1 << 20;
const PAGE: u64 = 0x1000;

pub struct DumpInfo {
    pub pid: u32,
    pub process_name: String,
    pub module_base: Option<u64>,
}

/// Write `regions` of `mem` to `out_dir`. Generic over `Memory` so it can be tested
/// without a live process.
pub fn write_dump(
    mem: &impl Memory,
    regions: &[MapRegion],
    info: DumpInfo,
    out_dir: &Path,
) -> Result<DumpManifest, MemError> {
    fs::create_dir_all(out_dir)?;
    let mut data = BufWriter::new(File::create(out_dir.join(DATA_FILE))?);
    let mut dumped = Vec::with_capacity(regions.len());
    let mut data_offset = 0;
    let total: u64 = regions.iter().map(MapRegion::len).sum();
    let mut done = 0;

    for region in regions {
        let holes = copy_region(mem, region, &mut data)?;
        dumped.push(DumpedRegion {
            start: region.start,
            end: region.end,
            perms: region.perms.to_string(),
            path: region.path.clone(),
            data_offset,
            holes,
        });
        data_offset += region.len();
        done += region.len();
        eprint!("\r{:>6} / {} MiB", done >> 20, total >> 20);
    }
    eprintln!();
    data.flush()?;

    let manifest = DumpManifest {
        format_version: FORMAT_VERSION,
        pid: info.pid,
        process_name: info.process_name,
        created_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        module_base: info.module_base,
        regions: dumped,
    };
    serde_json::to_writer_pretty(File::create(out_dir.join(MANIFEST_FILE))?, &manifest)?;
    Ok(manifest)
}

/// Copy one region chunk by chunk. A failed chunk is retried page by page, and
/// pages that still fail are zero-filled and reported as holes.
fn copy_region(
    mem: &impl Memory,
    region: &MapRegion,
    out: &mut impl Write,
) -> Result<Vec<(u64, u64)>, MemError> {
    let mut holes: Vec<(u64, u64)> = Vec::new();
    let mut buf = vec![0u8; CHUNK as usize];
    let mut addr = region.start;

    while addr < region.end {
        let len = CHUNK.min(region.end - addr);
        let chunk = &mut buf[..len as usize];

        if mem.read_into(addr, chunk).is_err() {
            for (i, page) in chunk.chunks_mut(PAGE as usize).enumerate() {
                let page_addr = addr + i as u64 * PAGE;
                if mem.read_into(page_addr, page).is_err() {
                    page.fill(0);
                    let page_end = page_addr + page.len() as u64;
                    match holes.last_mut() {
                        Some(last) if last.1 == page_addr => last.1 = page_end,
                        _ => holes.push((page_addr, page_end)),
                    }
                }
            }
        }

        out.write_all(chunk)?;
        addr += len;
    }
    Ok(holes)
}
