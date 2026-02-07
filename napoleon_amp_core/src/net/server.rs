use serbytes::prelude::{ReadByteBufferRefMut, SerBytes};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub(crate) struct NapoleonServer {
    main_thread_handle: JoinHandle<()>,
}

impl NapoleonServer {
    pub(crate) fn new() -> Self {
        let main_thread_handle = thread::spawn(server_main_thread);

        Self { main_thread_handle }
    }
}

struct AliveStream {
    stream: TcpStream,
    packet_data: Vec<u8>,
    packet_len_rem: usize,
}

fn server_main_thread() {
    let listener = TcpListener::bind("127.0.0.1:7124").expect("Unable to bind TcpListener to port");

    let mut alive_streams = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                alive_streams.push(AliveStream {
                    stream,
                    packet_data: Vec::new(),
                    packet_len_rem: 0,
                });
            }

            Err(e) => {
                eprintln!("Error connecting to new incoming stream: {}", e);
            }
        }
    }

    loop {
        let mut buf = [0; 1024];

        for AliveStream {
            stream,
            packet_len_rem,
            packet_data,
        } in alive_streams.iter_mut()
        {
            let bytes_read = match stream.read(&mut buf) {
                Ok(bytes_read) => bytes_read,

                Err(e) => {
                    panic!("Some stream error, should probably drop stream... {}", e)
                }
            };

            let mut buf_slice = &mut buf;

            if *packet_len_rem == 0 {
                // First read, extract the first 2 bytes as packet size
                // let rbb = ReadByteBufferRefMut::from_bytes(&mut buf, &mut 0, &mut 0);

                *packet_len_rem = u16::from_bytes(&buf_slice);
            }

            if packet_len_rem < bytes_read {
                // read more bytes than what was remaining on the last packet, therefore we have a fully constructed packet
            }

            for _ in 0..bytes_read {}
        }

        thread::sleep(Duration::from_millis(10))
    }
}
