use winit::{event::Event, event_loop::ControlFlow};

use crate::renderer::Renderer;

use self::event_handler::EventHandler;

mod event_handler;

pub struct App{
    event_handler : EventHandler,
}
impl App{
    pub fn new()->Self{
        let event_handler = EventHandler::new();
        return Self{
            event_handler,
        }
    }
    pub fn handle_event(&mut self , event : Event<()> , control_flow : &mut ControlFlow , renderer : &mut Renderer){
        self.event_handler.handle_event(event , control_flow , renderer);
    }
}