use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const RADIUS: i32 = 33;

struct World {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello World")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            world.update();
            window.request_redraw();
        }
    });
}

impl World {
    fn new() -> Self {
        Self {
            x: 70,
            y: 50,
            vx: -1,
            vy: -1,
        }
    }

    fn update(&mut self) {
        if self.x - RADIUS < 0 || self.x + RADIUS > WIDTH as i32 {
            self.vx *= -1;
        }
        if self.y - RADIUS < 0 || self.y + RADIUS > HEIGHT as i32 {
            self.vy *= -1;
        }

        self.x += self.vx;
        self.y += self.vy;
    }

    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let px = (i % WIDTH as usize) as i32;
            let py = (i / WIDTH as usize) as i32;

            let dist = (px - self.x).pow(2) + (py - self.y).pow(2);
            let inside = dist <= RADIUS.pow(2);

            let rgba = if inside {
                [0x03, 0x73, 0xa4, 0xff]
            } else {
                [0xe8, 0xd3, 0xb9, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
