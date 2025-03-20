// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;


#[derive(Debug, Error)]
pub enum SmtpError {
    #[error("421 {0} Service not available, closing transmission channel")]
    Shutdown(String),
    #[error("500 SyntaxError: Command unrecognized")]
    UnrecognizedCommand,
    #[error("501 SyntaxError: Wrong parameters or arguments")]
    WrongArgument,
    #[error("503 LogicalError: Bad sequence of commands")]
    BadSequence,
    #[error("555 SyntaxError: MAIL FROM parameters unrecognized")]
    UnrecognizedMAILParameter,
    #[allow(dead_code)]
    #[error("555 SyntaxError: MAIL FROM parameters not implemented")]
    UnimplementedMAILParameter,
}

