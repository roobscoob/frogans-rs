//! The shared wgpu context: one [`wgpu::Instance`] for the process, and — created
//! lazily from the first window's surface — one adapter/device/queue that every
//! surface then shares.
//!
//! The device is initialized lazily because picking an adapter wants a concrete
//! surface to be compatible with; the first window provides it.

use std::sync::Arc;

use winit::window::Window;

/// Process-wide GPU state shared by every [`Surface`](crate::surface::Surface).
pub struct Gpu {
    instance: wgpu::Instance,
    device: Option<DeviceCtx>,
}

/// The adapter/device/queue, created once from the first window's surface.
struct DeviceCtx {
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Gpu {
    /// A new context with just the instance; the device comes up on the first
    /// [`configure_window`](Gpu::configure_window).
    pub fn new() -> Self {
        Gpu {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle()),
            device: None,
        }
    }

    /// The shared device. Panics if called before the first window is configured.
    pub fn device(&self) -> &wgpu::Device {
        &self.expect_device().device
    }

    /// The shared queue. Panics if called before the first window is configured.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.expect_device().queue
    }

    /// Create and configure a swap-chain surface for `window`, bringing the shared
    /// device up on the first call. Returns the configured surface and its config.
    ///
    /// The surface owns an `Arc<Window>` clone, so it is `'static` and outlives the
    /// borrow handed in here.
    pub fn configure_window(
        &mut self,
        window: &Arc<Window>,
        transparent: bool,
    ) -> (wgpu::Surface<'static>, wgpu::SurfaceConfiguration) {
        let surface = self
            .instance
            .create_surface(Arc::clone(window))
            .expect("failed to create wgpu surface");

        let ctx = self.ensure_device(&surface);
        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&ctx.adapter, size.width.max(1), size.height.max(1))
            .expect("surface not supported by the adapter");

        // For a transparent window, composite with the window: egui outputs
        // premultiplied alpha, so prefer `PreMultiplied` (then `PostMultiplied`).
        // If the backend offers neither, the window stays opaque.
        if transparent {
            let caps = surface.get_capabilities(&ctx.adapter);
            for mode in [
                wgpu::CompositeAlphaMode::PreMultiplied,
                wgpu::CompositeAlphaMode::PostMultiplied,
            ] {
                if caps.alpha_modes.contains(&mode) {
                    config.alpha_mode = mode;
                    break;
                }
            }
        }
        surface.configure(&ctx.device, &config);

        (surface, config)
    }

    /// Initialize the shared device (compatible with `surface`) on first use.
    fn ensure_device(&mut self, surface: &wgpu::Surface<'_>) -> &DeviceCtx {
        if self.device.is_none() {
            let adapter = pollster::block_on(self.instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: Some(surface),
                },
            ))
            .expect("no compatible wgpu adapter");

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("frogans-surfaces"),
                    ..Default::default()
                },
            ))
            .expect("failed to create wgpu device");

            self.device = Some(DeviceCtx {
                adapter,
                device,
                queue,
            });
        }
        self.expect_device()
    }

    fn expect_device(&self) -> &DeviceCtx {
        self.device
            .as_ref()
            .expect("wgpu device used before the first window was configured")
    }
}
