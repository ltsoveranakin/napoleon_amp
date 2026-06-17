use crate::net::server::NapoleonServer;
use serbytes::prelude::SerBytes;
use std::cell::LazyCell;

mod packet;
pub(super) mod server;

#[derive(SerBytes)]
struct NetworkData {
    registered_addresses: Vec<String>,
    priority: u8,
}

struct Network {
    data: LazyCell<NetworkData>,
    server: NapoleonServer,
}
