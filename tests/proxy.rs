//! # Proxy
//!
//! The built binary checking an account of one network backend, which
//! reaches its server through the proxy it names: a local HTTP proxy
//! reads the `CONNECT` and refuses it, so nothing goes past it. Moved out
//! of the unit tests, which bind no socket.

#![cfg(any(
    feature = "caldav",
    feature = "jmap",
    feature = "gcal",
    feature = "msgraph"
))]

mod common;

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use tempfile::TempDir;

use crate::common::{MADE_UP, calendula, failure, write_config};

/// A proxy refusing the one tunnel it is asked for, its URL and the
/// request line it read.
fn refusing_proxy() -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (sender, lines) = mpsc::channel();

    thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut stream = BufReader::new(stream);
        let mut line = String::new();
        stream.read_line(&mut line).unwrap();

        let mut header = String::new();
        while stream.read_line(&mut header).unwrap() > 2 {
            header.clear();
        }

        let refusal = b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\n\r\n";
        stream.get_mut().write_all(refusal).unwrap();
        sender.send(line).unwrap();
    });

    (url, lines)
}

/// The request line the proxy read while the binary checked the account
/// made of `lines`, the check failing.
fn tunnel(lines: &str, requests: Receiver<String>) -> String {
    let dir = TempDir::new().unwrap();
    let config = write_config(dir.path(), "a", lines);
    let out = calendula(&config, MADE_UP, &["account", "check"]);

    assert_eq!(out.status.code(), Some(3), "{}", failure(&out));

    requests
        .recv_timeout(Duration::from_secs(10))
        .unwrap_or_else(|err| panic!("{err}: {}", failure(&out)))
}

#[cfg(feature = "caldav")]
#[test]
fn caldav_connects_through_the_account_proxy() {
    let (url, requests) = refusing_proxy();
    let lines = format!(
        "proxy.url = \"{url}\"\ncaldav.home = \"http://calendars.invalid/alice/\"\n\
         caldav.auth = \"none\""
    );

    let line = tunnel(&lines, requests);
    assert!(line.starts_with("CONNECT calendars.invalid:80 "), "{line}");
}

#[cfg(feature = "jmap")]
#[test]
fn jmap_connects_through_the_account_proxy() {
    let (url, requests) = refusing_proxy();
    let lines = format!(
        "proxy.url = \"{url}\"\njmap.server = \"jmap.invalid\"\n\
         jmap.auth.bearer.token.raw = \"made-up\""
    );

    let line = tunnel(&lines, requests);
    assert!(line.starts_with("CONNECT jmap.invalid:443 "), "{line}");
}

#[cfg(feature = "gcal")]
#[test]
fn gcal_connects_through_its_own_proxy() {
    let (url, requests) = refusing_proxy();
    let lines = format!(
        "proxy.url = \"http://127.0.0.1:9\"\ngcal.proxy.url = \"{url}\"\n\
         gcal.auth.token.raw = \"made-up\""
    );

    let line = tunnel(&lines, requests);
    assert!(
        line.starts_with("CONNECT www.googleapis.com:443 "),
        "{line}"
    );
}

#[cfg(feature = "msgraph")]
#[test]
fn msgraph_connects_through_the_account_proxy() {
    let (url, requests) = refusing_proxy();
    let lines = format!("proxy.url = \"{url}\"\nmsgraph.auth.token.raw = \"made-up\"");

    let line = tunnel(&lines, requests);
    assert!(
        line.starts_with("CONNECT graph.microsoft.com:443 "),
        "{line}"
    );
}
