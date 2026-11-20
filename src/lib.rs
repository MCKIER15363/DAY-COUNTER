use horizon_mod_sdk::{
    Mod, Registration, game,
    render::{self, FillShape, Point, Rect, Shape, TextShape},
};

// Distance from the bottom edge of the screen to the bottom of the text box,
// in GUI pixels. The hotbar, hearts and XP bar fill roughly the lowest 55;
// raise this if the text still touches them, lower it to sit closer.
const BOTTOM: f32 = 64.0;

struct CoordsDay {
    // What was drawn last, so the screen is only updated when something changes.
    last: (String, u32, u32),
    // The widest the text has been, so the background doesn't jitter.
    widest: f32,
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
        Ok((
            CoordsDay { last: (String::new(), 0, 0), widest: 0.0 },
            Registration::new().on_frame(),
        ))
    }

    fn frame(&mut self, _seconds: f32) {
        let lines = lines();
        let (w, h) = render::gui_size();
        let key = (lines.join("\n"), w as u32, h as u32);
        if key == self.last {
            return;
        }
        self.last = key;
        if lines.is_empty() {
            render::set_hud(&[]);
            return;
        }

        let pad = 3.0;
        let line_h = 10.0;
        let text_w = lines.iter().map(|l| render::text_width(l)).fold(0.0, f32::max);
        self.widest = self.widest.max(text_w);
        let box_w = self.widest + pad * 2.0;
        let box_h = lines.len() as f32 * line_h + pad * 2.0 - 1.0;

        // Centered, just above the hotbar and its status bars.
        let left = ((w - box_w) / 2.0).max(0.0);
        let top = (h - box_h - BOTTOM).max(0.0);

        let mut shapes = Vec::new();
        // Dark see-through background (0xAARRGGBB: about 50% black).
        shapes.push(Shape::Fill(FillShape {
            rect: Rect { x: left, y: top, w: box_w, h: box_h },
            color: 0x80000000,
        }));
        for (row, line) in lines.iter().enumerate() {
            shapes.push(Shape::Text(TextShape {
                text: line.clone(),
                at: Point { x: left + pad, y: top + pad + row as f32 * line_h },
                color: 0xFFFFFF,
                centered: false,
                shadow: true,
            }));
        }
        render::set_hud(&shapes);
    }
}

// Registers the mod, as PH-FPS does at the end of its lib.rs.
horizon_mod_sdk::export_mod!(CoordsDay);
