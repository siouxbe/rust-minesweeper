use super::*;

pub trait SlaveListener {
    fn on_request_to_join(&mut self, request: RequestFromSlave, addr: std::net::SocketAddr);
    fn on_action_from_slave(&mut self, action: ActionFromSlave, addr: std::net::SocketAddr);
}

pub trait MPLocalPlayerListener {
    fn on_left_click(&mut self, coord: &Coord) -> Option<Updates>;
    fn on_right_click(&mut self, coord: &Coord) -> Option<Updates>;
}
