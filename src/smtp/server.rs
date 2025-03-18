// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::{TcpListener, TcpStream};

use super::SMTPCommand;


#[derive(Debug)]
pub struct SMTPServer {
    is_closed: AtomicBool,
}

impl SMTPServer {
    pub fn new() -> Self {
        Self {
            is_closed: AtomicBool::new(true),
        }
    }

    async fn handle_smtp_connection(&self, stream: TcpStream) {
        let addr = stream.peer_addr().unwrap();
        let (stream_r, mut stream_w) = split(stream);
        let mut reader = BufReader::new(stream_r);
        let mut line = String::new();

        // states
        let mut helo = false;

        if let Err(e) = stream_w.write_all(b"220 smtp.example.com ESMTP ZephPost\r\n").await {
            eprintln!("Send error: {}", e);
        } else {

            loop {
                // check
                if self.is_closed.load(Ordering::Acquire) {
                    if let Err(e) = stream_w.write_all(b"421 smtp.example.com Service not available, closing transmission channel\r\n").await {
                        eprintln!("Send error: {}", e);
                    }
                    break;
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
                        if let Err(e) = stream_w.write_all(b"250 smtp.example.com\r\n").await {
                            eprintln!("Send error: {}", e);
                            break;
                        }
                    },
                    SMTPCommand::EHLO(_) => {
                        helo = true;
                        if let Err(e) = stream_w.write_all(b"250 smtp.example.com\r\n").await {
                            eprintln!("Send error: {}", e);
                            break;
                        }
                    },
                    SMTPCommand::QUIT(_) => {
                        if let Err(e) = stream_w.write_all(b"221 Bye\r\n").await {
                            eprintln!("Send error: {}", e);
                        }
                        break;
                    },
                    SMTPCommand::Err(e) => {
                        eprintln!("Receive error: {}", e);
                        if let Err(e) = stream_w.write_all(format!("{}\r\n", e).as_bytes()).await {
                            eprintln!("Send error: {}", e);
                            break;
                        }
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

