use super::*;

pub trait MasterListener {
    fn on_updates_from_master_to_slave(&mut self, update: UpdateFromMaster);
}
