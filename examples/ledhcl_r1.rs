use adir01p::presets::ledhcl_r1;
use clap::{Parser, ValueEnum};
use std::time::Duration;

#[derive(Parser)]
struct Opts {
    #[clap(long, default_value_t = 3)]
    repeat: usize,
    #[clap(value_enum)]
    command: Command,
}

#[derive(Clone, ValueEnum)]
enum Command {
    Power,
    Dimming,
    NightLight,
}

fn main() {
    let opts = Opts::parse();

    let mut device = adir01p::open(Duration::from_millis(200)).unwrap();
    dbg!(device.firmware_version().unwrap());

    let bits = match &opts.command {
        Command::Power => ledhcl_r1::power_ch1(opts.repeat),
        Command::Dimming => ledhcl_r1::dimming_ch1(opts.repeat),
        Command::NightLight => ledhcl_r1::night_light_ch1(opts.repeat),
    };
    device.send(ledhcl_r1::freq(), &bits).unwrap();
}
