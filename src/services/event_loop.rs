use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::vec;

use ggez::event::EventHandler;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use std::io::prelude::*;

use crate::core::reader::{ActorInfo, SoTMemoryReader};
use crate::entities::world::{self, World};
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
        let mut reader = self.sot_memory_reader.lock().unwrap();
        reader.read_actors(&mut self.actors_map);

        // for (actor_id, actor_info) in &self.actors_map {
        //     write!(
        //         writer,
        //         "{}, {}, {}\n",
        //         actor_info.id, actor_info.raw_name, actor_info.base_address
        //     )
        //     .expect("Unable to write data");
        // }

        for (actor_id, actor_info) in &self.actors_map {
            actor_info.raw_name.clone();
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

        for table_actor in vec![
            &self.world.gold_hoarders_emissary_table,
            &self.world.gold_hoarders_emissary_table,
            &self.world.gold_hoarders_emissary_table,
            &self.world.gold_hoarders_emissary_table,
            &self.world.gold_hoarders_emissary_table,
            &self.world.reaper_emissary_table,
            &self.world.athena_emissary_table,
        ] {
            if table_actor.is_none() {
                continue;
            }
            let table_actor_info = table_actor.clone().unwrap();
            let a = reader
                .rm
                .read_pointer(table_actor_info.base_address as *mut UObject)
                .unwrap();
            let class_ = reader.rm.read_pointer(a.u_class).unwrap();
            let class_name = reader.rm.read_gname(class_.name.index).unwrap();
            println!("{}", class_name);
            // self.sdk_service.get_offset(attribute_path)
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        // Draw code here...
        canvas.finish(ctx)
    }
}
