use std::option_env;
use std::time::{ Instant, Duration };

use image::DynamicImage;
use pixels::wgpu::RequestAdapterOptions;
use pixels::{ Pixels, PixelsBuilder, SurfaceTexture };
use winit::{
    dpi::PhysicalSize,
    event::{ ElementState, KeyboardInput, VirtualKeyCode },
    event_loop::ControlFlow,
    window::Window,
};

use clap::Parser;

mod config;
mod errors;
mod events;
mod graphics;
mod window;

use crate::config::Config;
use crate::errors::Result;
use crate::events::create_event_loop;
use crate::graphics::redraw_surface;
use crate::window::{ get_screen_size, create_window };

const SCREEN_PERCENT: u32 = 90;

fn main() -> Result<()> {
    let mut resize_requested = false;
    let mut last_resize = Instant::now();
    let debounce_duration = Duration::from_millis(100);

    if cfg!(debug_assertions) {
        std::env::set_var("RUST_BACKTRACE", "full");
    }
    let config: Config = Config::parse();

    if cfg!(debug_assertions) {
        println!("Fetching and decoding stream image");
    }
    let stream_image: DynamicImage = image::io::Reader
        ::open(&config.file_name)?
        .with_guessed_format()?
        .decode()?;

    let event_loop = create_event_loop();

    let screen_size: PhysicalSize<u32> = match get_screen_size(&event_loop) {
        Ok(screen_size) => screen_size,
        Err(_) => {
            let ss: (u32, u32) = match option_env!("SCREEN_SIZE") as Option<&str> {
                Some(ss) => {
                    let ss: Vec<&str> = ss.splitn(2, ",").collect();
                    (ss[0].parse().unwrap(), ss[1].parse().unwrap())
                }
                None => (640, 480),
            };
            PhysicalSize::new(ss.0, ss.1)
        }
    };

    if cfg!(debug_assertions) {
        dbg!(screen_size);
    }

    let mut scale: [f32; 2] = [
        calc_scale_factor(
            &((screen_size.width * SCREEN_PERCENT) / 100),
            &stream_image.width(),
            Some(config.up_scale)
        ),
        calc_scale_factor(
            &((screen_size.height * SCREEN_PERCENT) / 100),
            &stream_image.height(),
            Some(config.up_scale)
        ),
    ];

    float_ord::sort(&mut scale);

    let scale: f32 = scale[1];

    let window_inner_size: PhysicalSize<u32> = PhysicalSize::new(
        ((stream_image.width() as f32) / scale).ceil() as u32,
        ((stream_image.height() as f32) / scale).ceil() as u32
    );

    if cfg!(debug_assertions) {
        println!("Creating a new window");
    }

    let window = create_window(&event_loop, window_inner_size)?;

    let surface: SurfaceTexture<Window> = SurfaceTexture::new(
        window_inner_size.width,
        window_inner_size.height,
        &window
    );

    if cfg!(debug_assertions) {
        println!("Building initial pixels with low performance mode as:");
        dbg!(config.low_performance_mode);
        // Enumerate adapters
        let instance = pixels::wgpu::Instance::new(pixels::wgpu::Backends::all());
        for adapter in instance.enumerate_adapters(pixels::wgpu::Backends::all()) {
            dbg!(adapter);
        }
    }
    let mut pixels: Pixels = PixelsBuilder::new(
        window_inner_size.width,
        window_inner_size.height,
        surface
    )
        .device_descriptor(pixels::wgpu::DeviceDescriptor {
            features: pixels::wgpu::Features::empty(),
            limits: pixels::wgpu::Limits::default(),
            label: None,
        })
        .request_adapter_options(RequestAdapterOptions {
            power_preference: if config.low_performance_mode {
                pixels::wgpu::PowerPreference::default()
            } else {
                pixels::wgpu::PowerPreference::HighPerformance
            },
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .wgpu_backend(pixels::wgpu::Backends::all())
        .enable_vsync(false)
        .build()?;

    redraw_surface(&mut pixels, &window_inner_size, &stream_image)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent { window_id, event } if window_id == window.id() =>
                match event {
                    winit::event::WindowEvent::Resized(_) => {
                        last_resize = Instant::now();
                        resize_requested = true;
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    winit::event::WindowEvent::KeyboardInput {
                        input: KeyboardInput { state: ElementState::Pressed, virtual_keycode, .. },
                        ..
                    } =>
                        match virtual_keycode {
                            Some(VirtualKeyCode::Escape) => {
                                *control_flow = ControlFlow::Exit;
                            }
                            Some(VirtualKeyCode::R) => {
                                redraw_surface(
                                    &mut pixels,
                                    &window.inner_size(),
                                    &stream_image
                                ).unwrap();
                            }
                            _ => {}
                        }
                    _ => {}
                }
            winit::event::Event::MainEventsCleared => {
                if resize_requested && last_resize.elapsed() >= debounce_duration {
                    last_resize = Instant::now() - debounce_duration;
                    resize_requested = false;

                    redraw_surface(&mut pixels, &window.inner_size(), &stream_image).unwrap();
                    if cfg!(debug_assertions) { println!("redrawing surface") }
                }
            }
            _ => {}
        }
    })
}

fn calc_scale_factor(max_size: &u32, current_size: &u32, up_scale: Option<bool>) -> f32 {
    if max_size >= current_size && !up_scale.unwrap_or(false) {
        return 1 as f32;
    }
    (*current_size as f32) / (*max_size as f32)
}
