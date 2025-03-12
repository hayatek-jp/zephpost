// SPDX-FileCopyrightText: 2025 KATO Hayate <dev@hayatek.jp>
// SPDX-License-Identifier: GPL-3.0-only

mod commands {
    #[derive(Debug)]
    pub struct QUIT {}

    impl QUIT {
        pub fn new() -> Self {
            Self {}
        }
    }
}


pub enum SMTPCommand {
    QUIT(commands::QUIT),
    INVALID,
}

impl SMTPCommand {
    pub fn parse(line: &String) -> Self {
        let elm = line.trim().split(" ").collect::<Vec<_>>();
        match elm[0] {
            "QUIT" => Self::QUIT(commands::QUIT::new()),
            _ => SMTPCommand::INVALID,
        }
    }
}

