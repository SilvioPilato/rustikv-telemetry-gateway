use std::io::{self, Read, Write};
use std::net::TcpStream;

use rustikv::bffp::{Command, DecodedResponse, ResponseStatus, decode_response_frame, encode_command};

pub struct Client {
    addr: String,
    stream: TcpStream,
    collection: Option<String>,
}

impl Client {
    pub fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
        Ok(Self {
            addr: addr.to_string(),
            stream,
            collection: None,
        })
    }

    /// After connecting, optionally select a collection. Call this once on
    /// startup (and again after `reconnect`) when `--collection` is configured.
    pub fn use_collection(&mut self, name: &str) -> io::Result<()> {
        self.collection = Some(name.to_string());
        let resp = self.send(Command::Use(name.to_string()))?;
        if matches!(resp.status, ResponseStatus::Error) {
            let msg = resp.payload.join("; ");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("USE {name:?} failed: {msg}"),
            ));
        }
        Ok(())
    }

    /// Send one command (consumed by `encode_command`) and read the decoded response.
    /// On a connection error, reconnects once and retries before giving up.
    pub fn send(&mut self, command: Command) -> io::Result<DecodedResponse> {
        match self.send_once(command.clone()) {
            Ok(resp) => Ok(resp),
            Err(e) => {
                eprintln!("rustikv: send failed ({e}); reconnecting and retrying");
                self.reconnect()?;
                self.send_once(command)
            }
        }
    }

    fn send_once(&mut self, command: Command) -> io::Result<DecodedResponse> {
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

    /// Reconnect after a dropped connection, re-selecting the collection if one was set.
    pub fn reconnect(&mut self) -> io::Result<()> {
        eprintln!("rustikv: reconnecting to {}", self.addr);
        self.stream = TcpStream::connect(&self.addr)?;
        self.stream.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
        eprintln!("rustikv: reconnected");
        if let Some(name) = self.collection.clone() {
            eprintln!("rustikv: re-issuing USE {name:?}");
            let resp = self.send(Command::Use(name.clone()))?;
            if matches!(resp.status, ResponseStatus::Error) {
                let msg = resp.payload.join("; ");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("USE {name:?} failed after reconnect: {msg}"),
                ));
            }
        }
        Ok(())
    }
}
