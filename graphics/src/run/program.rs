use std::sync::Arc;

use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo,
        QueueFlags,
    },
    image::ImageUsage,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    memory::allocator::StandardMemoryAllocator,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    VulkanLibrary,
};

use crate::run;
use crate::spec;

use super::shaders::ShaderCache;

pub(super) struct Program {
    shader_cache: ShaderCache,
    render_passes: Vec<run::RenderPass>,
}
impl Program {
    pub fn build(spec: &spec::Program) -> Self {
        let mut shader_cache = ShaderCache::new();

        let library = VulkanLibrary::new().unwrap();

        let enabled_extensions = get_surface_required_extensions(&spec.surface);

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions,
                enumerate_portability: true,
                ..Default::default()
            },
        )
        .unwrap();

        let surface = get_surface(&spec.surface, instance.clone());

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.intersects(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .expect("no suitable physical device found");

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let (mut swapchain, swapchain_images) = {
            let surface_capabilities = device
                .physical_device()
                .surface_capabilities(&surface, Default::default())
                .unwrap();

            let image_format = Some(
                device
                    .physical_device()
                    .surface_formats(&surface, Default::default())
                    .unwrap()[0]
                    .0,
            );

            let resolution = spec.surface.renderable_size();
            Swapchain::new(
                device.clone(),
                surface,
                SwapchainCreateInfo {
                    min_image_count: surface_capabilities.min_image_count.max(2),
                    image_format,
                    image_extent: [resolution.0, resolution.1],
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha: surface_capabilities
                        .supported_composite_alpha
                        .into_iter()
                        .next()
                        .unwrap(),

                    ..Default::default()
                },
            )
            .unwrap()
        };

        let memory_allocator = StandardMemoryAllocator::new_default(device.clone());

        let render_passes = spec
            .render_passes
            .iter()
            .map(|render_pass| {
                run::RenderPass::build(
                    render_pass,
                    device.clone(),
                    swapchain.clone(),
                    swapchain_images.clone(),
                    &memory_allocator,
                    &mut shader_cache,
                )
            })
            .collect::<Vec<_>>();

        Self {
            shader_cache,
            render_passes,
        }
    }
}

fn get_surface(surface: &spec::Surface, instance: Arc<Instance>) -> Arc<Surface> {
    unsafe {
        match surface {
            crate::Surface::Window { winit_window } => match (
                winit_window.raw_window_handle(),
                winit_window.raw_display_handle(),
            ) {
                (RawWindowHandle::Wayland(window), RawDisplayHandle::Wayland(display)) => {
                    Surface::from_wayland(instance, display.display, window.surface, None)
                }
                (RawWindowHandle::Win32(window), RawDisplayHandle::Windows(_display)) => {
                    Surface::from_win32(instance, window.hinstance, window.hwnd, None)
                }
                (RawWindowHandle::Xcb(window), RawDisplayHandle::Xcb(display)) => {
                    Surface::from_xcb(instance, display.connection, window.window, None)
                }
                (RawWindowHandle::Xlib(window), RawDisplayHandle::Xlib(display)) => {
                    Surface::from_xlib(instance, display.display, window.window, None)
                }
                _ => unimplemented!(),
            },
        }
        .unwrap()
    }
}

fn get_surface_required_extensions(surface: &spec::Surface) -> InstanceExtensions {
    let mut extensions = InstanceExtensions {
        khr_surface: true,
        ..InstanceExtensions::empty()
    };

    match surface {
        crate::Surface::Window { winit_window } => match winit_window.raw_display_handle() {
            RawDisplayHandle::Windows(_) => extensions.khr_win32_surface = true,
            RawDisplayHandle::Wayland(_) => extensions.khr_wayland_surface = true,
            RawDisplayHandle::Xlib(_) => extensions.khr_xlib_surface = true,
            RawDisplayHandle::Xcb(_) => extensions.khr_xcb_surface = true,
            _ => unimplemented!("Cannot get required extensions for display handle"),
        },
    }

    extensions
}
