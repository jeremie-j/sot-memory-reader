//! Which mapped regions are worth dumping.

use crate::maps::MapRegion;

/// Decide whether `region` goes into the dump. `module` is the main executable's
/// file name (e.g. `"SoTGame.exe"`), whose regions hold the code to pattern-scan and
/// the globals (GWorld, GNames, GObjects).
///
/// Useful helpers on `MapRegion`: `perms.{read,write,exec,shared}`, `is_anonymous()`,
/// `is_pseudo()`, `file_name_is(name)`, `len()`, `path`.
pub fn should_dump(region: &MapRegion, module: &str) -> bool {
    region.file_name_is(module) || (region.is_anonymous() && region.perms.read)
}
