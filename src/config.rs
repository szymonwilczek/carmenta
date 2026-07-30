use clap::Parser;

const MIN_WINDOW_WIDTH: i64 = 280;
const MAX_WINDOW_WIDTH: i64 = 1400;
const MIN_WINDOW_HEIGHT: i64 = 320;
const MAX_WINDOW_HEIGHT: i64 = 1400;
const MIN_SCALE: f64 = 0.5;
const MAX_SCALE: f64 = 4.0;

fn parse_scale(s: &str) -> Result<f64, String> {
    let value: f64 = s
        .parse()
        .map_err(|_| format!("`{s}` is not a valid number"))?;
    if !value.is_finite() || !(MIN_SCALE..=MAX_SCALE).contains(&value) {
        return Err(format!("scale must be between {MIN_SCALE} and {MAX_SCALE}"));
    }
    Ok(value)
}

#[derive(Debug, Clone, Parser, PartialEq)]
#[command(name = "carmenta", about = "Fast emoji picker for Linux", version)]
pub struct AppConfig {
    #[arg(
        long,
        default_value_t = 420,
        value_parser = clap::value_parser!(i32).range(MIN_WINDOW_WIDTH..=MAX_WINDOW_WIDTH),
        help = "Window width in pixels"
    )]
    pub width: i32,

    #[arg(
        long,
        default_value_t = 480,
        value_parser = clap::value_parser!(i32).range(MIN_WINDOW_HEIGHT..=MAX_WINDOW_HEIGHT),
        help = "Window height in pixels"
    )]
    pub height: i32,

    #[arg(
        long = "disable-gifs",
        default_value_t = false,
        help = "Hide GIF tab to reduce resource usage"
    )]
    pub disable_gifs: bool,

    #[arg(
        long = "close-on-select",
        default_value_t = false,
        help = "Close the window automatically after selecting an item"
    )]
    pub close_on_select: bool,

    #[arg(
        long = "prewarm",
        default_value_t = false,
        help = "Start resident in the background without showing the window (warms caches so the first invocation is instant)"
    )]
    pub prewarm: bool,

    #[arg(
        long = "vim",
        default_value_t = false,
        help = "Enable Vim-style navigation: hjkl inside the focused zone, Alt+hjkl between zones"
    )]
    pub vim: bool,

    #[arg(
        long,
        default_value_t = 1.0,
        value_parser = parse_scale,
        help = "UI scale multiplier for emoji/kaomoji/symbols/GIFs (e.g. 1.25 = 125%)"
    )]
    pub scale: f64,
}

impl AppConfig {
    pub fn gifs_enabled(&self) -> bool {
        !self.disable_gifs
    }
}
