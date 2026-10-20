//! sot-dump: capture a process's memory to disk, and inspect the result.
//!
//! ```text
//! sot-dump dump [--name SoTGame.exe | --pid N] [--out DIR] [--no-freeze]   (Linux)
//! sot-dump info DIR
//! sot-dump read DIR ADDR [LEN]
//! ```

use std::path::Path;
use std::process::ExitCode;

use memory::{DumpMem, MemError, Memory};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("dump") => dump(&args[1..]),
        Some("info") if args.len() == 2 => info(Path::new(&args[1])),
        Some("read") if args.len() >= 3 => read(&args[1..]),
        _ => {
            eprintln!("usage:\n  sot-dump dump [--name NAME | --pid N] [--out DIR] [--no-freeze]\n  sot-dump info DIR\n  sot-dump read DIR ADDR [LEN]");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "linux")]
fn dump(args: &[String]) -> Result<(), MemError> {
    use memory::dump::filter::should_dump;
    use memory::dump::writer::{write_dump, DumpInfo};
    use memory::process::FreezeGuard;
    use memory::ProcMem;
    use std::path::PathBuf;

    const DEFAULT_PROCESS: &str = "SoTGame.exe";

    let mut name = DEFAULT_PROCESS.to_string();
    let mut pid = None;
    let mut out = None;
    let mut freeze = true;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().cloned().ok_or_else(|| bad_arg(format!("{arg} needs a value")));
        match arg.as_str() {
            "--name" => name = value()?,
            "--pid" => pid = Some(value()?.parse().map_err(|_| bad_arg("--pid expects a number"))?),
            "--out" => out = Some(PathBuf::from(value()?)),
            "--no-freeze" => freeze = false,
            other => return Err(bad_arg(format!("unknown argument {other}"))),
        }
    }

    let pid = match pid {
        Some(pid) => pid,
        None => ProcMem::find_pid(&name)?.ok_or_else(|| bad_arg(format!("no process named {name}")))?,
    };
    let out = out.unwrap_or_else(|| PathBuf::from(format!("dumps/{pid}")));
    let mem = ProcMem::open(pid)?;

    let _frozen = freeze.then(|| FreezeGuard::freeze(pid)).transpose()?;
    let all = mem.regions()?;
    let module_base = all.iter().filter(|r| r.file_name_is(&name)).map(|r| r.start).min();
    let selected: Vec<_> = all.into_iter().filter(|r| should_dump(r, &name)).collect();
    eprintln!(
        "pid {pid}: dumping {} regions, {} MiB, module base {}",
        selected.len(),
        selected.iter().map(|r| r.len()).sum::<u64>() >> 20,
        module_base.map_or("not found".into(), |b| format!("{b:#x}")),
    );

    let info = DumpInfo { pid, process_name: name, module_base };
    let manifest = write_dump(&mem, &selected, info, &out)?;
    let holes: u64 = manifest.regions.iter().flat_map(|r| &r.holes).map(|(s, e)| e - s).sum();
    eprintln!("written to {} ({} KiB unreadable)", out.display(), holes >> 10);
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn dump(_args: &[String]) -> Result<(), MemError> {
    Err(bad_arg("dumping a live process is only supported on Linux"))
}

fn info(dir: &Path) -> Result<(), MemError> {
    let dump = DumpMem::open(dir)?;
    let m = dump.manifest();
    println!("{} (pid {}), created at unix {}", m.process_name, m.pid, m.created_at_unix);
    if let Some(base) = m.module_base {
        println!("module base {base:#x}");
    }
    for r in &m.regions {
        println!(
            "{:#014x}-{:#014x} {} {:>8} KiB {:>3} holes  {}",
            r.start,
            r.end,
            r.perms,
            r.len() >> 10,
            r.holes.len(),
            r.path.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

fn read(args: &[String]) -> Result<(), MemError> {
    let dump = DumpMem::open(Path::new(&args[0]))?;
    let addr = parse_u64(&args[1])?;
    let len = args.get(2).map(|s| parse_u64(s)).transpose()?.unwrap_or(0x100) as usize;
    let bytes = dump.read_vec(addr, len)?;
    for (i, line) in bytes.chunks(16).enumerate() {
        let hex: Vec<_> = line.iter().map(|b| format!("{b:02x}")).collect();
        let ascii: String = line
            .iter()
            .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
            .collect();
        println!("{:#014x}  {:<47}  {ascii}", addr + i as u64 * 16, hex.join(" "));
    }
    Ok(())
}

fn parse_u64(s: &str) -> Result<u64, MemError> {
    match s.strip_prefix("0x") {
        Some(hex) => u64::from_str_radix(hex, 16),
        None => s.parse(),
    }
    .map_err(|_| bad_arg(format!("not a number: {s}")))
}

fn bad_arg(msg: impl Into<String>) -> MemError {
    MemError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, msg.into()))
}
