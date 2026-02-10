use quinn::rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use quinn::{ConnectionError, Endpoint, Incoming, ServerConfig};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use tokio::runtime::Runtime;

pub(crate) struct NapoleonServer {
    main_thread_handle: JoinHandle<()>,
}

impl NapoleonServer {
    pub(crate) fn new() -> Self {
        let main_thread_handle = thread::spawn(|| {
            let rt = Runtime::new().expect("Create tokio runtime");

            rt.block_on(server_main_thread());
        });

        Self { main_thread_handle }
    }
}

struct AliveStream {
    stream: TcpStream,
    packet_data: Vec<u8>,
    packet_len_rem: usize,
}

async fn server_main_thread() {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7124);

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());

    let mut server_config = ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())
        .expect("Create server config with certification");
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    let endpoint = Endpoint::server(server_config, server_addr).expect("Create server endpoint");

    loop {
        let incoming = if let Some(incoming) = endpoint.accept().await {
            incoming
        } else {
            break;
        };

        tokio::spawn(handle_connection(incoming));
    }
}

async fn handle_connection(incoming: Incoming) -> Result<(), ConnectionError> {
    let connection = incoming.await?;
    loop {
        let (send_stream, recv_stream) = match connection.accept_bi().await {
            Err(ConnectionError::ApplicationClosed { .. }) => {
                println!("Client disconnected");
                return Ok(());
            }

            Err(e) => {
                return Err(e);
            }

            Ok(s) => s,
        };

        let req = recv_stream.read_to_end()
    }
}
