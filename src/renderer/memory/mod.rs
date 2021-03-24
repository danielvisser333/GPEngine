mod buffers;
mod images;

use ash::{Device, Instance, vk::{BufferCreateFlags, BufferCreateInfo, BufferUsageFlags, PhysicalDevice, SharingMode, StructureType}};

use self::buffers::VertexBuffer;

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
    pub fn create_vertex_buffer(&self , size : u64 ,)->VertexBuffer{
        let buffer_create_info = BufferCreateInfo{
            s_type : StructureType::BUFFER_CREATE_INFO,
            p_next : std::ptr::null(),
            //Todo: Add the option to use sparse binding.
            flags : BufferCreateFlags::empty(),
            size,
            usage : BufferUsageFlags::VERTEX_BUFFER,
            //Force the usage of memory barriers for transfer.
            sharing_mode : SharingMode::EXCLUSIVE,
            queue_family_index_count : 0,
            p_queue_family_indices : std::ptr::null(),
        };
    }
}