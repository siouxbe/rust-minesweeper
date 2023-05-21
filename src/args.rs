use clap::{Arg, ArgAction, ArgMatches, Command};

static SUBCMD_LOCAL: &str = "local";
static SUBCMD_MASTER: &str = "master";
static SUBCMD_SLAVE: &str = "slave";

static ARG_WIDTH: &str = "width";
static ARG_WIDTH_DEFAULT_STR: &str = "8";
static ARG_HEIGHT: &str = "height";
static ARG_HEIGHT_DEFAULT_STR: &str = "10";
static ARG_MINES: &str = "mines";
static ARG_MINES_DEFAULT_STR: &str = "14";
static ARG_LIVES: &str = "lives";
static ARG_LIVES_DEFAULT_STR: &str = "3";

static ARG_MASTERIP: &str = "masterip";
static ARG_MASTERPORT: &str = "masterport";
static ARG_SLAVEIP: &str = "slaveip";
static ARG_SLAVEPORT: &str = "slaveport";

static ARG_NAME: &str = "name";

/*
 * TODO: Config can not be formatted yet.
 * Complete the code so that it implements both 'std::fmt::Debug' and 'std::fmt::Display'
 * and passes the unit tests.
 * The unit tests require that two lines are printed.
 * Complete the exercise first by adding a '\n' to the text string.
 * Then try to solve the exercise without explicitly adding '\n' to the text string.
 */
pub struct Config {
    pub width: u32,
    pub height: u32,
    pub mines: u32,
    pub lives: u32,
}

pub enum Modus {
    Local(Config),
    Slave {
        name: String,
        slave: std::net::SocketAddr,
        master: std::net::SocketAddr,
    },
    Master {
        name: String,
        port: u16,
        config: Config,
    },
}

impl std::fmt::Display for Modus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(config) => {
                write!(fmt, "Local singleplayer game. Config:\n{config}")
            }
            Self::Slave {
                name,
                slave,
                master,
            } => {
                writeln!(
                    fmt,
                    "Multiplayer game:\nYou are {name}, connecting from {slave} to {master}"
                )
            }
            Self::Master { name, port, config } => {
                write!(fmt, "Multiplayer game:\nYou are {name}, listening at port {port}. Config:\n{config}")
            }
        }
    }
}

fn build_app() -> ArgMatches {
    let width = Arg::new(ARG_WIDTH)
        .long("width")
        .value_name("WIDTH")
        .default_value(ARG_WIDTH_DEFAULT_STR)
        .value_parser(parse_u32)
        .action(ArgAction::Set);
    let height = Arg::new(ARG_HEIGHT)
        .long("height")
        .value_name("HEIGHT")
        .default_value(ARG_HEIGHT_DEFAULT_STR)
        .value_parser(parse_u32)
        .action(ArgAction::Set);
    let mines = Arg::new(ARG_MINES)
        .long("mines")
        .value_name("MINES")
        .default_value(ARG_MINES_DEFAULT_STR)
        .value_parser(parse_u32)
        .action(ArgAction::Set);
    let lives = Arg::new(ARG_LIVES)
        .long("lives")
        .value_name("LIVES")
        .default_value(ARG_LIVES_DEFAULT_STR)
        .value_parser(parse_u32)
        .action(ArgAction::Set);
    let masterip = Arg::new(ARG_MASTERIP)
        .long("masterip")
        .value_name("MASTERIP")
        .required(true)
        .value_parser(parse_ip)
        .action(ArgAction::Set);
    let masterport = Arg::new(ARG_MASTERPORT)
        .long("masterport")
        .value_name("PORT")
        .default_value("5566")
        .value_parser(parse_u16)
        .action(ArgAction::Set);
    let slaveip = Arg::new(ARG_SLAVEIP)
        .long("localip")
        .value_name("LOCALIP")
        .required(true)
        .value_parser(parse_ip)
        .action(ArgAction::Set);
    let slaveport = Arg::new(ARG_SLAVEPORT)
        .long("localport")
        .value_name("PORT")
        .default_value("5567")
        .value_parser(parse_u16)
        .action(ArgAction::Set);
    let name = Arg::new(ARG_NAME)
        .long("name")
        .value_name("NAME")
        .required(true);

    let local = Command::new(SUBCMD_LOCAL)
        .about("Run a singleplayer game")
        .arg(width.clone())
        .arg(height.clone())
        .arg(mines.clone())
        .arg(lives.clone());
    let slave = Command::new(SUBCMD_SLAVE)
        .about("Setup a multiplayer game server")
        .arg(name.clone())
        .arg(slaveip)
        .arg(slaveport)
        .arg(masterip)
        .arg(masterport.clone());
    let master = Command::new(SUBCMD_MASTER)
        .about("Join a multiplayer game")
        .arg(name)
        .arg(masterport)
        .arg(width)
        .arg(height)
        .arg(mines)
        .arg(lives);

    Command::new("sioux-rust-minesweeper")
        .version("1.0")
        .about("Rust training application")
        .subcommand(local)
        .subcommand(master)
        .subcommand(slave)
        .get_matches()
}

fn parse_config(m: &ArgMatches) -> Config {
    Config {
        width: *m.get_one(ARG_WIDTH).unwrap(),
        height: *m.get_one(ARG_WIDTH).unwrap(),
        lives: *m.get_one(ARG_LIVES).unwrap(),
        mines: *m.get_one(ARG_MINES).unwrap(),
    }
}

fn parse_name(m: &ArgMatches) -> String {
    m.get_one::<String>(ARG_NAME).unwrap().clone()
}

fn parse_port(m: &ArgMatches, arg: &str) -> u16 {
    *m.get_one(arg).unwrap()
}

pub fn parse_args() -> Modus {
    let matches = build_app();
    if let Some(m) = matches.subcommand_matches(SUBCMD_LOCAL) {
        let config = parse_config(m);
        return Modus::Local(config);
    }
    if let Some(m) = matches.subcommand_matches(SUBCMD_SLAVE) {
        let name = parse_name(m);
        let slave = {
            let ip = *m.get_one::<std::net::IpAddr>(ARG_SLAVEIP).unwrap();
            let port = parse_port(m, ARG_SLAVEPORT);
            std::net::SocketAddr::new(ip, port)
        };
        let master = {
            let ip = *m.get_one::<std::net::IpAddr>(ARG_MASTERIP).unwrap();
            let port = parse_port(m, ARG_MASTERPORT);
            std::net::SocketAddr::new(ip, port)
        };
        return Modus::Slave {
            name,
            slave,
            master,
        };
    }
    if let Some(m) = matches.subcommand_matches(SUBCMD_MASTER) {
        let name = parse_name(m);
        let config = parse_config(m);
        let port = parse_port(m, ARG_MASTERPORT);
        return Modus::Master { name, config, port };
    }
    let config = Config {
        width: parse_u32(ARG_WIDTH_DEFAULT_STR).unwrap(),
        height: parse_u32(ARG_HEIGHT_DEFAULT_STR).unwrap(),
        mines: parse_u32(ARG_MINES_DEFAULT_STR).unwrap(),
        lives: parse_u32(ARG_LIVES_DEFAULT_STR).unwrap(),
    };
    Modus::Local(config)
}

fn parse_u16(arg: &str) -> Result<u16, &'static str> {
    arg.parse().map_err(|_| "Invalid u16")
}

fn parse_u32(arg: &str) -> Result<u32, &'static str> {
    arg.parse().map_err(|_| "Invalid u32")
}

fn parse_ip(arg: &str) -> Result<std::net::IpAddr, &'static str> {
    use std::str::FromStr;
    std::net::Ipv4Addr::from_str(arg)
        .map_err(|_| "Invalid ipv4")
        .map(std::net::IpAddr::V4)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: Config = Config {
        width: 16,
        height: 20,
        mines: 15,
        lives: 3,
    };

    #[test]
    fn config_can_be_debug_formatted() {
        let s = format!("Game starting with {CONFIG:?}!");
        assert_eq!(
            &s,
            "Game starting with Config { width: 16, height: 20, mines: 15, lives: 3 }!"
        );
    }

    #[test]
    fn config_can_be_display_formatted() {
        let s = format!("Game starting with {CONFIG}!");
        assert_eq!(&s, "Game starting with Minefield: width = 16, height = 20.\nThere are 15 mines and players have a combined total of 3 lives.!");
    }

    #[test]
    fn display_config_will_return_err_when_formatter_returns_err_on_first_call() {
        display_config_may_or_may_not_return_err_when_formatter_returns_err_on_nth_call(1)
    }

    #[test]
    fn display_config_will_possibly_return_err_when_formatter_returns_err_on_second_call() {
        display_config_may_or_may_not_return_err_when_formatter_returns_err_on_nth_call(2)
    }

    fn display_config_may_or_may_not_return_err_when_formatter_returns_err_on_nth_call(n: u32) {
        let allow = n - 1;
        let mut f = Formatter::new(allow);
        let s: std::fmt::Result = write!(f, "{CONFIG}");
        if f.times_called() <= allow {
            assert!(s.is_ok());
        } else {
            assert!(s.is_err());
        }
    }

    struct Formatter {
        allow: u32,
        called: u32,
    }

    impl Formatter {
        pub fn new(allow: u32) -> Self {
            Self { allow, called: 0 }
        }

        pub fn write_fmt(&mut self, _args: std::fmt::Arguments<'_>) -> std::fmt::Result {
            self.called += 1;
            if self.allow == 0 {
                Err(std::fmt::Error)
            } else {
                self.allow -= 1;
                Ok(())
            }
        }

        pub fn times_called(&self) -> u32 {
            self.called
        }
    }
}
