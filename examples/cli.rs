use clap::{Parser, Subcommand};
use humantime::Duration;

#[derive(Parser)]
struct Opts {
    #[clap(long, default_value = "200ms")]
    timeout: Duration,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Recv(recv::Opts),
    Send(send::Opts),
}

mod recv {
    use adir01p::Device;
    use clap::Parser;
    use humantime::Duration;
    use rusb::GlobalContext;
    use std::io;
    use std::thread;

    #[derive(Parser)]
    pub(super) struct Opts {
        #[clap(long, default_value_t = 38_000)]
        freq: u16,
        #[clap(long, default_value = "5s")]
        wait: Duration,
    }

    pub(super) fn main(opts: &Opts, device: &mut Device<GlobalContext>) {
        let recv = device.recv(opts.freq).unwrap();
        thread::sleep(*opts.wait);
        let bits = recv.finish().unwrap();
        serde_json::to_writer(
            io::stdout().lock(),
            &bits.iter().map(|bit| (bit.on, bit.off)).collect::<Vec<_>>(),
        )
        .unwrap();
    }
}

mod send {
    use adir01p::{Bit, Device};
    use clap::Parser;
    use rusb::GlobalContext;
    use std::io;

    #[derive(Parser)]
    pub(super) struct Opts {
        #[clap(long, default_value_t = 38_000)]
        freq: u16,
    }

    pub(super) fn main(opts: &Opts, device: &mut Device<GlobalContext>) {
        let bits = serde_json::from_reader::<_, Vec<(u16, u16)>>(io::stdin().lock()).unwrap();
        device
            .send(
                opts.freq,
                &bits
                    .iter()
                    .map(|&(on, off)| Bit { on, off })
                    .collect::<Vec<_>>(),
            )
            .unwrap();
    }
}

fn main() {
    let opts = Opts::parse();

    let mut device = adir01p::open(*opts.timeout).unwrap();
    dbg!(device.firmware_version().unwrap());

    match &opts.command {
        Command::Recv(opts) => recv::main(opts, &mut device),
        Command::Send(opts) => send::main(opts, &mut device),
    }
}
