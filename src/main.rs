use image::imageops::FilterType;
use pixels::{SurfaceTexture, Pixels};
use winit::{
    event::{ElementState, KeyboardInput, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::PhysicalSize,
};

use clap::Parser;
use thiserror::Error;

const SCREEN_PERCENT: u32 = 90;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Config {
    /// Name of image to open
    file_name: String,

    /// Wether to scale the image up
    #[clap(short, long, takes_value = false)]
    up_scale: bool
}

// Custom error type that will handle errors inside
#[derive(Debug, Error)]
enum RviError {
    #[error("Unable to create window")]
    WindowError(#[from] winit::error::OsError),

    #[error("An error occurred while processing the image")]
    ImageError(#[from] image::ImageError),

    #[error("And error occurred while loading the image")]
    IoError(#[from] std::io::Error),

    #[error("Unable to create new pixels instance")]
    PixelsError(#[from] pixels::Error),

    #[error("Unable to convert image to RGB8")]
    ImageConversionError,

    #[error("Cannot find primary monitor")]
    NoPrimaryMonitor,
}

// Define type for main return result to auto convert error
type Result<T> = std::result::Result<T, RviError>;

// TODO: Make image rescale on event called the least
fn main() -> Result<()> {
    let config = Config::parse();
    
    let resize_pixels = |pixels: &mut Pixels, size: &PhysicalSize<u32>| {
        pixels.resize_surface(size.width, size.height)
    };

    let calc_scale_factor = |max_size: u32, current_size: u32| {
        if max_size >= current_size && !config.up_scale {
            return 1 as f32;
        }
        current_size as f32 / max_size as f32
    };

    let image = image::io::Reader::open(&config.file_name)?.with_guessed_format()?.decode()?;

    let event_loop = EventLoop::new();
    let primary_monitor = event_loop.primary_monitor().ok_or(RviError::NoPrimaryMonitor)?;

    let screen_size = primary_monitor.size();

    // Get an array of scale float sizes for scaling
    let mut scale = [
            calc_scale_factor(screen_size.width * SCREEN_PERCENT / 100, image.width()),
            calc_scale_factor(screen_size.height * SCREEN_PERCENT / 100, image.height())
    ];
    float_ord::sort(&mut scale);
    let scale = scale[1];

    let image = image.resize(
            (image.width() as f32 / scale).ceil() as u32, 
            (image.height() as f32 / scale).ceil() as u32, 
            FilterType::Lanczos3);

    let window_inner_size = PhysicalSize::new(image.width(), image.height());

    let window = WindowBuilder::new().with_title("RIV").with_inner_size(window_inner_size).build(&event_loop)?;

    let surface = SurfaceTexture::new(window_inner_size.width, window_inner_size.height, &window);
    let mut pixels = Pixels::new(window_inner_size.width, window_inner_size.height, surface)?;

    let image_bytes = image.as_rgb8().ok_or(RviError::ImageConversionError)?.as_flat_samples();
    let image_bytes = image_bytes.as_slice();

    image_bytes
        .chunks_exact(3)
        .zip(pixels.get_frame().chunks_exact_mut(4))
        .for_each(|(image_pixel, pixel)| {
            pixel[0] = image_pixel[0];
            pixel[1] = image_pixel[1];
            pixel[2] = image_pixel[2];
            pixel[3] = 0xff;
        });

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            winit::event::Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    winit::event::WindowEvent::Resized(size) => {
                        resize_pixels(&mut pixels, &size);
                    }
                    winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    winit::event::WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,

                    winit::event::WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        ..
                    } => {
                        resize_pixels(&mut pixels, new_inner_size);
                    }

                    _ => {}
                }
            }
            winit::event::Event::RedrawRequested(_) => {
                let _ = pixels.render();
            }

            _ => {}
        }
    })
}
