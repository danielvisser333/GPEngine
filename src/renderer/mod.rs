use std::{ffi::{CStr, CString}, u32};

use ash::{Device, Entry, Instance, InstanceError, extensions::khr::{Surface, Swapchain}, version::{DeviceV1_0, EntryV1_0, InstanceV1_0}, vk::{AccessFlags, ApplicationInfo, AttachmentDescription, AttachmentDescriptionFlags, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, ColorSpaceKHR, ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, DependencyFlags, DeviceCreateFlags, DeviceCreateInfo, DeviceMemory, DeviceQueueCreateFlags, DeviceQueueCreateInfo, Extent2D, Extent3D, Format, FormatFeatureFlags, Framebuffer, FramebufferCreateFlags, FramebufferCreateInfo, Image, ImageAspectFlags, ImageCreateFlags, ImageCreateInfo, ImageLayout, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags, ImageView, ImageViewCreateFlags, ImageViewCreateInfo, ImageViewType, InstanceCreateFlags, InstanceCreateInfo, MemoryAllocateInfo, MemoryPropertyFlags, MemoryRequirements, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties, PhysicalDeviceType, PipelineBindPoint, PipelineStageFlags, PresentModeKHR, QueueFamilyProperties, QueueFlags, RenderPass, RenderPassCreateFlags, RenderPassCreateInfo, SUBPASS_EXTERNAL, SampleCountFlags, SharingMode, StructureType, SubpassDependency, SubpassDescription, SubpassDescriptionFlags, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, SwapchainCreateFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR}};
use log::{error,info,warn,debug,trace};
use winit::window::Window;

pub struct Renderer{
    _entry : Entry,
    instance : Instance,
    surface_loader : Surface,
    surface : SurfaceKHR,
    _physical_device : PhysicalDevice,
    device : Device,
    swapchain_loader : Swapchain,
    swapchain : SwapchainKHR,
    _swapchain_images : Vec<Image>,
    swapchain_image_views : Vec<ImageView>,
    depth_image : Image,
    depth_image_memory : DeviceMemory,
    depth_image_view : ImageView,
    render_pass : RenderPass,
    framebuffers : Vec<Framebuffer>,
}
impl Renderer {
    pub fn new(window : &Window)->Self{
        let entry = unsafe{Entry::new()}.unwrap_or_else(|e|{error!("Failed to load Vulkan library:{}.",e);panic!("Failed to load Vulkan library.")});
        let vulkan_version = match entry.try_enumerate_instance_version().unwrap_or_else(|e|{error!("Generic error:{}.",e);panic!("Something went wrong whith the Vulkan library")}){
            Some(v)=>{(ash::vk::version_major(v),ash::vk::version_minor(v),ash::vk::version_patch(v))}
            None=>{(1,0,0)}
        };
        info!("Successfully loaded Vulkan library, version:{}.{}.{}.",vulkan_version.0,vulkan_version.1,vulkan_version.2);
        let instance = unsafe{create_instance(&entry, window).unwrap_or_else(|e|{
            error!("Failed to create Vulkan instance, {}.",e);
            panic!()})
        };
        info!("Created Vulkan instance.");
        let surface_loader = Surface::new(&entry , &instance);
        let surface = unsafe{ash_window::create_surface(&entry, &instance, window, None)}.unwrap_or_else(|e|{
            error!("Failed to create Vulkan surface, {}.",e);
            panic!()}
        );
        info!("Succesfully created Vulkan surface.");
        let (physical_device,physical_device_name) = unsafe{get_physical_device(&instance, &surface_loader, &surface)};
        info!("Selected {} as GPU for rendering",physical_device_name);
        let device = unsafe{create_device(&instance, &physical_device , &surface_loader , &surface)};
        info!("Created Vulkan device handle");
        let queue_family_properties = unsafe{instance.get_physical_device_queue_family_properties(physical_device)};
        let graphics_queue = get_graphics_queue_family(&queue_family_properties);
        let compute_queue_family = get_compute_queue_family(&queue_family_properties);
        let presentation_queue = if unsafe{check_queue_family_presentation_support(&surface_loader, &surface, &physical_device, graphics_queue)}{graphics_queue}
        else if unsafe{check_queue_family_presentation_support(&surface_loader, &surface, &physical_device, compute_queue_family)}{compute_queue_family}
        else{error!("Failed to get presentation queue family.");panic!()};
        let swapchain_loader = Swapchain::new(&instance, &device);
        let swapchain = unsafe{create_swapchain(&physical_device, &swapchain_loader, &surface_loader, &surface, graphics_queue, presentation_queue, window)};
        let swapchain_images = unsafe{swapchain_loader.get_swapchain_images(swapchain)}.unwrap_or_else(|e|{
            error!("Failed to acquire swapchain images, {}.",e);
            panic!();
        });
        let swapchain_format = unsafe{get_surface_format(&surface_loader, &surface, &physical_device)};
        let capabilities = unsafe{surface_loader.get_physical_device_surface_capabilities(physical_device, surface)}.unwrap_or_else(|e|{
            error!("Failed to get surface capabilities, {}.",e);
            panic!();
        });
        let swapchain_extent = unsafe{get_surface_extent(&capabilities, window)};
        let swapchain_image_views = unsafe{create_swapchain_image_views(&device, &swapchain_images, &swapchain_format.format)};
        info!("Created Vulkan swapchain.");
        let (depth_image_format,depth_image_tiling) = unsafe{get_depth_image_format_and_tiling(&instance, &physical_device)};
        let (depth_image,depth_image_memory,depth_image_view) = unsafe{create_depth_images_and_view(&device, &depth_image_format, &depth_image_tiling, &swapchain_extent, &instance, &physical_device)};
        info!("Created depth buffer.");
        let render_pass = unsafe{create_render_pass(&device , &swapchain_format.format , &depth_image_format)};
        info!("Created Render Pass");
        let framebuffers = unsafe{create_framebuffers(&device, &render_pass, &swapchain_extent, &swapchain_image_views, &depth_image_view)};
        
        return Self{
            _entry : entry , instance , surface_loader , surface , _physical_device : physical_device , device , swapchain_loader , swapchain , 
            _swapchain_images : swapchain_images , swapchain_image_views , depth_image , depth_image_memory , depth_image_view , render_pass , framebuffers ,
        }
    }
}
impl Drop for Renderer{
    fn drop(&mut self){
        unsafe{
            debug!("Destroying framebuffers.");
            for (i,&framebuffer) in self.framebuffers.iter().enumerate(){
                trace!("Destroying framebuffer {}.",i);
                self.device.destroy_framebuffer(framebuffer, None);
            }
            debug!("Destroying render pass.");
            self.device.destroy_render_pass(self.render_pass, None);
            debug!("Destroying depth image.");
            trace!("Destroying depth image view.");
            self.device.destroy_image_view(self.depth_image_view, None);
            trace!("Freeing depth image memory.");
            self.device.free_memory(self.depth_image_memory, None);
            trace!("Destroying depth image.");
            self.device.destroy_image(self.depth_image, None);
            debug!("Destroying Swapchain.");
            for (i,&image_view) in self.swapchain_image_views.iter().enumerate(){
                trace!("Destroying swapchain image view {}.",i);
                self.device.destroy_image_view(image_view, None);
            }
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            debug!("Destroying Device.");
            self.device.destroy_device(None);
            debug!("Destroying Surface.");
            self.surface_loader.destroy_surface(self.surface, None);
            debug!("Destroying Instance.");
            self.instance.destroy_instance(None);
        }
    }
}
unsafe fn create_instance(entry : &Entry , window : &Window) -> Result<Instance,InstanceError>{
    let api_version = match entry.try_enumerate_instance_version().unwrap(){Some(v)=>{v}None=>{ash::vk::make_version(1, 0, 0)}};
    let name = CString::new("gpengine").unwrap();
    let app_info = ApplicationInfo{
        s_type : StructureType::APPLICATION_INFO,
        p_next : std::ptr::null(),
        api_version,
        engine_version : 0,
        application_version : 0,
        p_engine_name : name.as_ptr(),
        p_application_name : name.as_ptr(),
    };
    let window_extensions = ash_window::enumerate_required_extensions(window).unwrap_or_else(|e|{error!("Failed to acquire Vulkan surface extensions, {}.",e);panic!("Failed to get surface extensions.")});
    let extensions = window_extensions.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();
    let mut validation = false;
    for arg in std::env::args(){
        if arg == "--vkdebug"{validation = true; warn!("Vulkan validation is enabled, performance is degraded significantly!")}
    }
    let enabled_layers = if validation{
        vec!(CString::new("VK_LAYER_KHRONOS_validation").unwrap())
    }   else{
        vec!()
    };
    let layers = enabled_layers.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();
    let instance_create_info = InstanceCreateInfo{
        s_type : StructureType::INSTANCE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : InstanceCreateFlags::empty(),
        p_application_info : &app_info,
        enabled_extension_count : extensions.len() as u32,
        pp_enabled_extension_names : extensions.as_ptr(),
        enabled_layer_count : enabled_layers.len() as u32,
        pp_enabled_layer_names : layers.as_ptr(),
    };
    return entry.create_instance(&instance_create_info, None);
}
unsafe fn get_physical_device(instance : &Instance , surface_loader : &Surface , surface : &SurfaceKHR)->(PhysicalDevice,String){
    let supported_gpus = get_supported_physical_devices(instance, surface_loader, surface);
    let mut prefered_gpu = None;
    for (physical_device,name) in supported_gpus.iter(){
        let device_properties = instance.get_physical_device_properties(*physical_device);
        if prefered_gpu.is_none(){prefered_gpu = Some((*physical_device,name.clone().to_string()))}
        else if device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU{return (*physical_device,name.clone().to_string())}
    }
    return prefered_gpu.unwrap_or_else(||{error!("No supported GPU's found.");panic!("No supported GPU's found")});
}
unsafe fn get_supported_physical_devices(instance : &Instance , surface_loader : &Surface , surface : &SurfaceKHR)->Vec<(PhysicalDevice,String)>{
    let physical_devices = instance.enumerate_physical_devices().unwrap_or_else(|e|{error!("Failed to get supported devices, {}.",e);panic!("Failed to get supported GPU's")});
    let mut supported_devices = vec!();
    for &physical_device in physical_devices.iter(){
        let device_properties = instance.get_physical_device_properties(physical_device);
        let gpu_name = CStr::from_ptr(device_properties.device_name.as_ptr()).to_str().to_owned().unwrap();
        trace!("Found GPU :{} of type:{:?}.",gpu_name,device_properties.device_type);
        let device_queue_family_properties = instance.get_physical_device_queue_family_properties(physical_device);
        let mut supports_graphics = false;
        let mut supports_compute = false;
        let mut supports_presentation = false;
        for (i,&queue_family) in device_queue_family_properties.iter().enumerate(){
            supports_graphics = supports_graphics || queue_family.queue_flags.contains(QueueFlags::GRAPHICS);
            supports_compute = supports_compute || queue_family.queue_flags.contains(QueueFlags::COMPUTE);
            supports_presentation = supports_presentation || surface_loader.get_physical_device_surface_support(physical_device, i as u32, *surface).unwrap_or_else(|e|{error!("Failed to check GPU surface support, {}.",e);panic!("Failed to check GPU surface support.")});
        }
        if supports_graphics && supports_compute && supports_presentation {
            trace!("GPU is compatible :{}.",gpu_name);
            supported_devices.push((physical_device,gpu_name.to_string()));
        }
    }
    return supported_devices;
}
fn get_graphics_queue_family(queue_family_properties : &Vec<QueueFamilyProperties>)->u32{
    for (i,queue_family) in queue_family_properties.iter().enumerate(){
        if queue_family.queue_flags.contains(QueueFlags::GRAPHICS){return i as u32}
    }
    error!("No supported graphics queue found.");
    panic!("No supported graphics queue found.");
}
fn get_compute_queue_family(queue_family_properties : &Vec<QueueFamilyProperties>)->u32{
    let mut fallback_family = None;
    for (i,queue_family) in queue_family_properties.iter().enumerate(){
        if queue_family.queue_flags.contains(QueueFlags::COMPUTE) && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS){return i as u32}
        else if queue_family.queue_flags.contains(QueueFlags::COMPUTE) && fallback_family.is_none(){fallback_family = Some(i as u32)}
    }
    return fallback_family.unwrap_or_else(||{
        error!("No queue family that supports compute operations found");
        panic!();
    });
}
///Gets the asynchronous transfer queue family.
///Not all GPU's have a dedicated transfer queue family.
fn get_dma_queue_family(queue_family_properties : &Vec<QueueFamilyProperties>)->Option<u32>{
    let dma_queue_family = None;
    for (i,queue_family) in queue_family_properties.iter().enumerate(){
        if queue_family.queue_flags.contains(QueueFlags::TRANSFER) && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS) && !queue_family.queue_flags.contains(QueueFlags::COMPUTE){return Some(i as u32)}
    }
    return dma_queue_family;
}
unsafe fn check_queue_family_presentation_support(surface_loader : &Surface , surface : &SurfaceKHR , physical_device : &PhysicalDevice , index : u32)->bool{
    return surface_loader.get_physical_device_surface_support(*physical_device, index, *surface).unwrap_or_else(|e|{
        error!("Failed to check device presentation support, {}.",e);
        panic!();
    });
}
unsafe fn create_device(instance : &Instance , physical_device : &PhysicalDevice , surface_loader : &Surface , surface : &SurfaceKHR)->Device{
    let device_features = PhysicalDeviceFeatures{
        ..Default::default()
    };
    let queue_family_properties = instance.get_physical_device_queue_family_properties(*physical_device);
    let graphics_queue_family = get_graphics_queue_family(&queue_family_properties);
    let compute_queue_family = get_compute_queue_family(&queue_family_properties);
    let dma_queue_family = get_dma_queue_family(&queue_family_properties);
    if !check_queue_family_presentation_support(surface_loader, surface, physical_device, graphics_queue_family) &&
    !check_queue_family_presentation_support(surface_loader, surface, physical_device, compute_queue_family){
        error!("Nor the graphics or the compute family support presentation capabilities.");
        panic!();
    }
    let device_extensions = [Swapchain::name().as_ptr()];
    let priority = [1.0];
    let mut queue_create_infos = vec!(
        DeviceQueueCreateInfo{
            s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DeviceQueueCreateFlags::empty(),
            p_queue_priorities : priority.as_ptr(),
            queue_count : 1,
            queue_family_index : graphics_queue_family,
        }
    );
    debug!("Using queue family {} as the graphics queue family.",graphics_queue_family);
    if graphics_queue_family != compute_queue_family{
        queue_create_infos.push(
            DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                p_queue_priorities : priority.as_ptr(),
                queue_count : 1,
                queue_family_index : compute_queue_family,
            }
        );
    }
    debug!("Using queue family {} as the compute queue family.",compute_queue_family);
    if dma_queue_family.is_some(){
        let dma_queue_family = dma_queue_family.unwrap();
        debug!("Using queue family {} as the dma queue family.", dma_queue_family);
        if dma_queue_family != compute_queue_family && dma_queue_family != graphics_queue_family{
            queue_create_infos.push(
                DeviceQueueCreateInfo{
                    s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next : std::ptr::null(),
                    flags : DeviceQueueCreateFlags::empty(),
                    p_queue_priorities : priority.as_ptr(),
                    queue_count : 1,
                    queue_family_index : dma_queue_family,
                }
            );
        }
    } else {warn!("GPU does not have a dedicated DMA queue family.")}
    let device_create_info = DeviceCreateInfo{
        s_type : StructureType::DEVICE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DeviceCreateFlags::empty(),
        enabled_layer_count : 0,
        pp_enabled_layer_names : std::ptr::null(),
        p_enabled_features : &device_features,
        enabled_extension_count : device_extensions.len() as u32,
        pp_enabled_extension_names : device_extensions.as_ptr(),
        queue_create_info_count : queue_create_infos.len() as u32,
        p_queue_create_infos : queue_create_infos.as_ptr(),
    };
    return instance.create_device(*physical_device, &device_create_info, None).unwrap_or_else(|e|{error!("Failed to create Vulkan device handle, {}.",e);panic!("Failed to create Vulkan device.")});
}
unsafe fn get_surface_present_mode(surface_loader : &Surface , surface : &SurfaceKHR , physical_device : &PhysicalDevice) ->PresentModeKHR{
    let available_present_modes = surface_loader.get_physical_device_surface_present_modes(*physical_device, *surface).unwrap_or_else(|e|{
        error!("Failed to get surface present modes, {}.",e);
        panic!();
    });
    if available_present_modes.contains(&PresentModeKHR::MAILBOX){return PresentModeKHR::MAILBOX}else{return PresentModeKHR::FIFO}
}
unsafe fn get_surface_format(surface_loader : &Surface , surface : &SurfaceKHR , physical_device : &PhysicalDevice) -> SurfaceFormatKHR{
    let available_surface_formats = surface_loader.get_physical_device_surface_formats(*physical_device, *surface).unwrap_or_else(|e|{
        error!("Failed to get surface formats, {}",e);
        panic!();
    });
    let prefered_surface_formats = [
        SurfaceFormatKHR{format : Format::R8G8B8A8_UNORM, color_space : ColorSpaceKHR::SRGB_NONLINEAR},
        SurfaceFormatKHR{format : Format::B8G8R8A8_UNORM, color_space : ColorSpaceKHR::SRGB_NONLINEAR},
    ];
    for surface_format in prefered_surface_formats.iter(){
        if available_surface_formats.contains(surface_format){return *surface_format}
    }
    return available_surface_formats[0];
}
unsafe fn get_surface_extent(capabilities : &SurfaceCapabilitiesKHR , window : &Window) -> Extent2D{
    if capabilities.current_extent.width != u32::MAX{return capabilities.current_extent} else {
        let size = window.inner_size();
        return Extent2D{
            width : size.width,
            height : size.height,
        }
    }
}
unsafe fn create_swapchain(physical_device : &PhysicalDevice , swapchain_loader : &Swapchain , surface_loader : &Surface , surface : &SurfaceKHR , graphics_queue : u32 , presentation_queue : u32 , window : &Window) -> SwapchainKHR{
    let queues = [graphics_queue,presentation_queue];
    let present_mode = get_surface_present_mode(surface_loader, surface, physical_device);
    let surface_format = get_surface_format(surface_loader, surface, physical_device);
    let surface_capabilities = surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface).unwrap_or_else(|e|{
        error!("Failed to get surface capabilities, {}.",e);
        panic!();
    });
    let surface_extent = get_surface_extent(&surface_capabilities, window);
    let swapchain_create_info = SwapchainCreateInfoKHR{
        s_type : StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next : std::ptr::null(),
        flags : SwapchainCreateFlagsKHR::empty(),
        surface : *surface,
        image_usage : ImageUsageFlags::COLOR_ATTACHMENT,
        image_array_layers : 1,
        clipped : 1,
        composite_alpha : CompositeAlphaFlagsKHR::OPAQUE,
        old_swapchain : SwapchainKHR::null(),
        queue_family_index_count : if graphics_queue == presentation_queue{0}else{2},
        image_sharing_mode : if graphics_queue == presentation_queue{SharingMode::EXCLUSIVE}else{SharingMode::CONCURRENT},
        p_queue_family_indices : if graphics_queue == presentation_queue{std::ptr::null()}else{queues.as_ptr()},
        present_mode,
        image_format : surface_format.format,
        image_color_space : surface_format.color_space,
        image_extent : surface_extent,
        pre_transform : surface_capabilities.current_transform,
        min_image_count : if surface_capabilities.min_image_count + 1 <= surface_capabilities.max_image_count || surface_capabilities.max_image_count == 0{surface_capabilities.min_image_count+1}else{surface_capabilities.max_image_count}
    };
    return swapchain_loader.create_swapchain(&swapchain_create_info, None).unwrap_or_else(|e|{
        error!("Failed to create Swapchain, {}.",e);
        panic!();
    });
}
unsafe fn create_swapchain_image_views(device : &Device , images : &Vec<Image> , format : &Format)->Vec<ImageView>{
    let mut image_views = vec!();
    for &image in images.iter(){
        let image_view_create_info = ImageViewCreateInfo{
            s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : ImageViewCreateFlags::empty(),
            image,
            view_type : ImageViewType::TYPE_2D,
            format : *format,
            components : ComponentMapping{ r : ComponentSwizzle::R , g : ComponentSwizzle::G , b : ComponentSwizzle::B , a : ComponentSwizzle::A},
            subresource_range : ImageSubresourceRange{
                aspect_mask : ImageAspectFlags::COLOR,
                base_array_layer : 0,
                base_mip_level : 0,
                layer_count : 1,
                level_count : 1,
            },
        };
        image_views.push(device.create_image_view(&image_view_create_info, None).unwrap_or_else(|e|{
            error!("Failed to create Swapchain image view, {}.",e);
            panic!();
        }))
    }
    return image_views;
}
unsafe fn get_depth_image_format_and_tiling(instance : &Instance , physical_device : &PhysicalDevice)->(Format,ImageTiling){
    let prefered_formats = [
        Format::D24_UNORM_S8_UINT,
        Format::D16_UNORM,
        Format::D16_UNORM_S8_UINT,
        Format::D32_SFLOAT,
        Format::D32_SFLOAT_S8_UINT,
        Format::X8_D24_UNORM_PACK32,
    ];
    for &format in prefered_formats.iter(){
        if instance.get_physical_device_format_properties(*physical_device, format).optimal_tiling_features.contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT){return (format,ImageTiling::OPTIMAL)}
    }
    for &format in prefered_formats.iter(){
        if instance.get_physical_device_format_properties(*physical_device, format).linear_tiling_features.contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT){return (format,ImageTiling::LINEAR)}
    }
    error!("No supported depth format found.");
    panic!();
}
fn get_memorytype_index(memory_requirements: &MemoryRequirements, memory_properties: &PhysicalDeviceMemoryProperties, flags: MemoryPropertyFlags) -> Option<u32> {
    memory_properties.memory_types[..memory_properties.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_requirements.memory_type_bits != 0
                && memory_type.property_flags & flags == flags
        })
        .map(|(index, _memory_type)| index as _)
}
unsafe fn create_depth_images_and_view(device : &Device , format : &Format , tiling : &ImageTiling , extent : &Extent2D , instance : &Instance , physical_device : &PhysicalDevice) -> (Image,DeviceMemory,ImageView){
    let image_create_info = ImageCreateInfo{
        s_type : StructureType::IMAGE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : ImageCreateFlags::empty(),
        image_type : ImageType::TYPE_2D,
        format : *format,
        mip_levels : 1,
        array_layers : 1,
        sharing_mode : SharingMode::EXCLUSIVE,
        samples : SampleCountFlags::TYPE_1,
        queue_family_index_count : 0,
        p_queue_family_indices : std::ptr::null(),
        usage : ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        initial_layout : ImageLayout::UNDEFINED,
        tiling : *tiling,
        extent : Extent3D{width : extent.width , height : extent.height , depth : 1},
    };
    let depth_image = device.create_image(&image_create_info, None).unwrap_or_else(|e|{
        error!("Failed to create depth image, {}.",e);
        panic!();
    });
    let memory_requirements = device.get_image_memory_requirements(depth_image);
    let memory_properties = instance.get_physical_device_memory_properties(*physical_device);
    let device_local_memory = get_memorytype_index(&memory_requirements, &memory_properties, MemoryPropertyFlags::DEVICE_LOCAL);
    let memory_allocate_info = MemoryAllocateInfo{
        s_type : StructureType::MEMORY_ALLOCATE_INFO,
        p_next : std::ptr::null(),
        allocation_size : memory_requirements.size,
        memory_type_index : if device_local_memory.is_some(){device_local_memory.unwrap()}else{get_memorytype_index(&memory_requirements, &memory_properties, MemoryPropertyFlags::empty()).unwrap_or_else(||{
            error!("No supported memory type index");
            panic!();
        })}
    };
    let allocation = device.allocate_memory(&memory_allocate_info, None).unwrap_or_else(|e|{
        error!("Failed to allocate memory for the depth image, {}.",e);
        panic!();
    });
    device.bind_image_memory(depth_image, allocation, 0).unwrap_or_else(|e|{
        error!("Failed to bind depth image memory, {}.",e);
        panic!();
    });
    let image_view_create_info = ImageViewCreateInfo{
        s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : ImageViewCreateFlags::empty(),
        format : *format,
        image : depth_image,
        view_type : ImageViewType::TYPE_2D,
        components : ComponentMapping{ r : ComponentSwizzle::R , g : ComponentSwizzle::G , b : ComponentSwizzle::B , a : ComponentSwizzle::A},
        subresource_range : ImageSubresourceRange{
            aspect_mask : ImageAspectFlags::DEPTH,
            base_array_layer : 0,
            base_mip_level : 0,
            layer_count : 1,
            level_count : 1,
        },
    };
    let depth_image_view = device.create_image_view(&image_view_create_info, None).unwrap_or_else(|e|{
        error!("Failed to create depth image view, {}.",e);
        panic!("");
    });
    return (depth_image,allocation,depth_image_view);
}
unsafe fn create_render_pass(device : &Device  , format : &Format , depth_format : &Format) -> RenderPass{
    let attachments = [AttachmentDescription{
        flags : AttachmentDescriptionFlags::empty(),
        format : *format,
        samples : SampleCountFlags::TYPE_1,
        load_op : AttachmentLoadOp::CLEAR,
        store_op : AttachmentStoreOp::STORE,
        stencil_load_op : AttachmentLoadOp::DONT_CARE,
        stencil_store_op : AttachmentStoreOp::DONT_CARE,
        initial_layout : ImageLayout::UNDEFINED,
        final_layout : ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    },
    AttachmentDescription{
        flags : AttachmentDescriptionFlags::empty(),
        format : *depth_format,
        samples : SampleCountFlags::TYPE_1,
        load_op : AttachmentLoadOp::CLEAR,
        store_op : AttachmentStoreOp::STORE,
        stencil_load_op : AttachmentLoadOp::DONT_CARE,
        stencil_store_op : AttachmentStoreOp::DONT_CARE,
        initial_layout : ImageLayout::UNDEFINED,
        final_layout : ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    }];
    let color_references = [AttachmentReference{
        attachment : 0,
        layout : ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let depth_reference = AttachmentReference{
        attachment : 1,
        layout : ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let subpasses = [SubpassDescription{
        flags : SubpassDescriptionFlags::empty(),
        pipeline_bind_point : PipelineBindPoint::GRAPHICS,
        input_attachment_count : 0,
        p_input_attachments : std::ptr::null(),
        color_attachment_count : color_references.len() as u32,
        p_color_attachments : color_references.as_ptr(),
        p_resolve_attachments : std::ptr::null(),
        p_depth_stencil_attachment : &depth_reference,
        preserve_attachment_count : 0,
        p_preserve_attachments : std::ptr::null(),
    }];
    let subpass_dependencies = [SubpassDependency{
        src_subpass : 0,
        dst_subpass : SUBPASS_EXTERNAL,
        src_stage_mask : PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask : PipelineStageFlags::FRAGMENT_SHADER,
        src_access_mask :  AccessFlags::COLOR_ATTACHMENT_WRITE,
        dst_access_mask : AccessFlags::SHADER_READ,
        dependency_flags : DependencyFlags::empty(),
    }];
    let render_pass_create_info = RenderPassCreateInfo{
        s_type : StructureType::RENDER_PASS_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : RenderPassCreateFlags::empty(),
        attachment_count : attachments.len() as u32,
        p_attachments : attachments.as_ptr(),
        subpass_count : subpasses.len() as u32,
        p_subpasses : subpasses.as_ptr(),
        dependency_count : subpass_dependencies.len() as u32,
        p_dependencies : subpass_dependencies.as_ptr(),
    };
    return device.create_render_pass(&render_pass_create_info, None).unwrap_or_else(|e|{
        error!("Failed to create Render Pass, {}.",e);
        panic!();
    });
}
unsafe fn create_framebuffers(device : &Device , render_pass : &RenderPass , extent : &Extent2D , swapchain_image_views : &Vec<ImageView> , depth_image : &ImageView)->Vec<Framebuffer>{
    let mut framebuffers = vec!();
    for &swapchain_image_view in swapchain_image_views.iter(){
        let attachments = [swapchain_image_view,*depth_image];
        let framebuffer_create_info = FramebufferCreateInfo{
            s_type : StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : FramebufferCreateFlags::empty(),
            render_pass : *render_pass,
            width : extent.width,
            height : extent.height,
            attachment_count : attachments.len() as u32,
            p_attachments : attachments.as_ptr(),
            layers : 1,
        };
        framebuffers.push(device.create_framebuffer(&framebuffer_create_info, None).unwrap_or_else(|e|{
            error!("Failed to create Framebuffer, {}.",e);
            panic!();
        }));
    }
    return framebuffers;
}