use log::info;

use crate::core::{MemoryReader, SoTMemoryReader};
mod core;

fn main() {
    let memory_reader: MemoryReader = MemoryReader::new("SoTGame.exe").unwrap();
    if let Ok(sot_reader) = SoTMemoryReader::new(memory_reader) {
        let _ = sot_reader.read_actors();
    };

    info!("hello");
    println!("Hello, world!");
}
