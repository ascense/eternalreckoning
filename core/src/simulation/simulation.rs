use std::time::{ Duration, Instant };
use std::sync::mpsc::Receiver;

use specs::{
    Dispatcher,
    World,
    WorldExt,
};

use super::TickTime;

pub struct Simulation<'a, 'b, T: std::marker::Send + Sync> {
    dispatcher: Dispatcher<'a, 'b>,
    world: World,
    _events: std::marker::PhantomData<T>,
}

impl<'a, 'b, T: 'static + std::marker::Send + Sync> Simulation<'a, 'b, T> {
    pub fn new(mut dispatcher: Dispatcher<'a, 'b>, mut world: World)
        -> Simulation<'a, 'b, T>
    {
        world.insert::<Vec<T>>(Vec::new());
        world.insert(TickTime::default());

        dispatcher.setup(&mut world);

        Simulation {
            dispatcher,
            world,
            _events: std::marker::PhantomData,
        }
    }

    pub fn push_event(&mut self, event: T) {
        let mut queue = self.world.write_resource::<Vec<T>>();
        (*queue).push(event);
    }

    fn clear_events(&mut self) {
        let mut queue = self.world.write_resource::<Vec<T>>();
        (*queue).clear();
    }

    fn set_tick_time(&mut self, time: std::time::Instant) {
        let mut tick_time = self.world.write_resource::<TickTime>();
        tick_time.0 = time;
    }

    pub fn next_tick(&mut self, tick_time: std::time::Instant) {
        self.set_tick_time(tick_time);

        self.dispatcher.dispatch(&mut self.world);
        self.world.maintain();

        self.clear_events();
    }

    pub fn run(&mut self, receiver: Receiver<T>, tick_length: Duration) {
        let mut next_frame = Instant::now();

        loop {
            while Instant::now() < next_frame {
                std::thread::sleep(next_frame - Instant::now());
            }

            loop {
                let event = receiver.try_recv();
                match event {
                    Ok(e) => self.push_event(e),
                    Err(_) => break,
                }
            }

            self.next_tick(next_frame);

            next_frame = next_frame + tick_length;
        }
    }
}