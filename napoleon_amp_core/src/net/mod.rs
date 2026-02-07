use serbytes::prelude::SerBytes;
use crate::net::server::NapoleonServer;

pub(super)mod server;

#[derive(SerBytes)]
struct NetworkData {
    registered_addresses: Vec<String>,
    priority: u8
}


struct Network {
    data: NetworkData,
    server: NapoleonServer
}