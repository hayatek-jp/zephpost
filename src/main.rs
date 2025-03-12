// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

mod smtp;

use std::sync::Arc;

use bpaf::{Bpaf};
use sysexits::ExitCode;

use crate::smtp::SMTPServer;


#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
struct Options {
}


#[tokio::main]
async fn main() -> ExitCode {
    // Parse cmdline arguments
    let args = options().run();

    // Build server
    let server = Arc::new(SMTPServer::new());
    // Run server
    server.run()
        .await
        .expect("Failed to start SMTP server");

    return ExitCode::Ok;
}

