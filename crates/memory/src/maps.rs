//! Parser for `/proc/<pid>/maps`.
//!
//! ```text
//! start-end         perms offset   dev   inode  path
//! 140000000-140001000 r--p 00000000 08:02 1234   /games/SoTGame.exe
//! 7f3a1c000000-7f3a1c021000 rw-p 00000000 00:00 0
//! ```

use serde::{Deserialize, Serialize};

use crate::MemError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Perms {
    pub read: bool,
    pub write: bool,
    pub exec: bool,
    /// `s` (shared) vs `p` (private, copy-on-write).
    pub shared: bool,
}

impl Perms {
    fn parse(s: &str) -> Option<Self> {
        let b = s.as_bytes();
        if b.len() != 4 {
            return None;
        }
        Some(Self {
            read: b[0] == b'r',
            write: b[1] == b'w',
            exec: b[2] == b'x',
            shared: b[3] == b's',
        })
    }
}

impl std::fmt::Display for Perms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flag = |set, c| if set { c } else { '-' };
        write!(
            f,
            "{}{}{}{}",
            flag(self.read, 'r'),
            flag(self.write, 'w'),
            flag(self.exec, 'x'),
            if self.shared { 's' } else { 'p' }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapRegion {
    pub start: u64,
    pub end: u64,
    pub perms: Perms,
    /// Offset into the backing file (0 for anonymous mappings).
    pub offset: u64,
    pub inode: u64,
    /// File path, `[heap]`/`[stack]`/`[vvar]`-style pseudo name, or `None` for anonymous memory.
    pub path: Option<String>,
}

impl MapRegion {
    pub fn len(&self) -> u64 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn is_anonymous(&self) -> bool {
        self.path.is_none()
    }

    /// Kernel pseudo regions: `[heap]`, `[stack]`, `[vvar]`, `[vdso]`, `[vsyscall]`, ...
    pub fn is_pseudo(&self) -> bool {
        self.path.as_deref().is_some_and(|p| p.starts_with('['))
    }

    /// True if the backing file name (last path component) equals `name`.
    pub fn file_name_is(&self, name: &str) -> bool {
        self.path
            .as_deref()
            .and_then(|p| p.rsplit(['/', '\\']).next())
            .is_some_and(|file| file.eq_ignore_ascii_case(name))
    }

    pub fn parse(line: &str) -> Option<Self> {
        // The first five fields are separated by single spaces; the path is padded with
        // spaces and may itself contain spaces, so it is the trimmed remainder.
        let mut fields = line.splitn(6, ' ');
        let (start, end) = fields.next()?.split_once('-')?;
        let perms = Perms::parse(fields.next()?)?;
        let offset = u64::from_str_radix(fields.next()?, 16).ok()?;
        let _dev = fields.next()?;
        let inode = fields.next()?.parse().ok()?;
        let path = fields
            .next()
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(String::from);

        Some(Self {
            start: u64::from_str_radix(start, 16).ok()?,
            end: u64::from_str_radix(end, 16).ok()?,
            perms,
            offset,
            inode,
            path,
        })
    }
}

pub fn parse_maps(text: &str) -> Result<Vec<MapRegion>, MemError> {
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            MapRegion::parse(l).ok_or_else(|| MemError::InvalidDump(format!("bad maps line: {l}")))
        })
        .collect()
}

pub fn read_maps(pid: u32) -> Result<Vec<MapRegion>, MemError> {
    parse_maps(&std::fs::read_to_string(format!("/proc/{pid}/maps"))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
140000000-140001000 r--p 00000000 08:02 1234                       /home/me/.steam/steamapps/common/Sea of Thieves/SoTGame.exe
7f3a1c000000-7f3a1c021000 rw-p 00000000 00:00 0
7f3a20000000-7f3a24000000 ---p 00000000 00:00 0
7ffd5e1f0000-7ffd5e1f4000 r--p 00000000 00:00 0                    [vvar]
";

    #[test]
    fn parses_sample() {
        let regions = parse_maps(SAMPLE).unwrap();
        assert_eq!(regions.len(), 4);

        let exe = &regions[0];
        assert_eq!(exe.start, 0x1_4000_0000);
        assert_eq!(exe.len(), 0x1000);
        assert!(exe.perms.read && !exe.perms.write);
        assert!(exe.file_name_is("sotgame.exe"));
        assert!(exe.path.as_deref().unwrap().ends_with("Sea of Thieves/SoTGame.exe"));

        assert!(regions[1].is_anonymous());
        assert!(regions[1].perms.write);
        assert!(!regions[2].perms.read);
        assert!(regions[3].is_pseudo());
        assert_eq!(regions[3].perms.to_string(), "r--p");
    }
}
