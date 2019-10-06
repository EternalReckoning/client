use failure::{
    format_err,
    Error,
};
use rendy::wsi::winit;

pub struct Window {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl Window {
    pub fn new() -> Result<Window, Error> {
        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("World Client")
            .build(&event_loop)
            .map_err(|err| format_err!("failed to create window: {:?}", err))?;

        Ok(Window { window, event_loop })
    }

    pub fn get_size(&self) -> winit::dpi::PhysicalSize {
        self.window
            .inner_size()
            .to_physical(self.window.hidpi_factor())
    }

    pub fn get_aspect_ratio(&self) -> f64 {
        let size = self.get_size();

        size.width / size.height
    }

    pub fn create_surface<B>(
        &self,
        factory: &mut rendy::factory::Factory<B>,
    ) -> rendy::wsi::Surface<B>
    where
        B: rendy::hal::Backend,
    {
        factory.create_surface(&self.window)
    }

    pub fn run<F>(self, event_handler: F)
    where
        F: 'static + FnMut(winit::event::Event<()>, &winit::event_loop::EventLoopWindowTarget<()>, &mut winit::event_loop::ControlFlow),
    {
        self.event_loop.run(event_handler);
    }
}
