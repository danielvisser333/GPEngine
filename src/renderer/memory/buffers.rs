use ash::vk::DeviceMemory;

pub struct VertexBuffer{
    pub vertex_count : u32,
    pub vertex_size : u32,
    pub offset : u32,
    pub allocation : DeviceMemory,
}
pub struct IndexBuffer{
    
}