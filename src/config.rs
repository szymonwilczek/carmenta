use clap::Parser;

const MIN_WINDOW_WIDTH: i64 = 280;
const MAX_WINDOW_WIDTH: i64 = 1400;
const MIN_WINDOW_HEIGHT: i64 = 320;
const MAX_WINDOW_HEIGHT: i64 = 1400;

#[derive(Debug, Clone, Parser)]
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
}

impl AppConfig {
    pub fn gifs_enabled(&self) -> bool {
        !self.disable_gifs
    }
}
