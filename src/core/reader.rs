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

pub enum MemoryErrors {
    MemoryReadingError(String),
    ByteToStringConversion,
}

pub struct MemoryReader {
    process: Process<'static>,
    module: Module,
    u_world_base: usize,
    g_object_base: usize,
    g_name_base: usize,
    g_name_start_address: u8,
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
            module.base_address + g_name_base + 7 as usize,
            size_of::<u32>(),
            &mut g_name_offset as *mut u32,
        )
        .expect("Could not get g_name_offset offset");

        let g_name_ptr: usize = module.base_address + g_name_base + (g_name_offset + 7) as usize;
        let mut g_name_start_address: u8 = 0;
        read::<u8>(
            &process.handle,
            g_name_ptr,
            size_of::<u8>(),
            &mut g_name_start_address as *mut u8,
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

    pub fn read_string_default_size(&self, address: usize) -> Result<String, MemoryErrors> {
        self.read_string(address, 50)
    }

    pub fn read_string(&self, address: usize, size: usize) -> Result<String, MemoryErrors> {
        let mut target_buffer: Vec<u8> = vec![];
        read::<Vec<u8>>(
            &self.process.handle,
            address,
            size,
            &mut target_buffer as *mut Vec<u8>,
        )
        .map_err(|err| {
            MemoryErrors::MemoryReadingError(format!("Could not read string at {:#X}", address))
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

    pub fn read_name_string(&self, address: usize, size: usize) -> Result<String, MemoryErrors> {
        let mut target_buffer: Vec<u8> = vec![];
        read::<Vec<u8>>(
            &self.process.handle,
            address,
            size,
            &mut target_buffer as *mut Vec<u8>,
        )
        .map_err(|err| {
            MemoryErrors::MemoryReadingError(format!("Could not read string at {:#X}", address))
        })?;

        let i = target_buffer
            .windows(3)
            .position(|window| window == b"\x00\x00\x00")
            .ok_or(MemoryErrors::ByteToStringConversion)?;

        let u16_buffer: Vec<u16> = target_buffer[0..i]
            .chunks_exact(2)
            .into_iter()
            .map(|a| u16::from_ne_bytes([a[0], a[1]]))
            .collect();

        match String::from_utf16(&u16_buffer) {
            Ok(v) => Ok(v),
            Err(_) => Err(MemoryErrors::ByteToStringConversion),
        }
    }

    pub fn read_gname(&self, actor_id: u8) -> Result<String, MemoryErrors> {
        let actor_id: u32 = u32::from(actor_id);
        let g_name_start_address = u32::from(self.g_name_start_address);
        let name_ptr: u32 = self
            .read_ptr((g_name_start_address + actor_id / 0x4000 * 0x8) as usize)?
            .into();
        let name = self.read_ptr((name_ptr + 0x8 * actor_id % 0x4000) as usize)?;
        Ok(self.read_string((name + 0x10) as usize, 64))?
    }

    pub fn read_ptr(&self, address: usize) -> Result<u8, MemoryErrors> {
        let mut target_buffer: u8 = 0;
        read::<u8>(
            &self.process.handle,
            address,
            size_of::<u8>(),
            &mut target_buffer as *mut u8,
        )
        .map_err(|err| {
            MemoryErrors::MemoryReadingError(format!("Could not read pointer at {:#X}", address))
        })?;
        Ok(target_buffer)
    }

    pub fn read_actors(&self){
        let u_world_offset: u8 = 0;
        read(&self.process.handle, )
        
        
        .read_ulong(
            base_address + self.rm.u_world_base + 3
        )
        u_world = base_address + self.rm.u_world_base + u_world_offset + 7
        self.world_address = self.rm.read_ptr(u_world)


        let actor_base_address = read<u8>(
            &self.process.handle,
            self.u_world_base
        )
        let actors_length = 


        // actor_raw = self.rm.read_bytes(self.u_level + 0xa0, 0xC)
        // actor_data = struct.unpack("<Qi", actor_raw)

        // raw_name = ""
        // actor_address = self.rm.read_ptr(actor_data[0] + (x * 0x8))
        // actor_id = self.rm.read_int(
        //     actor_address + OFFSETS.get('Actor.actorId')
        // )

        // # We save a mapping of actor id to actor name for the sake of
        // # saving memory calls
        // if actor_id not in self.actor_name_map and actor_id != 0:
        //     try:
        //         raw_name = self.rm.read_gname(actor_id)
        //         self.actor_name_map[actor_id] = raw_name
        //     except Exception as e:
        //         logger.error(f"Unable to find actor name: {e}")
        // elif actor_id in self.actor_name_map:
        //     raw_name = self.actor_name_map.get(actor_id)
            

        // actor_raw = self.rm.read_bytes(self.u_level + 0xa0, 0xC)
        // actor_data = struct.unpack("<Qi", actor_raw)
    }


    pub fn read_address<T>(&self, address: usize) -> Result<T, MemoryErrors> {
        let mut target_buffer: T;
        read::<T>(
            &self.process.handle,
            address,
            size_of::<T>(),
            &mut target_buffer as *mut T
        ).map_err(|err| MemoryErrors::MemoryReadingError(format!("Could not read {} type at {:#X}", type_name::<T>(), address)))?;
        Ok(target_buffer)
    }
}
