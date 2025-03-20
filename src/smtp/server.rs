// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::{TcpListener, TcpStream};

use super::{SMTPCommand, SMTPError};


#[derive(Debug)]
pub struct SMTPServer {
    is_closed: AtomicBool,
    hostname: String,
}

impl SMTPServer {
    pub fn new() -> Self {
        Self {
            is_closed: AtomicBool::new(true),
            hostname: String::from("smtp.example.com"),
        }
    }

    async fn write_stream<T: AsyncWriteExt + Unpin, U: AsRef<[u8]>>(&self, stream_w: &mut T, message: U) -> io::Result<()> {
        if let Err(e) = stream_w.write_all(message.as_ref()).await {
            // TODO: Improve error handling
            eprintln!("Send error: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }

    async fn handle_smtp_connection(&self, stream: TcpStream) {
        let addr = stream.peer_addr().unwrap();
        let (stream_r, mut stream_w) = split(stream);
        let mut reader = BufReader::new(stream_r);
        let writer = &mut stream_w;
        let mut line = String::new();

        // states
        let mut helo = false;

        if let Ok(_) = self.write_stream(writer, format!("220 {} ESMTP ZephPost\r\n", self.hostname)).await {
            loop {
                // check
                if self.is_closed.load(Ordering::Acquire) {
                    if let Err(_) = self.write_stream(writer, format!("{}\r\n", SMTPError::Shutdown(self.hostname.to_string()))).await { break; };
                }

                // read line
                line.clear();
                let _bytes = match reader.read_line(&mut line).await {
                    Ok(0) => break, // connection closed
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Receive error: {}", e);
                        break;
                    }
                };
                println!("Received: {}", line.trim_end());

                // parse SMTP commands
                match SMTPCommand::parse(&line) {
                    SMTPCommand::HELO(_) => {
                        helo = true;
                        if let Err(_) = self.write_stream(writer, format!("250 {}\r\n", self.hostname)).await { break; }
                    },
                    SMTPCommand::EHLO(_) => {
                        helo = true;
                        if let Err(_) = self.write_stream(writer, format!("250 {}\r\n", self.hostname)).await { break; }
                    },
                    SMTPCommand::MAIL(_) => {
                        if helo {
                            if let Err(_) = self.write_stream(writer, "250 OK\r\n").await { break; }
                        } else {
                            let e = SMTPError::BadSequence;
                            eprintln!("Receive error: {}", e);
                            if let Err(_) = self.write_stream(writer, format!("{}\r\n", e)).await { break; }
                        }
                    },
                    SMTPCommand::NOOP => {
                        if let Err(_) = self.write_stream(writer, "250 OK\r\n").await { break; }
                    },
                    SMTPCommand::QUIT => {
                        let _ = self.write_stream(writer, "221 Bye\r\n").await;
                        break;
                    },
                    SMTPCommand::Err(e) => {
                        eprintln!("Receive error: {}", e);
                        if let Err(_) = self.write_stream(writer, format!("{}\r\n", e)).await { break; }
                    },
                }
            }
        }

        println!("Connection from {} closed", addr);
    }

    pub async fn run(self: Arc<Self>) -> io::Result<()> {
        // configuration
        let port = if !cfg!(debug_assertions) { 25 } else { 2525 };

        // start SMTP server
        // TODO: IPv6
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port)).await?;
        self.is_closed.store(false, Ordering::Release);
        println!("Started SMTP server on port {}", port);

        // listen
        loop {
            if self.is_closed.load(Ordering::Acquire) {
                println!("SMTP server closed");
                break;
            }
            let (stream, addr) = listener.accept().await?;
            println!("Connected from {}", addr);
            let self_ref = Arc::clone(&self);
            tokio::spawn(async move {
                self_ref.handle_smtp_connection(stream).await;
            });
        }

        Ok(())
    }

    pub async fn close(&mut self) {
        self.is_closed.store(true, Ordering::Release);
    }
}

