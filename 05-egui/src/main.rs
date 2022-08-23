use egui::{ClippedPrimitive, Context, TexturesDelta};
use egui_wgpu::renderer::{RenderPass, ScreenDescriptor};
use egui_wgpu::wgpu;
use egui_winit::winit::dpi::LogicalSize;
use egui_winit::winit::event::{Event, VirtualKeyCode};
use egui_winit::winit::event_loop::EventLoopWindowTarget;
use egui_winit::winit::event_loop::{ControlFlow, EventLoop};
use egui_winit::winit::window::Window;
use egui_winit::winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

struct UiContext {
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    rpass: RenderPass,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    ui: Ui,
}

impl UiContext {
    fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let max_texture_side = device.limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_side);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let rpass = RenderPass::new(device, surface_format, 1);
        let textures = TexturesDelta::default();
        let ui = Ui::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            rpass,
            paint_jobs: Vec::new(),
            textures,
            ui,
        }
    }

    fn on_event(&mut self, event: &egui_winit::winit::event::WindowEvent) {
        self.egui_state.on_event(&self.egui_ctx, event);
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    fn on_scale_factor_update(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    fn prepare(&mut self, window: &Window) {
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            self.ui.ui(egui_ctx);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        for (id, image_delta) in &self.textures.set {
            self.rpass.update_texture(device, queue, *id, image_delta);
        }

        self.rpass
            .update_buffers(device, queue, &self.paint_jobs, &self.screen_descriptor);

        self.rpass.execute(
            encoder,
            render_target,
            &self.paint_jobs,
            &self.screen_descriptor,
            None,
        );

        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.rpass.free_texture(id);
        }
    }
}

struct Ui {
    window_open: bool,
}

impl Ui {
    fn new() -> Self {
        Self { window_open: true }
    }

    fn ui(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Windows", |ui| {
                    if ui.button("Toggle Window").clicked() {
                        self.window_open = !self.window_open;
                        ui.close_menu();
                    }
                })
            });
        });

        egui::Window::new("Hello World!")
            .open(&mut self.window_open)
            .show(ctx, |ui| {
                ui.label("Hi.");
            });
    }
}

struct SurfaceContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_format: wgpu::TextureFormat,
}

impl SurfaceContext {
    fn new<T: raw_window_handle::HasRawWindowHandle>(window: &T, width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .unwrap();

        let surface_format = surface.get_supported_formats(&adapter)[0];

        let mut result = Self {
            surface,
            device,
            queue,
            surface_format,
        };

        result.on_resize(width, height);

        result
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: width,
                height: height,
                present_mode: wgpu::PresentMode::Fifo,
            };
            self.surface.configure(&self.device, &surface_config);
        }
    }

    fn render_with<F>(&mut self, render_function: F)
    where
        F: FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, &wgpu::Device, &wgpu::Queue) -> (),
    {
        let output_frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                return;
            }
            Err(e) => {
                log::error!("render failed: {}", e);
                return;
            }
        };
        let render_target = output_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        render_function(&mut encoder, &render_target, &self.device, &self.queue);

        self.queue.submit(std::iter::once(encoder.finish()));
        output_frame.present();
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(640, 480);
        WindowBuilder::new()
            .with_title("Hello egui")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut surface_ctx = {
        let window_size = window.inner_size();
        SurfaceContext::new(&window, window_size.width, window_size.height)
    };

    let mut ui_ctx = {
        let window_size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        UiContext::new(
            &event_loop,
            window_size.width,
            window_size.height,
            scale_factor,
            &surface_ctx.device,
            surface_ctx.surface_format,
        )
    };

    event_loop.run(move |event, _, control_flow| {
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(scale_factor) = input.scale_factor() {
                ui_ctx.on_scale_factor_update(scale_factor);
            }

            if let Some(size) = input.window_resized() {
                surface_ctx.on_resize(size.width, size.height);
                ui_ctx.on_resize(size.width, size.height);
            }

            window.request_redraw();
        }

        match event {
            Event::WindowEvent { event, .. } => {
                ui_ctx.on_event(&event);
            }

            Event::RedrawRequested(_) => {
                ui_ctx.prepare(&window);

                surface_ctx.render_with(|encoder, render_target, device, queue| {
                    ui_ctx.render(encoder, render_target, device, queue);
                });
            }
            _ => (),
        }
    });
}
