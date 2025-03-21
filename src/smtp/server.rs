// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::{TcpListener, TcpStream};

use super::{SmtpCommand, SmtpError};


#[derive(Debug)]
struct SmtpSession {
    helo_received: bool,
    sender: Option<String>,
    recipients: Vec<String>,
    data: String,
}

impl SmtpSession {
    pub fn new() -> Self {
        Self {
            helo_received: false,
            sender: None,
            recipients: Vec::new(),
            data: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.sender = None;
        self.recipients.clear();
        self.data.clear();
    }

    pub fn is_helo_received(&self) -> bool {
        self.helo_received
    }

    pub fn set_helo_received(&mut self) {
        if !self.helo_received {
            self.helo_received = true;
        } else {
            self.reset();
            self.helo_received = true;
        }
    }

    pub fn is_sender_set(&self) -> bool {
        self.sender != None
    }

    pub fn get_sender(&self) -> &Option<String> {
        &self.sender
    }

    pub fn set_sender(&mut self, sender: String) {
        if self.sender == None {
            self.sender = Some(sender);
        } else {
            self.reset();
            self.sender = Some(sender);
        }
    }

    pub fn get_recipients(&self) -> &Vec<String> {
        &self.recipients
    }

    pub fn add_recipient(&mut self, recipient: String) {
        self.recipients.push(recipient);
    }

    pub fn get_data(&self) -> &String{
        &self.data
    }

    pub fn append_data(&mut self, buf: &str) {
        self.data.push_str(buf);
    }
}


#[derive(Debug)]
pub struct SmtpServer {
    is_closed: AtomicBool,
    hostname: String,
}

impl SmtpServer {
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

        // SMTP session
        let mut session = SmtpSession::new();

        if let Ok(_) = self.write_stream(writer, format!("220 {} ESMTP ZephPost\r\n", self.hostname)).await {
            loop {
                // check
                if self.is_closed.load(Ordering::Acquire) {
                    if let Err(_) = self.write_stream(writer, format!("{}\r\n", SmtpError::Shutdown(self.hostname.to_string()))).await { break; };
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
                match SmtpCommand::parse(&line) {
                    SmtpCommand::HELO(_) => {
                        session.set_helo_received();
                        if let Err(_) = self.write_stream(writer, format!("250 {}\r\n", self.hostname)).await { break; }
                    },
                    SmtpCommand::EHLO(_) => {
                        session.set_helo_received();
                        if let Err(_) = self.write_stream(writer, format!("250 {}\r\n", self.hostname)).await { break; }
                    },
                    SmtpCommand::MAIL(mail) => {
                        if session.is_helo_received() {
                            session.set_sender(mail.from);
                            if let Err(_) = self.write_stream(writer, "250 OK\r\n").await { break; }
                        } else {
                            let e = SmtpError::BadSequence;
                            eprintln!("Receive error: {}", e);
                            if let Err(_) = self.write_stream(writer, format!("{}\r\n", e)).await { break; }
                        }
                    },
                    SmtpCommand::RCPT(rcpt) => {
                        if session.is_helo_received() && session.is_sender_set() {
                            // TODO: User check
                            session.add_recipient(rcpt.to);
                            if let Err(_) = self.write_stream(writer, "250 OK\r\n").await { break; }
                        } else {
                            let e = SmtpError::BadSequence;
                            eprintln!("Receive error: {}", e);
                            if let Err(_) = self.write_stream(writer, format!("{}\r\n", e)).await { break; }
                        }
                    },
                    SmtpCommand::RSET => {
                        session.reset();
                        if let Err(_) = self.write_stream(writer, "250 OK\r\n").await { break; }
                    },
                    SmtpCommand::NOOP => {
                        if let Err(_) = self.write_stream(writer, "250 OK\r\n").await { break; }
                    },
                    SmtpCommand::QUIT => {
                        let _ = self.write_stream(writer, "221 Bye\r\n").await;
                        break;
                    },
                    SmtpCommand::Err(e) => {
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

