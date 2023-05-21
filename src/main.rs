use game::{local, multiplayer as network, session};
use sioux_rust_minesweeper_crate::*;

fn convert_config(c: args::Config) -> session::SessionConfig {
    let args::Config {
        width,
        height,
        lives,
        mines,
    } = c;
    session::SessionConfig {
        coords: coordinations::Coordinations::from_width_and_height(width, height),
        mines: game::Mines(mines),
        lives: game::Lives(lives),
    }
}

pub fn main() {
    let args = args::parse_args();
    println!("{args}");
    match args {
        args::Modus::Local(config) => {
            let config = convert_config(config);
            adapter::Main::new(local::create_manager(config)).exec()
        }
        args::Modus::Slave {
            name,
            slave,
            master,
        } => {
            let slave = network::slave::Slave(slave);
            let master = network::slave::Master(master);
            adapter::Main::new(network::slave::Manager::new(name, slave, master)).exec()
        }
        args::Modus::Master { name, config, port } => {
            let master = {
                let ip = std::net::Ipv4Addr::UNSPECIFIED;
                let ip = std::net::IpAddr::V4(ip);
                std::net::SocketAddr::new(ip, port)
            };
            let config = convert_config(config);
            adapter::Main::new(network::master::Manager::new(name, master, config)).exec()
        }
    }
}
