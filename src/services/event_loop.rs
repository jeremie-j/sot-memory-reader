use std::collections::HashMap;

use std::sync::{Arc, Mutex};

use ggez::event::EventHandler;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};

use crate::core::reader::{ActorInfo, SoTMemoryReader};
use crate::entities::world::{CrewService, World};

pub struct MyGame {
    sot_memory_reader: Arc<Mutex<SoTMemoryReader>>,
    actors_map: HashMap<u32, ActorInfo>,
    player_in_game: bool,
    world: Option<World>,
}

impl MyGame {
    pub fn new(_ctx: &mut Context) -> MyGame {
        Arc::new(SoTMemoryReader::new("SoTGame.exe").unwrap());
        let sot_memory_reader = SoTMemoryReader::new("SoTGame.exe").unwrap();
        let mutex_reader = Mutex::new(sot_memory_reader);
        let arc_reader = Arc::new(mutex_reader);

        MyGame {
            sot_memory_reader: arc_reader.clone(),
            actors_map: HashMap::new(),
            player_in_game: false,
            world: None,
        }
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        print!("\x1B[2J\x1B[1;1H");
        let mut reader = self.sot_memory_reader.lock().unwrap();
        let _ = reader.read_actors(&mut self.actors_map);

        let mut world = World::new();

        for (_actor_id, actor_info) in &self.actors_map {
            if actor_info.raw_name == "CrewService" {
                world.crew_service = Some(CrewService::new(actor_info.clone()));
                continue;
            }
        }
        if world.crew_service.is_none() {
            self.player_in_game = false;
        } else {
            self.world = Some(world);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        print!("\x1B[2J\x1B[1;1H");
        let canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        if *&self.world.is_none() {
            println!("player is not ingame");
        }
        let world = self.world.as_mut().unwrap();
        let crew_service = world.crew_service.as_mut().unwrap();
        crew_service.update();
        crew_service.print_crews();

        // Draw code here...
        canvas.finish(ctx)
    }
}
