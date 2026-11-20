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
    scale: u32,
    x: u32,
    y: u32,
    opacity: u32,
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
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_owned(), v.trim().to_owned()))
        })
        .collect()
}

fn stored(cfg: &[(String, String)], key: &str, default: u32, min: u32, max: u32) -> u32 {
    cfg.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.parse::<u32>().ok())
        .map_or(default, |v| v.clamp(min, max))
}

fn lines(coords: bool, day: bool) -> Vec<String> {
    let mut out = Vec::new();
    if coords {
        if let Some(p) = game::get_player() {
            out.push(format!("XYZ: {:.1} {:.1} {:.1}", p.x, p.y, p.z));
        }
    }
    if day {
        if let Some(c) = game::get_clock() {
            let d = c.day_time / 24000 + 1;
            let in_day = c.day_time % 24000;
            let hours = (in_day / 1000 + 6) % 24;
            let minutes = (in_day % 1000) * 60 / 1000;
            out.push(format!("Day {d}  {hours:02}:{minutes:02}"));
        }
    }
    out
}

fn button(id: u32, label: String) -> Widget {
    Widget::Button(ButtonWidget { id, label })
}

fn slider(id: u32, label: String, steps: u32, position: u32) -> Widget {
    Widget::Slider(SliderWidget { id, label, steps, position })
}

impl CoordsDay {
    fn scale(&self) -> f32 {
        self.scale as f32 / 100.0
    }

    fn slider_label(&self, id: u32) -> (String, u32, u32) {
        match id {
            S_SIZE => (
                format!("Size: {:.2}x", self.scale()),
                (SCALE_MAX - SCALE_MIN) / SCALE_STEP,
                (self.scale - SCALE_MIN) / SCALE_STEP,
            ),
            S_X => (format!("Horizontal position: {:.1}%", self.x as f32 / 10.0), 200, self.x / 5),
            S_Y => (format!("Vertical position: {:.1}%", self.y as f32 / 10.0), 200, self.y / 5),
            _ => (format!("Background opacity: {}%", self.opacity), 100, self.opacity),
        }
    }

    fn slider_widget(&self, id: u32) -> Widget {
        let (label, steps, position) = self.slider_label(id);
        slider(id, label, steps, position)
    }

    fn page(&self) {
        ui::set_settings_v2(&[
            button(SHOW, format!("Show: {}", on_off(self.show))),
            button(COORDS, format!("Coordinates: {}", on_off(self.coords))),
            button(DAY, format!("Day and time: {}", on_off(self.day))),
            button(BACKGROUND, format!("Background: {}", on_off(self.background))),
            button(MOVE, "Move with the mouse or finger...".into()),
            self.slider_widget(S_SIZE),
            self.slider_widget(S_X),
            self.slider_widget(S_Y),
            self.slider_widget(S_OPACITY),
            button(RESET, "Reset to defaults".into()),
        ]);
    }

    fn defaults(&mut self) {
        self.show = true;
        self.background = true;
        self.coords = true;
        self.day = true;
        self.scale = 100;
        self.x = 500;
        self.y = 850;
        self.opacity = 50;
    }

    fn save(&self) {
        let text = format!(
            "show={}\nbackground={}\ncoords={}\nday={}\nscale={}\nx={}\ny={}\nopacity={}\n",
            u32::from(self.show),
            u32::from(self.background),
            u32::from(self.coords),
            u32::from(self.day),
            self.scale,
            self.x,
            self.y,
            self.opacity,
        );
        let _ = storage::set(FILE, text.as_bytes());
    }

    fn redraw(&mut self) {
        self.drawn = false;
        self.last = (String::new(), 0, 0);
    }

    fn draw(&mut self) {
        if !self.show {
            if self.drawn {
                render::set_hud(&[]);
                render::set_hud_drag(None);
                self.drawn = false;
            }
            return;
        }
        let lines = lines(self.coords, self.day);
        let (gw, gh) = render::gui_size();
        let key = (lines.join("\n"), gw as u32, gh as u32);
        if self.drawn && key == self.last {
            return;
        }
        self.last = key;
        self.drawn = true;
        if lines.is_empty() {
            render::set_hud(&[]);
            render::set_hud_drag(None);
            return;
        }

        let scale = self.scale();
        let (w, h) = (gw / scale, gh / scale);
        let pad = 3.0;
        let line_h = 10.0;
        let text_w = lines.iter().map(|l| render::text_width(l)).fold(0.0, f32::max);
        self.widest = self.widest.max(text_w);
        let box_w = self.widest + pad * 2.0;
        let box_h = lines.len() as f32 * line_h + pad * 2.0 - 1.0;
        let left = (w - box_w).max(0.0) * self.x as f32 / 1000.0;
        let top = (h - box_h).max(0.0) * self.y as f32 / 1000.0;

        let mut shapes = Vec::new();
        if self.background {
            shapes.push(Shape::Fill(FillShape {
                rect: Rect { x: left, y: top, w: box_w, h: box_h },
                color: (self.opacity * 255 / 100) << 24,
            }));
        }
        for (row, line) in lines.iter().enumerate() {
            shapes.push(Shape::Text(TextShape {
                text: line.clone(),
                at: Point { x: left + pad, y: top + pad + row as f32 * line_h },
                color: 0xFFFFFF,
                centered: false,
                shadow: true,
            }));
        }
        render::set_hud_scale(scale);
        render::set_hud_always(true);
        render::set_hud(&shapes);
        render::set_hud_drag(Some(Rect { x: left, y: top, w: box_w, h: box_h }));
        self.area = (left, top, box_w, box_h, w, h);
    }

    fn dragged(&mut self) {
        let Some(d) = render::take_hud_drag() else { return };
        let (left, top, box_w, box_h, w, h) = self.area;
        let left = (left + d.dx).clamp(0.0, (w - box_w).max(0.0));
        let top = (top + d.dy).clamp(0.0, (h - box_h).max(0.0));
        let span_x = (w - box_w).max(1.0);
        let span_y = (h - box_h).max(1.0);
        self.x = ((left / span_x) * 1000.0).round().clamp(0.0, 1000.0) as u32;
        self.y = ((top / span_y) * 1000.0).round().clamp(0.0, 1000.0) as u32;
        self.redraw();
        if d.done {
            self.save();
            self.page();
        }
    }
}

impl Mod for CoordsDay {
    fn init() -> Result<(Self, Registration), String> {
        let cfg = load_config();
        let mut m = CoordsDay {
            show: stored(&cfg, "show", 1, 0, 1) == 1,
            background: stored(&cfg, "background", 1, 0, 1) == 1,
            coords: stored(&cfg, "coords", 1, 0, 1) == 1,
            day: stored(&cfg, "day", 1, 0, 1) == 1,
            scale: stored(&cfg, "scale", 100, SCALE_MIN, SCALE_MAX) / SCALE_STEP * SCALE_STEP,
            x: stored(&cfg, "x", 500, 0, 1000) / 5 * 5,
            y: stored(&cfg, "y", 850, 0, 1000) / 5 * 5,
            opacity: stored(&cfg, "opacity", 50, 0, 100),
            last: (String::new(), 0, 0),
            drawn: false,
            widest: 0.0,
            area: (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        };
        m.scale = m.scale.max(SCALE_MIN);
        m.save();
        m.page();
        Ok((m, Registration::new().on_frame()))
    }

    fn frame(&mut self, _seconds: f32) {
        self.dragged();
        self.draw();
    }

    fn ui_select(&mut self, id: u32, row: u32) {
        let slid = match id {
            S_SIZE => {
                self.scale = SCALE_MIN + row * SCALE_STEP;
                true
            }
            S_X => {
                self.x = (row * 5).min(1000);
                true
            }
            S_Y => {
                self.y = (row * 5).min(1000);
                true
            }
            S_OPACITY => {
                self.opacity = row.min(100);
                true
            }
            _ => false,
        };
        if slid {
            let (label, _, position) = self.slider_label(id);
            ui::set_slider(id, position, &label);
            self.redraw();
            self.save();
            return;
        }
        match id {
            SHOW => self.show = !self.show,
            COORDS => self.coords = !self.coords,
            DAY => self.day = !self.day,
            BACKGROUND => self.background = !self.background,
            RESET => self.defaults(),
            MOVE => {
                ui::open_screen(&ui::Screen {
                    title: "Coords and Day".into(),
                    widgets: vec![
                        ui::Widget::Text(ui::TextWidget {
                            id: 1,
                            text: "Drag the text where you want it, then press Done".into(),
                            color: 0xB98CF2,
                        }),
                        ui::Widget::Button(ButtonWidget { id: DONE, label: "Done".into() }),
                    ],
                });
                return;
            }
            DONE => {
                ui::close_screen();
                return;
            }
            _ => return,
        }
        self.redraw();
        self.save();
        self.page();
    }

    fn ui_settings_opened(&mut self) {
        self.page();
    }
}

horizon_mod_sdk::export_mod!(CoordsDay);
