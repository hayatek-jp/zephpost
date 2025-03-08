// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use bpaf::{Bpaf};
use sysexits::ExitCode;

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options, version)]
struct Options {
}

fn main() -> ExitCode {
    let args = options().run();
    return ExitCode::Ok;
}

