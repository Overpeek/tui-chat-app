use clap::Parser;
use eznet::socket::Socket;
use std::{
    net::{Ipv6Addr, SocketAddrV6},
    time::Duration,
};

//

pub mod handler;
pub mod tui;

//

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// TUI update rate in milliseconds between ticks
    #[clap(short, long, default_value_t = 100)]
    tui_tick_rate: u16,

    /// Disable TUI unicode symbols
    #[clap(short = 'u', long)]
    no_unicode: bool,
}

#[tokio::main]
async fn main() {
    let CliArgs {
        tui_tick_rate,
        no_unicode,
    } = CliArgs::parse();

    tui::run(Duration::from_millis(tui_tick_rate as _), no_unicode).unwrap();
    return;

    let addr = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 13331, 0, 0).into();
    let socket = Socket::connect(addr).await;

    match socket {
        Ok(socket) => handler::handler(socket).await,
        Err(err) => eprintln!("{err}"),
    }
}
