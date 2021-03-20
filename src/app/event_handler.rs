use winit::{event::Event, event_loop::ControlFlow};

use crate::renderer::Renderer;

pub struct EventHandler{

}
impl EventHandler{
    pub fn new()->Self{
        return Self{

        }
    }
    pub fn handle_event(&mut self , event : Event<()> , control_flow : &mut ControlFlow , renderer : &mut Renderer){
        match event{
            Event::NewEvents(start_cause)=>{
                match start_cause{
                    winit::event::StartCause::Init=>{renderer.debug()}
                    _=>{}
                }
            }
            Event::WindowEvent {event,..}=>{
                match event{
                    winit::event::WindowEvent::CloseRequested=>{*control_flow = ControlFlow::Exit}
                    _=>{}
                }
            }
            _=>{}
        }
    }
}