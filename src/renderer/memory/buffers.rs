use ash::vk::DeviceMemory;

pub struct VertexBuffer{
    vertex_count : u32,
    vertex_size : u32,
    offset : u32,
    allocation : DeviceMemory,
}
pub struct IndexBuffer{
    
}