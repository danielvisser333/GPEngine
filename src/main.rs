mod renderer;
mod app;

use winit::{event_loop::EventLoop, window::Window};

fn main() {
    println!("Starting Program");
    println!("Creating Event Loop");
    let event_loop = EventLoop::new();
    println!("Creating Window");
    let window = Window::new(&event_loop).expect("Failed to create window");
    println!("Creating Vulkan Renderer");
    let mut renderer = renderer::Renderer::new(&window);
    println!("Creating global state");
    let mut app = app::App::new();
    println!("Starting Event Loop");
    event_loop.run(move |event,_,control_flow|{
        app.handle_event(event,control_flow,&mut renderer);
    });
}
