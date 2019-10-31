use eternalreckoning_ui::{
    Component,
    dimension::{
        Dimension,
        Offset,
        Position,
    },
    element::{
        Element,
        ElementDisplay,
    }
};

pub struct Splash;

pub struct SplashLogo {
    texture: String,
    width: i32,
    height: i32,
}

impl Splash {
    pub fn new() -> Splash {
        Splash {}
    }
}

impl Component for Splash {
    fn render(&self) -> Element {
        let mut splash = Element::new(
            Position {
                x: Offset::new(0.0, 0),
                y: Offset::new(0.0, 0),
            },
            Dimension {
                width: Offset::new(1.0, 0),
                height: Offset::new(1.0, 0),
            },
            None
        );

        splash.add_child(Box::new(SplashLogo {
            texture: "assets/logo.png".to_string(),
            width: 512,
            height: 512,
        }));

        splash
    }
}

impl Component for SplashLogo {
    fn render(&self) -> Element {
        Element::new(
            Position {
                x: Offset::new(0.5, -self.width / 2),
                y: Offset::new(0.5, -self.height / 2),
            },
            Dimension {
                width: Offset::new(0.0, self.width),
                height: Offset::new(0.0, self.height),
            },
            Some(ElementDisplay::new(
                self.texture.clone(),
                [0.0, 0.0],
                [1.0, 1.0]
            ))
        )
    }
}