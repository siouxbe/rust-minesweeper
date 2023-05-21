mod messages;

use super::*;

use messages as msg;

use std::net::UdpSocket;

const SOCKET_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(5);
const BUFFER_FIXED_SIZE: usize = 1 << 22;

pub enum MessageSent {
    Yes,
    No,
}

pub struct MessengerBuffer(Vec<u8>);

pub struct Messenger {
    socket: UdpSocket,
}

impl Messenger {
    pub fn new(socket: UdpSocket) -> Self {
        socket
            .set_read_timeout(Some(SOCKET_TIMEOUT))
            .expect("Failed to set socket read timeout");
        socket
            .set_write_timeout(Some(SOCKET_TIMEOUT))
            .expect("Failed to set socket read timeout");
        Self { socket }
    }

    #[allow(unused_variables)]
    pub fn request_to_join(
        &self,
        buffer: &mut MessengerBuffer,
        player_name: &str,
    ) -> UpdateFromMaster {
        let msg = msg::from_slave::JoinRequest {
            name: player_name.into(),
        };
        let msg = msg::from_slave::MessageSentByClient;
        loop {
            println!("Sending a join request");
            if let MessageSent::Yes = self.send_packet_from_slave(buffer, &msg).unwrap() {
                if let Some(update) = self.receive_packet_from_master(buffer) {
                    println!("Received a join reply");
                    return update;
                }
            }
        }
    }

    pub fn left_click(&self, buffer: &mut MessengerBuffer, session: SessionID, coord: &Coord) {
        let left = true;
        self.click(buffer, session, coord, left)
    }
    pub fn right_click(&self, buffer: &mut MessengerBuffer, session: SessionID, coord: &Coord) {
        let left = false;
        self.click(buffer, session, coord, left)
    }

    #[allow(unused_variables)]
    fn click(&self, buffer: &mut MessengerBuffer, session: SessionID, coord: &Coord, left: bool) {
        let msg = msg::from_slave::Click {
            session: session.into(),
            coord: (*coord).into(),
            left,
        };
        let msg = msg::from_slave::MessageSentByClient;
        self.send_packet_from_slave(buffer, &msg).unwrap();
    }

    fn send_packet_from_slave(
        &self,
        buffer: &mut MessengerBuffer,
        msg: &msg::from_slave::MessageSentByClient,
    ) -> Result<MessageSent, &'static str> {
        let MessengerBuffer(b) = buffer;
        if serde_json::to_writer(b, &msg).is_err() {
            let MessengerBuffer(b) = buffer;
            b.clear();
            return Err("failed to write packet to memory");
        }
        let MessengerBuffer(b) = buffer;
        let err = match self.socket.send(&b[..]) {
            Ok(s) if s == b[..].len() => Ok(MessageSent::Yes),
            Ok(_) => Err("Failed to send entire packet over ip"),
            Err(_) => Ok(MessageSent::No),
        };
        b.clear();
        err
    }

    pub fn send_updates_from_master(
        &self,
        buffer: &mut MessengerBuffer,
        addr: std::net::SocketAddr,
        uid: SessionUserID,
        msg: UpdateFromMaster,
    ) -> Result<MessageSent, &'static str> {
        let msg: msg::from_master::Update = (msg, uid).into();
        let MessengerBuffer(b) = buffer;
        if serde_json::to_writer(b, &msg).is_err() {
            let MessengerBuffer(b) = buffer;
            b.clear();
            return Err("failed to write packet to memory");
        }
        let MessengerBuffer(b) = buffer;
        let err = match self.socket.send_to(b, addr) {
            Ok(s) if s == b[..].len() => Ok(MessageSent::Yes),
            Ok(_) => Err("Failed to send entire packet over ip"),
            Err(_) => Ok(MessageSent::No),
        };
        let MessengerBuffer(b) = buffer;
        b.clear();
        err
    }

    pub fn receive_packet_from_master(
        &self,
        buffer: &mut MessengerBuffer,
    ) -> Option<UpdateFromMaster> {
        self.receive_packet::<msg::from_master::Update>(buffer)
            .map(|(u, _addr)| u.into())
    }

    #[allow(dead_code)]
    pub fn receive_packet_from_slave(
        &self,
        buffer: &mut MessengerBuffer,
    ) -> Option<(MessageFromSlave, std::net::SocketAddr)> {
        self.receive_packet::<msg::from_slave::MessageSentByClient>(buffer)
            .map(|(msg, addr)| (msg.into(), addr))
    }

    fn receive_packet<M>(&self, buffer: &mut MessengerBuffer) -> Option<(M, std::net::SocketAddr)>
    where
        for<'a> M: serde::Deserialize<'a>,
    {
        let MessengerBuffer(buffer) = buffer;
        buffer.resize(BUFFER_FIXED_SIZE, 0);
        let (size, addr) = self.socket.recv_from(buffer).ok()?;
        let b = &buffer[..size];
        let packet = serde_json::from_reader(b)
            .map(|msg| (msg, addr))
            .expect("Failed to parse packet");
        buffer.clear();
        Some(packet)
    }
}

impl MessengerBuffer {
    pub fn new() -> Self {
        let buffer = Vec::new();
        Self(buffer)
    }
}

pub struct MessengerThread {
    keep_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MessengerThread {
    pub fn new(
        keep_running: std::sync::Arc<std::sync::atomic::AtomicBool>,
        thread: std::thread::JoinHandle<()>,
    ) -> Self {
        let thread = Some(thread);
        Self {
            keep_running,
            thread,
        }
    }
}

impl Drop for MessengerThread {
    fn drop(&mut self) {
        self.keep_running
            .store(false, std::sync::atomic::Ordering::Release);
        let thread = self.thread.take().unwrap();
        assert_ne!(std::thread::current().id(), thread.thread().id());
        thread
            .join()
            .map_err(|any| -> String {
                if let Some(err) = any.downcast_ref::<&str>() {
                    err.to_string()
                } else if let Some(err) = any.downcast_ref::<String>() {
                    err.clone()
                } else {
                    "Thread panicked with something different than a string".into()
                }
            })
            .unwrap()
    }
}
