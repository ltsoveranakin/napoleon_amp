use crate::net::packet::NapoleonPacket;
use renet::{ClientId, ConnectionConfig, DefaultChannel, RenetServer, ServerEvent};
use renet_netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use serbytes::prelude::{ReadByteBufferSlice, SerBytes};
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

const SERVER_PORT: u16 = 5562;

pub(crate) struct NapoleonServer {
    main_thread_handle: JoinHandle<()>,
}

impl NapoleonServer {
    pub(super) fn new() -> Self {
        Self {
            main_thread_handle: thread::spawn(Self::server_thread),
        }
    }

    fn handle_packet(packet: NapoleonPacket) -> Option<NapoleonPacket> {}

    fn server_thread() {
        let mut server = RenetServer::new(ConnectionConfig::default());
        let mut outgoing_packets = VecDeque::new();

        // Setup transport layer using renet_netcode
        const SERVER_ADDR: SocketAddr =
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), SERVER_PORT);
        let socket: UdpSocket = UdpSocket::bind(SERVER_ADDR).unwrap();
        let server_config = ServerConfig {
            current_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            max_clients: 64,
            protocol_id: 0,
            public_addresses: vec![SERVER_ADDR],
            authentication: ServerAuthentication::Unsecure,
        };
        let mut transport = NetcodeServerTransport::new(server_config, socket).unwrap();

        // Your gameplay loop
        loop {
            let delta_time = Duration::from_millis(10);
            // Receive new messages and update clients
            server.update(delta_time);
            transport.update(delta_time, &mut server)?;

            // Check for client connections/disconnections
            while let Some(event) = server.get_event() {
                match event {
                    ServerEvent::ClientConnected { client_id } => {
                        println!("Client {client_id} connected");
                    }
                    ServerEvent::ClientDisconnected { client_id, reason } => {
                        println!("Client {client_id} disconnected: {reason}");
                    }
                }
            }

            // Receive message from channel
            for client_id in server.clients_id() {
                // The enum DefaultChannel describe the channels used by the default configuration

                while let Some(message) =
                    server.receive_message(client_id, DefaultChannel::ReliableOrdered)
                {
                    let mut rbb = ReadByteBufferSlice::new(&message).rbb_ref_mut();

                    match NapoleonPacket::from_buf(&mut rbb) {
                        Ok(packet) => {
                            Self::handle_packet(packet);
                        }

                        Err(e) => {}
                    }
                }
            }

            // Send a text message for all clients
            server.broadcast_message(DefaultChannel::ReliableOrdered, "server message");

            let client_id: ClientId = 0;
            // Send a text message for all clients except for Client 0
            server.broadcast_message_except(
                client_id,
                DefaultChannel::ReliableOrdered,
                "server message",
            );

            // Send message to only one client
            server.send_message(client_id, DefaultChannel::ReliableOrdered, "server message");

            // Send packets to clients using the transport layer
            transport.send_packets(&mut server);

            std::thread::sleep(delta_time); // Running at 60hz
        }
    }
}
