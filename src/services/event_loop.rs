use std::collections::HashMap;
use std::ffi::c_void;

use std::sync::{Arc, Mutex};
use std::vec;

use ggez::event::EventHandler;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};

use crate::core::reader::{find_dma_addy, read_pointer, ActorInfo, SoTMemoryReader};
use crate::entities::world::World;
use crate::structs::unreal::UObject;

use super::sdk::SdkService;

pub struct MyGame {
    sot_memory_reader: Arc<Mutex<SoTMemoryReader>>,
    sdk_service: SdkService,
    actors_map: HashMap<u32, ActorInfo>,
    world: World,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        Arc::new(SoTMemoryReader::new("SoTGame.exe").unwrap());
        let sot_memory_reader = SoTMemoryReader::new("SoTGame.exe").unwrap();
        let mutex_reader = Mutex::new(sot_memory_reader);
        let arc_reader = Arc::new(mutex_reader);

        let mut sdk_service = SdkService::new();
        sdk_service.scan_sdk();

        MyGame {
            sot_memory_reader: arc_reader.clone(),
            sdk_service,
            actors_map: HashMap::new(),
            world: World {
                sot_memory_reader: arc_reader.clone(),
                athena_emissary_table: None,
                reaper_emissary_table: None,
                sovereign_emissary_table: None,
                merchant_alliance_emissary_table: None,
                gold_hoarders_emissary_table: None,
                order_of_souls_emissary_table: None,
            },
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        print!("\x1B[2J\x1B[1;1H");
        let mut reader = self.sot_memory_reader.lock().unwrap();
        let _ = reader.read_actors(&mut self.actors_map);

        for (_actor_id, actor_info) in &self.actors_map {
            match actor_info.raw_name.as_str() {
                "BP_EmissaryTable_GoldHoarders_01" => {
                    self.world.gold_hoarders_emissary_table = Some(actor_info.clone())
                }
                "BP_EmissaryTable_MerchantAlliance_01" => {
                    self.world.merchant_alliance_emissary_table = Some(actor_info.clone())
                }
                "BP_EmissaryTable_OrderOfSouls_01" => {
                    self.world.order_of_souls_emissary_table = Some(actor_info.clone())
                }
                "BP_EmissaryTable_Sov_01_a_C" => {
                    self.world.sovereign_emissary_table = Some(actor_info.clone())
                }
                "BP_FactionEmissaryTable_Reapers2" => {
                    self.world.reaper_emissary_table = Some(actor_info.clone())
                }
                "BP_FactionEmissaryTable_Athena" => {
                    self.world.athena_emissary_table = Some(actor_info.clone())
                }
                _ => continue,
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        let mut reader = self.sot_memory_reader.lock().unwrap();
        for (table_actor, emissary_label) in vec![
            (&self.world.gold_hoarders_emissary_table, "Gold Hoarders : "),
            (&self.world.merchant_alliance_emissary_table, "Merchants : "),
            (
                &self.world.order_of_souls_emissary_table,
                "Order of souls : ",
            ),
            (&self.world.sovereign_emissary_table, "Sovereign : "),
            (&self.world.reaper_emissary_table, "Reaper : "),
            (&self.world.athena_emissary_table, "Athena : "),
        ] {
            if table_actor.is_none() {
                continue;
            }
            let table_actor_info = table_actor.clone().unwrap();
            let u_object = read_pointer(table_actor_info.base_address as *mut UObject).unwrap();

            let class_ = read_pointer(u_object.u_class).unwrap();
            let class_name = reader.rm.read_gname(class_.name.index).unwrap();

            let emissary_ship_affiliation_tracker_offset = self
                .sdk_service
                .get_offset(&format!("{}.EmissaryShipAffiliationTracker", class_name));

            let emissary_count_offset = self
                .sdk_service
                .get_offset("EmissaryShipAffiliationTrackerComponent.EmissaryCount");

            let emmisary_count = find_dma_addy::<u32>(
                table_actor_info.base_address,
                vec![
                    emissary_ship_affiliation_tracker_offset,
                    emissary_count_offset,
                ],
            )
            .unwrap();

            println!("{} {}", emissary_label, emmisary_count);
        }

        // Draw code here...
        canvas.finish(ctx)
    }
}
