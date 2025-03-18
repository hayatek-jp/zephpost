// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;


#[derive(Debug, Error)]
pub enum SMTPError {
    #[error("500 SyntaxError: Command unrecognized")]
    UnrecognizedCommand,
    #[error("501 SyntaxError: Wrong parameters or arguments")]
    WrongArgument,
}

