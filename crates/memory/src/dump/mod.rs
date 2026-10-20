//! On-disk dump of a process's memory.
//!
//! ```text
//! <dir>/manifest.json   DumpManifest: metadata + one entry per dumped region
//! <dir>/memory.bin      raw bytes of every region, concatenated in manifest order
//! ```
//!
//! Unreadable pages are written as zeros (so each region stays contiguous in
//! `memory.bin`) and listed in `holes`, so `DumpMem` can refuse to read them.

pub mod filter;
pub mod reader;
pub mod writer;

use serde::{Deserialize, Serialize};

pub const MANIFEST_FILE: &str = "manifest.json";
pub const DATA_FILE: &str = "memory.bin";
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpManifest {
    pub format_version: u32,
    pub pid: u32,
    pub process_name: String,
    pub created_at_unix: u64,
    /// Lowest mapped address of the main module (e.g. `SoTGame.exe`), if found.
    #[serde(with = "hex::option")]
    pub module_base: Option<u64>,
    pub regions: Vec<DumpedRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DumpedRegion {
    #[serde(with = "hex")]
    pub start: u64,
    #[serde(with = "hex")]
    pub end: u64,
    pub perms: String,
    pub path: Option<String>,
    /// Where this region's bytes start in `memory.bin`.
    pub data_offset: u64,
    /// Absolute `[start, end)` address ranges that could not be read (zero-filled in `memory.bin`).
    #[serde(with = "hex::ranges")]
    pub holes: Vec<(u64, u64)>,
}

impl DumpedRegion {
    pub fn len(&self) -> u64 {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Addresses are stored as `"0x7f3a1c000000"` strings so the manifest is readable by a human.
mod hex {
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{v:#x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        parse(&String::deserialize(d)?).map_err(D::Error::custom)
    }

    fn parse(s: &str) -> Result<u64, String> {
        let digits = s.strip_prefix("0x").ok_or_else(|| format!("expected 0x prefix: {s}"))?;
        u64::from_str_radix(digits, 16).map_err(|e| format!("{s}: {e}"))
    }

    pub mod option {
        use super::*;

        pub fn serialize<S: Serializer>(v: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
            match v {
                Some(v) => super::serialize(v, s),
                None => s.serialize_none(),
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
            Option::<String>::deserialize(d)?
                .map(|s| parse(&s).map_err(D::Error::custom))
                .transpose()
        }
    }

    pub mod ranges {
        use super::*;
        use serde::ser::SerializeSeq;

        pub fn serialize<S: Serializer>(v: &[(u64, u64)], s: S) -> Result<S::Ok, S::Error> {
            let mut seq = s.serialize_seq(Some(v.len()))?;
            for (start, end) in v {
                seq.serialize_element(&[format!("{start:#x}"), format!("{end:#x}")])?;
            }
            seq.end()
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<(u64, u64)>, D::Error> {
            Vec::<[String; 2]>::deserialize(d)?
                .iter()
                .map(|[a, b]| Ok((parse(a)?, parse(b)?)))
                .collect::<Result<_, String>>()
                .map_err(D::Error::custom)
        }
    }
}
