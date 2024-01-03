mod core;
mod entities;
mod services;
mod structs;

use ggez::{event, ContextBuilder};
use services::event_loop as my_event_loop;

fn main() {
    let (mut ctx, event_loop) = ContextBuilder::new("sot_reader", "Sot Reader")
        .build()
        .expect("aieee, could not create ggez context!");

    let my_game = my_event_loop::MyGame::new(&mut ctx);

    event::run(ctx, event_loop, my_game);
}
