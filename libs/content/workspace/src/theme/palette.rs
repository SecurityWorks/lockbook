use egui::Color32;
use lb_rs::model::svg::element::DynamicColor;

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ThemePalette {
    pub black: Color32,
    pub red: Color32,
    pub green: Color32,
    pub yellow: Color32,
    pub blue: Color32,
    pub magenta: Color32,
    pub cyan: Color32,
    pub white: Color32,
}

impl ThemePalette {
    pub const DARK: Self = Self {
        black: Color32::from_rgb(0, 0, 0),
        red: Color32::from_rgb(255, 69, 58),
        green: Color32::from_rgb(50, 215, 75),
        yellow: Color32::from_rgb(255, 214, 10),
        blue: Color32::from_rgb(10, 132, 255),
        magenta: Color32::from_rgb(191, 90, 242),
        cyan: Color32::from_rgb(90, 200, 245),
        white: Color32::from_rgb(255, 255, 255),
    };

    pub const LIGHT: Self = Self {
        black: Color32::from_rgb(0, 0, 0),
        red: Color32::from_rgb(255, 59, 48),
        green: Color32::from_rgb(40, 205, 65),
        yellow: Color32::from_rgb(255, 204, 0),
        blue: Color32::from_rgb(10, 132, 255),
        magenta: Color32::from_rgb(175, 82, 222),
        cyan: Color32::from_rgb(85, 190, 240),
        white: Color32::from_rgb(255, 255, 255),
    };

    pub fn resolve_dynamic_color(dynamic_color: DynamicColor, dark_mode: bool) -> egui::Color32 {
        if dark_mode {
            Color32::from_rgb(
                dynamic_color.dark.red,
                dynamic_color.dark.green,
                dynamic_color.dark.blue,
            )
        } else {
            Color32::from_rgb(
                dynamic_color.light.red,
                dynamic_color.light.green,
                dynamic_color.light.blue,
            )
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ColorAlias {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl std::ops::Index<ColorAlias> for ThemePalette {
    type Output = Color32;

    fn index(&self, alias: ColorAlias) -> &Self::Output {
        use ColorAlias::*;
        match alias {
            Black => &self.black,
            Red => &self.red,
            Green => &self.green,
            Yellow => &self.yellow,
            Blue => &self.blue,
            Magenta => &self.magenta,
            Cyan => &self.cyan,
            White => &self.white,
        }
    }
}
