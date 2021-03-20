use ash::{Device, Instance , vk::PhysicalDevice};

pub struct MemoryAllocator{
    instance : Instance,
    physical_device : PhysicalDevice,
    device : Device,
}
impl MemoryAllocator{
    pub fn new(instance : &Instance , physical_device : &PhysicalDevice , device : &Device)->Self{
        return Self{
            instance : instance.clone(),
            physical_device : *physical_device,
            device : device.clone(),
        }
    }
}