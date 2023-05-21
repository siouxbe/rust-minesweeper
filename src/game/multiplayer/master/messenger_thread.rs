use super::*;

use std::sync;

pub fn spawn_read_messages_from_slaves_to_master_thread<L>(
    messenger: sync::Weak<Messenger>,
    listener: L,
) -> MessengerThread
where
    L: SlaveListener + Send + 'static,
{
    let keep_running = sync::Arc::new(sync::atomic::AtomicBool::new(true));
    let keep_running2 = keep_running.clone();
    let thread = std::thread::Builder::new()
        .name("Messenger reading thread".into())
        .spawn(move || thread_fn(keep_running2, messenger, listener))
        .expect("Unable to spawn messenger reading thread");
    MessengerThread::new(keep_running, thread)
}

fn thread_fn<L>(
    keep_running: sync::Arc<sync::atomic::AtomicBool>,
    messenger: sync::Weak<Messenger>,
    mut listener: L,
) where
    L: SlaveListener,
{
    let mut buffer = MessengerBuffer::new();
    while let (true, Some(messenger)) = (
        keep_running.load(sync::atomic::Ordering::Acquire),
        messenger.upgrade(),
    ) {
        let messenger = &messenger;
        thread_cycle(messenger, &mut buffer, &mut listener);
    }
}

fn thread_cycle<L>(messenger: &Messenger, buffer: &mut MessengerBuffer, listener: &mut L)
where
    L: SlaveListener,
{
    let (packet, addr) = match messenger.receive_packet_from_slave(buffer) {
        Some(packet_and_address) => packet_and_address,
        None => {
            return;
        }
    };

    match packet {
        MessageFromSlave::Join(request) => listener.on_request_to_join(request, addr),
        MessageFromSlave::Action(action) => {
            listener.on_action_from_slave(action, addr);
        }
    }
}
