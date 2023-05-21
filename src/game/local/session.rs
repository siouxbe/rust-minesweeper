use super::*;

use crate::game::session::Namer;

/// Contains all data required to run a single-player game
pub struct Session {
    server: server::Server<game::client::session::Session>,
    /*
     * TODO: Replace by another type
     */
    client: game::client::session::Session,
    namer: LocalSessionNamer,
}

impl Session {
    pub fn new(config: &game::session::SessionConfig) -> Self {
        let game::session::SessionConfig {
            coords,
            mines,
            lives,
        } = *config;
        let client = game::client::session::Session::new(coords, mines, lives);
        let server = game::server::session::Session::new(coords, mines, lives);
        /*
         * TODO: The 'server' variable contains an instance of the multi-purpose struct
         * 'game::server::session::Session'. It must be wrapped into a more specific instance of
         * 'game::local::server::Server'. In the documentation of that last one, you will find that
         * it requires an updates_listener, and that it must implement the trait UpdatesListener
         * for it to work. In the documentation of UpdatesListener, you will find that it has one
         * method 'on_updates' which takes &mut self as the first argument.
         *
         * Now luckily for us, we already have an instance here that implements that trait: the
         * variable 'client' holds a value that does. Unfortunately, we can not pass that variable
         * by value to the call of 'server::Server::new' (try it out, the compiler will confirm)
         * because it must also be passed by value to the return value of this function.
         *
         * The instance of 'client' also does not implement the 'Clone' trait, and besides, we
         * don't want to have two instances of it anyway.
         *
         * You must replace the type of the field 'client: game::client::session::Session' with
         * something that allows us to pass multiple references to it (shared ownership) at
         * runtime, so that we can pass it to both 'server::Server::new' and the returned value.
         * As long as that new type still implements the trait 'UpdatesListener', we should be
         * fine.
         *
         * Two extra warnings though:
         * First, you can safely assume that whenever the value of the
         * client is needed, there will not be another function deeper in the call stack that is
         * referencing it.
         * Second, for you to implement 'UpdatesListener', you need to have exclusive access to the
         * client. So the problem is dual: we need both shared ownership and the option to acquire
         * exclusive access to its contents at runtime.
         *
         */
        let local_updates_listener = todo!();
        let server = server::Server::new(server, local_updates_listener, SessionUserID::new(1));
        let namer = LocalSessionNamer;
        Self {
            server,
            client,
            namer,
        }
    }
}

impl game::session::Session for Session {
    fn snapshot<F, R>(&mut self, f: F) -> R
    where
        for<'a> F: FnOnce(game::session::SessionSnapshot<'a>) -> R,
    {
        let user_stats = self.server.user_stats();
        let status = self.server.status();
        let coords = self.server.coords();
        f(game::session::SessionSnapshot {
            stats: self.client.stats(),
            user_stats,
            status,
            coords,
            namer: &self.namer,
            field_provider: &self.client,
            local_player_listener: &mut self.server,
        })
    }
}

struct LocalSessionNamer;

impl Namer for LocalSessionNamer {
    fn name(&self, _uid: SessionUserID) -> &str {
        "Single Player"
    }
}

