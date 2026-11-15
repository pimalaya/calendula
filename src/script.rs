//! # Scripted server
//!
//! A stream answering each request a client writes with the next scripted
//! response, for the tests of the protocol commands: io-webdav, io-gcal
//! and io-msgraph clients take any `Read + Write`.

use std::{
    collections::VecDeque,
    io::{Cursor, Read, Result, Write},
    sync::{Arc, Mutex},
};

/// The requests a [`Script`] received, each as written.
pub type Requests = Arc<Mutex<Vec<Vec<u8>>>>;

/// A server answering each request with the next scripted response.
pub struct Script {
    responses: VecDeque<Vec<u8>>,
    response: Cursor<Vec<u8>>,
    requests: Requests,
    answered: bool,
}

impl Script {
    /// A server answering `responses` in order, and the requests it
    /// receives.
    pub fn new(responses: impl IntoIterator<Item = Vec<u8>>) -> (Self, Requests) {
        let requests = Requests::default();
        let script = Self {
            responses: responses.into_iter().collect(),
            response: Cursor::default(),
            requests: requests.clone(),
            answered: true,
        };

        (script, requests)
    }
}

impl Read for Script {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if !self.answered {
            let response = self.responses.pop_front().unwrap_or_default();
            self.response = Cursor::new(response);
            self.answered = true;
        }

        self.response.read(buf)
    }
}

impl Write for Script {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut requests = self.requests.lock().unwrap();

        if self.answered {
            requests.push(Vec::new());
            self.answered = false;
        }

        requests.last_mut().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// An HTTP/1.1 response of `status` carrying `body` as `content_type`.
pub fn response(status: u16, content_type: &str, body: &str) -> Vec<u8> {
    let len = body.len();

    format!(
        "HTTP/1.1 {status} Scripted\r\nContent-Type: {content_type}\r\n\
         Content-Length: {len}\r\n\r\n{body}"
    )
    .into_bytes()
}
