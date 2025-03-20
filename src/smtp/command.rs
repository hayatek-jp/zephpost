// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

use super::SMTPError;


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
    pub struct QUIT {}

    impl QUIT {
        pub fn new() -> Self {
            Self {}
        }
    }
}


pub enum SMTPCommand {
    HELO(commands::HELO),
    EHLO(commands::EHLO),
    QUIT(commands::QUIT),
    NOOP,
    Err(SMTPError),
}

impl SMTPCommand {
    pub fn parse(line: &String) -> Self {
        let elm = line.trim().split(" ").collect::<Vec<_>>();
        let command: &str = &elm[0].to_uppercase();
        match command {
            "HELO" => {
                if elm.len() == 2 {
                    Self::HELO(commands::HELO::new(elm[1].to_lowercase()))
                } else {
                    Self::Err(SMTPError::WrongArgument)
                }
            },
            "EHLO" => {
                if elm.len() == 2 {
                    Self::EHLO(commands::EHLO::new(elm[1].to_lowercase()))
                } else {
                    Self::Err(SMTPError::WrongArgument)
                }
            },
            "QUIT" => Self::QUIT(commands::QUIT::new()),
            "NOOP" => Self::NOOP,
            _ => SMTPCommand::Err(SMTPError::UnrecognizedCommand),
        }
    }
}

