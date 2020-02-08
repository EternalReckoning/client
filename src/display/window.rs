use failure::{
    format_err,
    Error,
};

use super::displayconfig::{
    DisplayConfig,
    DisplayMode,
};

pub struct Window {
    window: winit::window::WindowBuilder,
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

        let window = builder;

        Ok(Window { window, event_loop })
    }

    pub fn split(self) -> (winit::window::WindowBuilder, winit::event_loop::EventLoop<()>) {
        (self.window, self.event_loop)
    }
}