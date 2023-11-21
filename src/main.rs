mod core;
mod entities;
mod services;

use core::reader::SoTMemoryReader;
use services::sdk::SdkService;

fn main() {
    let mut sot_memory_reader = SoTMemoryReader::new("SoTGame.exe").unwrap();
    sot_memory_reader.read_actors();
    let mut sdk_service = SdkService::new();
    sdk_service.scan_sdk();
}
