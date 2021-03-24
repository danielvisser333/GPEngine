mod buffers;
mod images;

use ash::{Device, Instance, version::DeviceV1_0, vk::{BufferCreateFlags, BufferCreateInfo, BufferUsageFlags, DeviceMemory, PhysicalDevice, SharingMode, StructureType}};

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
    pub unsafe fn create_vertex_buffer(&self , vertex_count : u32 , vertex_size : u32 , device_local : bool)->VertexBuffer{
        let buffer_create_info = BufferCreateInfo{
            s_type : StructureType::BUFFER_CREATE_INFO,
            p_next : std::ptr::null(),
            //Todo: Add the option to use sparse binding.
            flags : BufferCreateFlags::empty(),
            size : (vertex_count * vertex_size) as u64,
            usage : BufferUsageFlags::VERTEX_BUFFER,
            //Force the usage of memory barriers for transfer.
            sharing_mode : SharingMode::EXCLUSIVE,
            queue_family_index_count : 0,
            p_queue_family_indices : std::ptr::null(),
        };
        let vertex_buffer = self.device.create_buffer(&&buffer_create_info, None).expect("Memory allocation error");
        
        return VertexBuffer{
            vertex_count , vertex_size , offset : u32::MAX , allocation : DeviceMemory::null() ,
        }
    }
}