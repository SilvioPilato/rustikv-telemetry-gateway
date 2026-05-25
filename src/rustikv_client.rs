use std::io::{self, Read, Write};
use std::net::TcpStream;

use rustikv::bffp::{Command, DecodedResponse, decode_response_frame, encode_command};

pub struct Client {
    addr: String,
    stream: TcpStream,
}

impl Client {
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self {
            addr: addr.to_string(),
            stream,
        })
    }

    /// Send one command (consumed by `encode_command`) and read the decoded response.
    pub fn send(&mut self, command: Command) -> io::Result<DecodedResponse> {
        let frame = encode_command(command);
        self.stream.write_all(&frame)?;

        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)?;
        let body_len = u32::from_be_bytes(len_buf) as usize;
        let mut body = vec![0u8; body_len];
        self.stream.read_exact(&mut body)?;

        let mut full = Vec::with_capacity(4 + body_len);
        full.extend_from_slice(&len_buf);
        full.extend_from_slice(&body);
        decode_response_frame(&full)
    }

    /// Reconnect after a dropped connection.
    pub fn reconnect(&mut self) -> io::Result<()> {
        self.stream = TcpStream::connect(&self.addr)?;
        Ok(())
    }
}
