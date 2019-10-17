use failure::{
    format_err,
    Error,
};
use rendy::wsi::winit;

use super::displayconfig::{
    DisplayConfig,
    DisplayMode,
};

pub struct Window {
    window: winit::window::Window,
    event_loop: winit::event_loop::EventLoop<()>,
}

impl Window {
    pub fn new(config: &DisplayConfig) -> Result<Window, Error> {
        let event_loop = winit::event_loop::EventLoop::new();

        let mut builder = winit::window::WindowBuilder::new()
            .with_title("Eternal Reckoning");

        let monitor = event_loop.primary_monitor();

        builder = builder.with_fullscreen(match config.display_mode {
            DisplayMode::Windowed => None,
            DisplayMode::Borderless => Some(
                winit::window::Fullscreen::Borderless(monitor)
            ),
            DisplayMode::Fullscreen => {
                let mut mode: _ = None;
                for avail_mode in monitor.video_modes() {
                    if avail_mode.size() == monitor.size() {
                        mode = Some(avail_mode);
                        break;
                    }
                }

                if mode.is_none() {
                    return Err(
                        format_err!("no suitable video mode found")
                    );
                }

                Some(winit::window::Fullscreen::Exclusive(mode.unwrap()))
            },
        });

        let window = builder
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
