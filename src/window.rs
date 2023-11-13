use super::errors::{ RviError, Result };
use winit::{
    dpi::{ PhysicalSize, PhysicalPosition },
    event_loop::EventLoop,
    monitor::MonitorHandle,
    window::{ Window, WindowBuilder },
};

pub fn create_window(event_loop: &EventLoop<()>, size: PhysicalSize<u32>) -> Result<Window> {
    WindowBuilder::new()
        .with_title("RIV")
        .with_inner_size(size)
        .with_position(PhysicalPosition::new(20, 20))
        .build(&event_loop)
		.map_err(RviError::WindowError)
}

pub fn get_screen_size(
    event_loop: &EventLoop<()>
) -> Result<PhysicalSize<u32>> {
    let primary_monitor: MonitorHandle = event_loop
        .primary_monitor()
        .ok_or(RviError::NoPrimaryMonitor)?;

    Ok(primary_monitor.size())
}
