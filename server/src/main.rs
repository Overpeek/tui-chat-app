#![feature(const_socketaddr)]
#![feature(try_blocks)]
#![feature(duration_constants)]

//

use clap::{Parser, ValueEnum};
use dashmap::DashSet;
use eznet::listener::Listener;
use std::{
    fmt::{self, Display, Formatter},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6},
    sync::Arc,
};
use tokio::sync::broadcast::channel;

//

pub mod handler;

//

pub static DEFAULT_ADDRESS: SocketAddr =
    SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 13331, 0, 0));

//

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Server listen address (IPv4 or IPv6)
    ///
    /// Note: you might have to surround IPv6 addresses with '' or ""
    ///
    /// Examples:
    /// - 127.0.0.1:1234
    /// - [::1]:1234
    /// - 0.0.0.0:1234
    /// - [::]:1234
    #[clap(short, long, value_name = "ADDRESS", default_value_t = DEFAULT_ADDRESS)]
    listen: SocketAddr,

    /// User interface method
    #[clap(short, long, default_value_t = Method::Quiet)]
    method: Method,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Method {
    /// Terminal User Interface
    Tui,

    /// Prompts to config stuff
    Prompts,

    /// Use cli arguments and be quiet
    Quiet,
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_possible_value()
                .as_ref()
                .map(|s| s.get_name())
                .unwrap_or("<none>")
        )
    }
}

//

#[tokio::main]
async fn main() {
    // parse cli
    let CliArgs { listen, method } = CliArgs::parse();

    match method {
        Method::Tui => {}
        Method::Prompts => {}
        Method::Quiet => {}
    }

    // start listening for connections
    let mut listener = Listener::bind(listen);

    let connections = Arc::new(DashSet::new());

    let (send, recv) = channel(256);
    let send = Arc::new(send);

    while let Some(conn) = listener.next().await {
        let send = send.clone();
        let recv = recv.resubscribe();

        tokio::spawn(handler::handler(conn, connections.clone(), send, recv));
    }
}
