use std::ffi::CString;

use ash::{Entry, Instance, extensions::ext::DebugUtils, version::EntryV1_0, vk::{ApplicationInfo, InstanceCreateFlags, InstanceCreateInfo, StructureType}};
use winit::window::Window;

pub unsafe fn create_instance(entry : &Entry , window : &Window , validation : bool) -> Instance{
    let app_name = CString::new("GPEngine").unwrap();
    let api_version = match entry.try_enumerate_instance_version().expect("Failed to get Vulkan driver version"){
        Some(v)=>{v}
        None=>{ash::vk::make_version(1, 0, 0)}
    };
    let application_info = ApplicationInfo{
        s_type : StructureType::APPLICATION_INFO,
        p_next : std::ptr::null(),
        engine_version : 1,
        application_version : 1,
        p_engine_name : app_name.as_ptr(),
        p_application_name : app_name.as_ptr(),
        api_version,
    };
    let mut surface_extensions = ash_window::enumerate_required_extensions(window).expect("Failed to get required surface extensions");
    if validation {surface_extensions.push(DebugUtils::name())}
    let instance_extensions = surface_extensions.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();
    let instance_layers = if validation {vec!(CString::new("VK_LAYER_KHRONOS_validation").unwrap())}else{vec!()};
    let instance_layer_names = instance_layers.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();
    let instance_create_info = InstanceCreateInfo{
        s_type : StructureType::INSTANCE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : InstanceCreateFlags::empty(),
        p_application_info : &application_info,
        enabled_extension_count : surface_extensions.len() as u32,
        pp_enabled_extension_names : instance_extensions.as_ptr(),
        enabled_layer_count : instance_layers.len() as u32,
        pp_enabled_layer_names : instance_layer_names.as_ptr(),
    };
    return entry.create_instance(&instance_create_info, None).expect("Failed to create Vulkan instance, is validation enabled?");
}