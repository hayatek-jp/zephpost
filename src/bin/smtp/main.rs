// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

mod command;
mod error;
mod server;

#[path = "../../utils.rs"]
mod utils;

use std::sync::Arc;

use sysexits::ExitCode;

use crate::server::SmtpServer;


#[tokio::main]
async fn main() -> ExitCode {
    // Build server
    let server = Arc::new(SmtpServer::new());
    // Run server
    server.run()
        .await
        .expect("Failed to start SMTP server");

    return ExitCode::Ok;
}

