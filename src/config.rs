use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Config {
    /// Name of image to open
    pub file_name: String,

    /// Wether to scale the image up
    #[clap(short, long, takes_value = false)]
    pub up_scale: bool,

    /// Whether to force integrated gpu
    #[clap(short, long, takes_value = false)]
    pub low_performance_mode: bool,
}
