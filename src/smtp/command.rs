// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use super::SmtpError;
use crate::utils::validate_email_address;


mod commands {
    #[derive(Debug)]
    pub struct HELO {
        pub sender: String,
    }

    impl HELO {
        pub fn new(sender: String) -> Self {
            Self { sender: sender }
        }
    }


    #[derive(Debug)]
    pub struct EHLO {
        pub sender: String,
    }

    impl EHLO {
        pub fn new(sender: String) -> Self {
            Self { sender: sender }
        }
    }


    #[derive(Debug)]
    pub struct MAIL {
        pub from: String,
    }

    impl MAIL {
        pub fn new(from: String) -> Self {
            Self { from: from }
        }
    }
}


pub enum SmtpCommand {
    HELO(commands::HELO),
    EHLO(commands::EHLO),
    MAIL(commands::MAIL),
    RSET,
    NOOP,
    QUIT,
    Err(SmtpError),
}

impl SmtpCommand {
    pub fn parse(line: &String) -> Self {
        let elm = line.trim().split(" ").collect::<Vec<_>>();
        let elm_len = elm.len();
        let command: &str = &elm[0].to_uppercase();
        match command {
            "HELO" => {
                if elm_len == 2 {
                    Self::HELO(commands::HELO::new(elm[1].to_lowercase()))
                } else {
                    Self::Err(SmtpError::WrongArgument)
                }
            },
            "EHLO" => {
                if elm_len == 2 {
                    Self::EHLO(commands::EHLO::new(elm[1].to_lowercase()))
                } else {
                    Self::Err(SmtpError::WrongArgument)
                }
            },
            "MAIL" => {
                if elm_len == 1 {
                    Self::Err(SmtpError::WrongArgument)
                } else {
                    if elm[1].to_uppercase().starts_with("FROM:<") && elm[1].ends_with(">") {
                        let from = &elm[1][6..elm[1].len() - 1];
                        if validate_email_address(from) {
                            println!("{}", from);
                            if elm_len == 2 {
                                Self::MAIL(commands::MAIL::new(from.to_string()))
                            } else {
                                Self::Err(SmtpError::UnrecognizedMAILParameter)
                            }
                        } else {
                            Self::Err(SmtpError::WrongArgument)
                        }
                    } else {
                        Self::Err(SmtpError::WrongArgument)
                    }
                }
            },
            "RSET" => {
                if elm_len == 1 {
                    Self::RSET
                } else {
                    Self::Err(SmtpError::WrongArgument)
                }
            },
            "NOOP" => Self::NOOP,
            "QUIT" => Self::QUIT,
            _ => SmtpCommand::Err(SmtpError::UnrecognizedCommand),
        }
    }
}

