mod instance;
mod device;

use std::ffi::CStr;

use ash::{Device, Entry, Instance, extensions::khr::Surface, version::{DeviceV1_0, InstanceV1_0}, vk::{PhysicalDevice, SurfaceKHR}};
use winit::window::Window;

pub struct Renderer{
    entry : Entry,
    instance : Instance,
    surface_loader : Surface,
    surface : SurfaceKHR,
    gpu : PhysicalDevice,
    device : Device,
}
impl Renderer{
    pub fn new(window : &Window)->Self{
        let entry = unsafe{Entry::new()}.expect("Failed to create Vulkan entry");
        let instance = unsafe{instance::create_instance(&entry,window,true)};
        let surface_loader = Surface::new(&entry, &instance);
        let surface = unsafe{ash_window::create_surface(&entry, &instance, window, None)}.expect("Failed to create Vulkan surface");
        let gpu = unsafe{device::get_physical_device(&instance, &surface_loader, &surface)};
        let device = unsafe{device::create_device(&instance, &gpu, &surface_loader, &surface)};
        return Self{
            entry , instance , surface_loader , surface , gpu , device ,
        }
    }
    pub fn debug(&self){
        let vulkan_version = match self.entry.try_enumerate_instance_version().expect("Failed to get Vulkan version"){
            Some(v)=>{(ash::vk::version_major(v),ash::vk::version_minor(v),ash::vk::version_patch(v))}
            None=>{(1,0,0)}
        };
        let device_name = {
            let array = unsafe{self.instance.get_physical_device_properties(self.gpu)}.device_name;
            let cstring = unsafe{CStr::from_ptr(array.as_ptr())};
            cstring.to_str().unwrap().to_owned()
        };
        println!("Renderer Info:\n");
        println!("Using Vulkan version : {}.{}.{}.",vulkan_version.0,vulkan_version.1,vulkan_version.2);
        println!("Using GPU : {}.\n",device_name);
    }
}
impl Drop for Renderer{
    fn drop(&mut self) {
        unsafe{
            self.device.device_wait_idle().expect("Failed to properly shutdown renderer");
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}