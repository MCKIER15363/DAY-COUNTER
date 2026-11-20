use horizon_mod_sdk::{
    Mod, Registration, game,
    render::{self, Point, Shape, TextShape},
};

struct CoordsDay {
    // The text drawn last, so the screen is only updated when it changes.
    last: String,
}

/// The lines to show: position, then day and time of day.
fn lines() -> Vec<String> {
    let mut out = Vec::new();
    if let Some(p) = game::get_player() {
        out.push(format!("XYZ: {:.1} {:.1} {:.1}", p.x, p.y, p.z));
    }
    if let Some(c) = game::get_clock() {
        // 24000 ticks per day; tick 0 is 6:00 on the first day.
        let day = c.day_time / 24000 + 1;
        let in_day = c.day_time % 24000;
        let hours = (in_day / 1000 + 6) % 24;
        let minutes = (in_day % 1000) * 60 / 1000;
        out.push(format!("Day {day}  {hours:02}:{minutes:02}"));
    }
    out
}

impl Mod for CoordsDay {
    fn init() -> Result<(Self, Registration), String> {
        Ok((CoordsDay { last: String::new() }, Registration::new().on_frame()))
    }

    fn frame(&mut self, _seconds: f32) {
        let lines = lines();
        let text = lines.join("\n");
        if text == self.last {
            return;
        }
        self.last = text;
        let shapes: Vec<Shape> = lines
            .iter()
            .enumerate()
            .map(|(row, line)| {
                Shape::Text(TextShape {
                    text: line.clone(),
                    at: Point { x: 4.0, y: 4.0 + row as f32 * 10.0 },
                    color: 0xFFFFFF,
                    centered: false,
                    shadow: true,
                })
            })
            .collect();
        render::set_hud(&shapes);
    }
}

// Registers the mod, as PH-FPS does at the end of its lib.rs.
horizon_mod_sdk::export_mod!(CoordsDay);
