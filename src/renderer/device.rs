use ash::{Device, Instance, extensions::khr::{Surface, Swapchain}, version::InstanceV1_0, vk::{DeviceCreateFlags, DeviceCreateInfo, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceType, QueueFlags, StructureType, SurfaceKHR}};

pub unsafe fn get_physical_device(instance : &Instance , surface_loader : &Surface , surface : &SurfaceKHR)->PhysicalDevice{
    let physical_devices = instance.enumerate_physical_devices().expect("Failed to get available GPU's");
    let mut prefered_device = None;
    for &physical_device in physical_devices.iter(){
        let mut graphics = false;
        let mut compute = false;
        let mut presentation = false;
        for (i,&queue_family) in instance.get_physical_device_queue_family_properties(physical_device).iter().enumerate(){
            graphics = graphics || queue_family.queue_flags.contains(QueueFlags::GRAPHICS);
            compute = compute || queue_family.queue_flags.contains(QueueFlags::COMPUTE);
            presentation = presentation || surface_loader.get_physical_device_surface_support(physical_device, i as u32, *surface).expect("Failed to check physical device surface support");
        }
        let device_type = instance.get_physical_device_properties(physical_device).device_type;
        if prefered_device.is_none() && graphics && compute && presentation {
            if device_type == PhysicalDeviceType::DISCRETE_GPU{return physical_device}
            prefered_device = Some(physical_device)
        }
    }
    return prefered_device.unwrap();
}
pub unsafe fn create_device(instance : &Instance , physical_device : &PhysicalDevice)->Device{
    let extensions = [Swapchain::name().as_ptr()];
    let features = PhysicalDeviceFeatures{
        ..Default::default()
    };
    let device_create_info = DeviceCreateInfo{
        s_type : StructureType::DEVICE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DeviceCreateFlags::empty(),
        enabled_layer_count : 0,
        pp_enabled_layer_names : std::ptr::null(),
        enabled_extension_count : extensions.len() as u32,
        pp_enabled_extension_names : extensions.as_ptr(),
        p_enabled_features : &features,
    };
}
pub unsafe fn get_graphics_queue_family(instance : &Instance , physical_device : &PhysicalDevice) -> u32{
    for (i,&queue_family) in instance.get_physical_device_queue_family_properties(*physical_device).iter().enumerate(){
        if queue_family.queue_flags.contains(QueueFlags::GRAPHICS){return i as u32}
    }
    panic!("No graphics operations supported on the GPU");
}