use std::any::type_name;
use std::str::{from_utf8, Utf8Error};
use std::{mem::size_of, str::Bytes};

use toy_arms::external::module::Module;
use toy_arms::external::process::Process;
use toy_arms::external::read;

use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

const UWORLDPATTERN: &'static str = "48 8B 05 ? ? ? ? 48 8B 88 ? ? ? ? 48 85 C9 74 06 48 8B 49 70";
const GOBJECTPATTERN: &'static str = "89 0D ? ? ? ? 48 8B DF 48 89 5C 24";
const GNAMEPATTERN: &'static str = "48 8B 1D ? ? ? ? 48 85 DB 75 ? B9 08 04 00 00";

pub enum MemoryReaderError {
    MemoryReadingError(String),
    ByteToStringConversion,
}

pub struct MemoryReader {
    process: Process<'static>,
    module: Module,
    u_world_base: usize,
    g_object_base: usize,
    g_name_base: usize,
    g_name_start_address: u64,
}

impl MemoryReader {
    pub fn new(process_name: &'static str) -> Result<Self, &'static str> {
        let process = Process::from_process_name(process_name).expect("Process could be found");
        let mut module = process.get_module_info(process_name).unwrap();

        let u_world_base = module
            .find_pattern(UWORLDPATTERN)
            .expect("Could not find u_world_base offsets");
        let g_object_base = module
            .find_pattern(GOBJECTPATTERN)
            .expect("Could not find g_object_base offsets");
        let g_name_base = module
            .find_pattern(GNAMEPATTERN)
            .expect("Could not find g_name_base offsets");

        let mut g_name_offset: u32 = 0;
        read::<u32>(
            &process.handle,
            module.base_address + g_name_base + 3 as usize,
            size_of::<u32>(),
            &mut g_name_offset as *mut u32,
        )
        .expect("Could not get g_name_offset offset");

        let g_name_ptr: usize = module.base_address + g_name_base + (g_name_offset + 7) as usize;
        let mut g_name_start_address: u64 = 0;
        read::<u64>(
            &process.handle,
            g_name_ptr,
            size_of::<u64>(),
            &mut g_name_start_address as *mut u64,
        )
        .expect("Could not get g_name_start_address offset");

        Ok(Self {
            process,
            module,
            u_world_base,
            g_object_base,
            g_name_base,
            g_name_start_address,
        })
    }

    pub fn check_process_is_active(&self) -> bool {
        let s = System::new_all();
        for process in s.processes_by_exact_name(self.process.name) {
            if process.pid().as_u32() == self.process.id {
                return true;
            }
        }
        false
    }

    pub fn read_string_default_size(&self, address: usize) -> Result<String, MemoryReaderError> {
        self.read_string(address, 50)
    }

    pub fn read_string(&self, address: usize, size: usize) -> Result<String, MemoryReaderError> {
        let mut target_buffer: Vec<u8> = vec![];
        read::<Vec<u8>>(
            &self.process.handle,
            address,
            size,
            &mut target_buffer as *mut Vec<u8>,
        )
        .map_err(|err| {
            MemoryReaderError::MemoryReadingError(format!(
                "Could not read string at {:#X}",
                address
            ))
        })?;
        let i = match target_buffer.iter().position(|r| r == &b'\x00') {
            Some(v) => v,
            None => target_buffer.len(),
        };
        let result = from_utf8(&target_buffer[0..i]);

        match result {
            Ok(v) => Ok(String::from(v)),
            Err(_) => Ok(self.read_name_string(address, size)?),
        }
    }

    pub fn read_name_string(
        &self,
        address: usize,
        size: usize,
    ) -> Result<String, MemoryReaderError> {
        let mut target_buffer: Vec<u8> = vec![];
        read::<Vec<u8>>(
            &self.process.handle,
            address,
            size,
            &mut target_buffer as *mut Vec<u8>,
        )
        .map_err(|err| {
            MemoryReaderError::MemoryReadingError(format!(
                "Could not read string at {:#X}",
                address
            ))
        })?;

        let i = target_buffer
            .windows(3)
            .position(|window| window == b"\x00\x00\x00")
            .ok_or(MemoryReaderError::ByteToStringConversion)?;

        let u16_buffer: Vec<u16> = target_buffer[0..i]
            .chunks_exact(2)
            .into_iter()
            .map(|a| u16::from_ne_bytes([a[0], a[1]]))
            .collect();

        match String::from_utf16(&u16_buffer) {
            Ok(v) => Ok(v),
            Err(_) => Err(MemoryReaderError::ByteToStringConversion),
        }
    }

    pub fn read_gname(&self, actor_id: u32) -> Result<String, MemoryReaderError> {
        let actor_id = u64::from(actor_id);
        let name_ptr = self
            .read_address::<u64>((self.g_name_start_address + actor_id / 0x4000 * 0x8) as usize)?;
        let name = self.read_address::<u64>((name_ptr + 0x8 * actor_id % 0x4000) as usize)?;
        Ok(self.read_string((name + 0x10) as usize, 64))?
    }

    pub fn read_address<T: Default>(&self, address: usize) -> Result<T, MemoryReaderError> {
        let mut target_buffer = T::default();
        read::<T>(
            &self.process.handle,
            address,
            size_of::<T>(),
            &mut target_buffer as *mut T,
        )
        .map_err(|err| {
            MemoryReaderError::MemoryReadingError(format!(
                "Could not read {} type at {:#X}",
                type_name::<T>(),
                address
            ))
        })?;
        Ok(target_buffer)
    }

    pub fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>, MemoryReaderError> {
        let mut target_buffer: Vec<u8> = vec![];
        read::<Vec<u8>>(
            &self.process.handle,
            address,
            size,
            &mut target_buffer as *mut Vec<u8>,
        )
        .map_err(|err| {
            MemoryReaderError::MemoryReadingError(format!("Could not read bytes at {:#X}", address))
        })?;
        Ok((target_buffer))
    }
}

pub struct SoTMemoryReader {
    rm: MemoryReader,
    world_address: usize,
    u_level: usize,
}

impl SoTMemoryReader {
    pub fn new(rm: MemoryReader) -> Result<Self, MemoryReaderError> {
        let base_address = rm.module.base_address;

        let u_world_offset = rm.read_address::<u32>(base_address + rm.u_world_base + 3)? as usize;
        let u_world_ptr = base_address + rm.u_world_base + u_world_offset + 7;
        let world_address = rm.read_address::<u64>(u_world_ptr)? as usize;
        let g_objects_offset =
            rm.read_address::<u64>(base_address + rm.g_object_base + 2)? as usize;
        let g_objects_address = base_address + rm.g_object_base + g_objects_offset + 22;

        let u_level = rm.read_address::<u64>(world_address + 0x30)? as usize;

        Ok(Self {
            rm,
            world_address,
            u_level,
        })
    }

    pub fn read_actors(&self) -> Result<(), MemoryReaderError> {
        let actor_base = self.rm.read_address::<u64>(self.u_level + 0xa0)?;
        let actor_array_size = self.rm.read_address::<u32>(self.u_level + 0xa0 + 8)? as usize;

        // Credit @mogistink https://www.unknowncheats.me/forum/members/3434160.html
        let level_actors_raw: Vec<u8> = self
            .rm
            .read_bytes(actor_base as usize, actor_array_size as usize * 8)?;

        for i in 0..actor_array_size {
            let slice = &level_actors_raw[(i * 8)..(i * 8 + 8)];

            let mut raw_actor_address = [0u8; 8];
            raw_actor_address.copy_from_slice(slice);
            let actor_address = usize::from_le_bytes(raw_actor_address);

            let actor_id = self.rm.read_address::<u32>(actor_address + 0x18)?;
            if actor_id != 0 {
                let name = match self.rm.read_gname(actor_id) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                println!("{} at {:#X}", name, actor_address)
            }
        }

        Ok(())
    }
}
