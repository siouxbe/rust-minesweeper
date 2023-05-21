use super::*;

pub fn create_client_session_from_updates_from_master(
    update: UpdateFromMasterForClient,
) -> game::client::session::Session {
    let UpdateFromMasterForClient { coords, updates } = update;
    let mut session = game::client::session::Session::blank(coords, updates.stats);
    session.on_updates(updates);
    session
}
