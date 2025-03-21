// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use bpaf::{Bpaf};
use sysexits::ExitCode;
use tokio::process::Command;


#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
struct Options {
}


#[tokio::main]
async fn main() -> ExitCode {
    // Parse cmdline arguments
    let args = options().run();

    // components
    let smtp = if !cfg!(debug_assertions) { "../libexec/zephpost/smtp" } else { "./target/debug/smtp" };

    // Run smtp
    Command::new(smtp).spawn().unwrap();

    loop {}

    return ExitCode::Ok;
}

