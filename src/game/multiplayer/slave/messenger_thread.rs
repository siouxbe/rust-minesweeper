use super::*;

use std::sync;

pub fn spawn_read_updates_from_master_to_slave_thread<L>(
    messenger: sync::Weak<Messenger>,
    listener: L,
) -> MessengerThread
where
    L: MasterListener + Send + 'static,
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
    L: MasterListener,
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
    L: MasterListener,
{
    let update = match messenger.receive_packet_from_master(buffer) {
        Some(update) => update,
        None => return,
    };

    listener.on_updates_from_master_to_slave(update);
}
