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

pub struct Hotbar {
    button_size: i32,
    button_padding: i32,
    buttons: Vec<String>,
}

pub struct HotbarButton {
    icon: String,
    size: i32,
    offset: i32,
}

impl Hotbar {
    pub fn new() -> Hotbar {
        Hotbar {
            button_size: 128,
            button_padding: 8,
            buttons: vec![
                "assets/icon_attack.png".to_string(),
            ],
        }
    }
}

impl Component for Hotbar {
    fn render(&self) -> Element {
        let width = 
            (self.button_padding + self.button_size) *
            self.buttons.len() as i32 +
            self.button_padding;

        let mut bar = Element::new(
            Position {
                x: Offset::new(0.5, -(width / 2)),
                y: Offset::new(0.85, -(self.button_size / 2)),
            },
            Dimension {
                width: Offset::new(0.0, width),
                height: Offset::new(0.0, self.button_size),
            },
            None
        );

        for (index, child) in (&self.buttons).iter().enumerate() {
            bar.add_child(Box::new(HotbarButton {
                icon: child.clone(),
                size: self.button_size,
                offset: index as i32 * (self.button_size + self.button_padding),
            }));
        }

        bar
    }
}

impl Component for HotbarButton {
    fn render(&self) -> Element {
        Element::new(
            Position {
                x: Offset::new(0.0, self.offset),
                y: Offset::new(0.0, 0),
            },
            Dimension {
                width: Offset::new(0.0, self.size),
                height: Offset::new(0.0, self.size),
            },
            Some(ElementDisplay::new(
                self.icon.clone(),
                [0.0, 0.0],
                [1.0, 1.0]
            ))
        )
    }
}