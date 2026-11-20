use horizon_mod_sdk::{
    Mod, Registration, game,
    render::{self, FillShape, Point, Rect, Shape, TextShape},
    storage,
    ui::{self, ButtonWidget, SliderWidget, WidgetV2 as Widget},
};

const SCALE_MIN: u32 = 50;
const SCALE_MAX: u32 = 200;
const SCALE_STEP: u32 = 5;

const SHOW: u32 = 1;
const BACKGROUND: u32 = 2;
const COORDS: u32 = 3;
const DAY: u32 = 4;
const MOVE: u32 = 5;
const RESET: u32 = 6;
const DONE: u32 = 7;
const S_SIZE: u32 = 20;
const S_X: u32 = 21;
const S_Y: u32 = 22;
const S_OPACITY: u32 = 23;

const FILE: &str = "coords_day.txt";

struct CoordsDay {
    show: bool,
    background: bool,
    coords: bool,
    day: bool,
    scale: u32,   // percent, 50 to 200
    x: u32,       // 0 (left) to 1000 (right)
    y: u32,       // 0 (top) to 1000 (bottom)
    opacity: u32, // background, 0 to 100
    last: (String, u32, u32),
    drawn: bool,
    widest: f32,
    area: (f32, f32, f32, f32, f32, f32),
}

fn on_off(on: bool) -> &'static str {
    if on { "On" } else { "Off" }
}

fn load_config() -> Vec<(String, String)> {
    let Some(text) = storage::get(FILE).and_then(|v| String::from_utf8(v).ok()) else {
        return Vec::new();
    };
    text.lines()
