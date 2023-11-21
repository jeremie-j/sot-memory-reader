use std::sync::{Arc, Mutex};

use crate::core::reader::{ActorInfo, SoTMemoryReader};

pub struct World {
    pub sot_memory_reader: Arc<Mutex<SoTMemoryReader>>,
    pub athena_emissary_table: Option<ActorInfo>,
    pub reaper_emissary_table: Option<ActorInfo>,
    pub sovereign_emissary_table: Option<ActorInfo>,
    pub merchant_alliance_emissary_table: Option<ActorInfo>,
    pub gold_hoarders_emissary_table: Option<ActorInfo>,
    pub order_of_souls_emissary_table: Option<ActorInfo>,
}
