use winit::event_loop::EventLoop;


pub fn create_event_loop() -> EventLoop<()> {
    winit::event_loop::EventLoopBuilder::with_user_event().build()
}
