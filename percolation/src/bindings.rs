use crate::{
    animation::{
        canvas_size, convert, draw_graph, draw_percolation, GRAPH_HEIGHT, GRAPH_WIDTH, TILE_TIME,
    },
    percolation::{Percolatable, Percolation},
    percolationstats::{calculate_stats, PercolationStats},
    random::Random,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn console_log_str(s: &str);

    // fun fact: all javascript numbers are f64 under the hood (except bigint)
    // canvas can handle floating point coords so might as well use them
    fn set_canvas_size(width: f64, height: f64);
    fn request_animation_frame();
    pub fn draw_rectangle(x: f64, y: f64, width: f64, height: f64, color: &str);
    pub fn draw_text(text: &str, x: f64, y: f64, color: &str, font: &str);
    pub fn set_bottom_text(text: &str);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn console_log_str(s: &str) {
    println!("{}", s);
}

macro_rules! log {
    ($($t:tt)*) => (console_log_str(&format_args!($($t)*).to_string()))
}
pub(crate) use log; // make log macro public

#[wasm_bindgen]
pub struct Visualizer {
    mode: VisualizationMode,
    rand: Random,
}

enum VisualizationMode {
    Interactive(Percolation),
    Picture {
        percolation: Percolation,
        tiles: Vec<(usize, usize)>,
        animation_progress: u8,
    },
    Stats(PercolationStats),
}

#[wasm_bindgen]
impl Visualizer {
    pub fn new(width: usize, height: usize, seed: &str) -> Self {
        log!("New Visualizer created! Defaulting to interactive mode");
        let v = Self {
            mode: VisualizationMode::Interactive(Percolation::new(width, height)),
            rand: Random::new(seed),
        };
        after_start(width, height);
        return v;
    }

    pub fn start_interactive(&mut self, width: usize, height: usize) {
        self.mode = VisualizationMode::Interactive(Percolation::new(width, height));
        after_start(width, height);
    }

    pub fn start_picture(&mut self, input: &str) {
        if let Some((w, h, m)) = parse_input(input) {
            self.mode = m;
            after_start(w, h);
        }
    }

    pub fn start_stats(&mut self, width: usize, height: usize, trials: usize) {
        set_canvas_size(GRAPH_WIDTH, GRAPH_HEIGHT);
        set_bottom_text("Calculating...");
        match calculate_stats::<Percolation>(width, height, trials, &mut self.rand) {
            Ok(stats) => {
                self.mode = VisualizationMode::Stats(stats);
                set_bottom_text("Done");
                request_animation_frame();
            }
            Err(_) => set_bottom_text("Error calculating stats"),
        }
    }

    pub fn respond_to_mousedown(&mut self, x: f64, y: f64) {
        if let VisualizationMode::Interactive(percolation) = &mut self.mode {
            let (w, h) = (percolation.width(), percolation.height());
            if let Some((row, col)) = convert(x, y, w, h) {
                if let Ok(open) = percolation.is_open(row, col) {
                    if !open {
                        percolation.open(row, col).expect("out of bounds in open");
                        request_animation_frame();
                    }
                }
            }
        }
    }

    pub fn draw_animation_frame(&mut self) {
        match &mut self.mode {
            VisualizationMode::Interactive(percolation) => draw_percolation(percolation),
            VisualizationMode::Stats(stats) => draw_graph(stats),
            VisualizationMode::Picture {
                percolation,
                tiles,
                animation_progress,
            } => {
                *animation_progress += 1;
                if *animation_progress == TILE_TIME {
                    draw_percolation(percolation);
                    *animation_progress = 0;
                    if let Some((row, col)) = tiles.pop() {
                        percolation.open(row, col).expect("out of bounds in open");
                        request_animation_frame();
                    }
                } else {
                    request_animation_frame();
                }
            }
        }
    }
}

fn after_start(width: usize, height: usize) {
    let (canvas_width, canvas_height) = canvas_size(width, height);
    set_canvas_size(canvas_width, canvas_height);
    request_animation_frame();
}

fn parse_input(input: &str) -> Option<(usize, usize, VisualizationMode)> {
    let mut lines = input.lines();
    let (width, height) = parse_line(lines.next()?)?;
    let mut tiles = lines
        .map(|a| parse_line(a))
        .collect::<Option<Vec<(usize, usize)>>>()?;
    tiles.reverse();

    Some((
        width,
        height,
        VisualizationMode::Picture {
            percolation: Percolation::new(width, height),
            tiles,
            animation_progress: 0,
        },
    ))
}

fn parse_line(line: &str) -> Option<(usize, usize)> {
    let mut tokens = line.trim().split_whitespace();
    let a: usize = tokens.next()?.parse().ok()?;
    let b: usize = tokens.next()?.parse().ok()?;
    if tokens.next().is_none() {
        Some((a, b))
    } else {
        None
    }
}
