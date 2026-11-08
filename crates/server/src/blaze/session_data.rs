use parking_lot::{RwLock, RwLockReadGuard};
use serde::Serialize;
use std::{net::Ipv4Addr, sync::Arc, task::Context, time::Duration};
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

use crate::{
    blaze::{
        components::user_sessions,
        models::{
            game_manager::RemoveReason,
            user_sessions::{
                HardwareFlags, LookupResponse, NetworkAddress, NotifyUserAdded, NotifyUserRemoved,
                NotifyUserUpdated, OwnedUserIdentification, QosNetworkData, UserDataFlags,
                UserIdentification, UserSessionExtendedData, UserSessionExtendedDataUpdate,
            },
        },
        session::WeakSessionLink,
    },
    services::{
        game::{GameID, GameRef, WeakGameRef},
        sessions::{AssociationId, SessionUserAssociation},
    },
};

use pocket_ark_blaze_networking::packet::Packet;
use pocket_ark_database::dto::users::{UserDto, UserId};

pub struct SessionInitData {
    /// IP address associated with the session
    pub addr: Ipv4Addr,
    /// User will not have an association if they are using an outdated
    /// client version.
    pub association_id: Option<AssociationId>,
}

pub struct SessionData {
    /// Extended session data, writable data
    ext: RwLock<SessionDataExt>,

    /// IP address associated with the session
    addr: Ipv4Addr,

    /// User will not have an association if they are using an outdated
    /// client version.
    association: Option<AssociationId>,
}

struct SessionDataExt {
    /// Data for authorized sessions
    auth: Option<SessionDataAuth>,

    /// Keep-alive data for the session
    keep_alive: SessionDataKeepAlive,
}

impl SessionDataExt {
    fn new() -> Self {
        Self {
            auth: None,
            keep_alive: SessionDataKeepAlive::new(),
        }
    }
}

pub struct SessionDataKeepAlive {
    /// Last time a keep-alive message was received through the tunnel
    pub last_keep_alive: Instant,

    /// Time that has been granted as a grace period to allow the
    /// session to go without a keep-alive message for
    pub extended_grace: Duration,

    /// Interval for polling connection alive checks
    pub keep_alive_interval: Interval,
}

/// Delay between each keep-alive check
pub const KEEP_ALIVE_DELAY: Duration = Duration::from_secs(15);

/// When this duration elapses between keep-alive checks for a connection
/// the connection is considered to be dead (4 missed keep-alive check intervals)
pub const KEEP_ALIVE_TIMEOUT: Duration = Duration::from_secs(KEEP_ALIVE_DELAY.as_secs() * 4);

impl SessionDataKeepAlive {
    fn new() -> Self {
        let now = Instant::now();

        // Create the interval to track keep alive checking
        let keep_alive_start = now + KEEP_ALIVE_DELAY;
        let mut keep_alive_interval = interval_at(keep_alive_start, KEEP_ALIVE_DELAY);

        keep_alive_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        Self {
            last_keep_alive: Instant::now(),
            extended_grace: Duration::from_secs(0),
            keep_alive_interval,
        }
    }
}

impl SessionData {
    /// Creates new session data
    pub fn new(addr: Ipv4Addr, association: Option<AssociationId>) -> Self {
        Self {
            ext: RwLock::new(SessionDataExt::new()),
            addr,
            association,
        }
    }

    /// Polls the keep alive check to see if its ready and if the connection is dead
    pub fn poll_keep_alive_dead(&self, cx: &mut Context<'_>) -> bool {
        let keep_alive = &mut self.ext.write().keep_alive;

        // Not ready to perform a keep-alive check
        if !keep_alive.keep_alive_interval.poll_tick(cx).is_ready() {
            return false;
        }

        // Check the keep alive state
        let now = Instant::now();
        let last_alive = keep_alive
            .last_keep_alive
            // Get time since last keep alive message
            .duration_since(now)
            // Remove current grace period from the elapsed time
            .saturating_sub(keep_alive.extended_grace);

        // Connection to the client has timed out as no keep alive messages were
        // given by the client
        last_alive > KEEP_ALIVE_TIMEOUT
    }

    /// Sets the connection as alive
    pub fn set_alive(&self) {
        let keep_alive = &mut self.ext.write().keep_alive;

        // Clear existing grace period
        keep_alive.extended_grace = Duration::from_secs(0);

        // Mark current alive period
        keep_alive.last_keep_alive = Instant::now();
    }

    /// Grants a grace period duration where the client is allowed to not send any keep-alive
    /// messages and won't be timed-out for doing so
    pub fn set_keep_alive_grace(&self, grace: Duration) {
        let keep_alive = &mut self.ext.write().keep_alive;
        let now = Instant::now()
            .checked_add(grace)
            .expect("reached limit of time");

        // Delay next keep alive check
        keep_alive.keep_alive_interval.reset_at(now);

        // Apply grace period to next check
        keep_alive.extended_grace = grace;
    }

    pub fn get_addr(&self) -> Ipv4Addr {
        self.addr
    }

    pub fn get_association(&self) -> Option<AssociationId> {
        self.association
    }

    // Read from the underlying session data
    fn read(&self) -> RwLockReadGuard<'_, SessionDataExt> {
        self.ext.read()
    }

    /// Writes to the underlying session data without publishing the changes
    fn write_silent<F, O>(&self, update: F) -> Option<O>
    where
        F: FnOnce(&mut SessionDataAuth) -> O,
    {
        self.ext.write().auth.as_mut().map(update)
    }

    /// Writes to the underlying session data, publishes changes to
    /// subscribers
    #[inline]
    fn write_publish<F, O>(&self, update: F) -> Option<O>
    where
        F: FnOnce(&mut SessionDataAuth) -> O,
    {
        self.ext.write().auth.as_mut().map(|data| {
            let value = update(data);
            data.publish_update();
            value
        })
    }

    /// Clears the underlying authenticated session data
    pub fn clear_auth(&self) {
        let auth = { self.ext.write().auth.take() };

        // Auth MUST be dropped *after* the write lock is released
        // as releasing a player from a game requires accessing this
        drop(auth);
    }

    /// Starts a session from the provided player association
    pub fn set_auth(&self, player: SessionUserAssociation) -> Arc<UserDto> {
        self.ext
            .write()
            .auth
            .insert(SessionDataAuth::new(player))
            // Obtain the player to return
            .player_assoc
            .user
            .clone()
    }

    /// Gets the currently authenticated player
    pub fn get_player(&self) -> Option<Arc<UserDto>> {
        self.read()
            .auth
            .as_ref()
            .map(|value| value.player_assoc.user.clone())
    }

    /// Updates the session hardware flags
    pub fn set_hardware_flags(&self, value: HardwareFlags) {
        self.write_publish(|data| data.net = Arc::new(data.net.with_hardware_flags(value)));
    }

    /// Sets the current session network information
    pub fn set_network_info(
        &self,
        address: NetworkAddress,
        qos: QosNetworkData,
        ping_site_latency: Vec<u32>,
    ) {
        self.write_publish(|data| {
            data.net = Arc::new(data.net.with_basic(address, qos, ping_site_latency))
        });
    }

    /// Obtains the network data for the session
    pub fn network_info(&self) -> Option<Arc<NetData>> {
        self.read().auth.as_ref().map(|value| value.net.clone())
    }

    /// Sets the game the session is currently apart of
    pub fn set_game(&self, game_id: GameID, game_ref: WeakGameRef) {
        // Set the current game
        self.write_publish(|data| {
            data.game = Some(SessionGameData {
                player_id: data.player_assoc.user.id,
                game_id,
                game_ref,
            });
        });
    }

    /// Clears the game the session is apart of
    ///
    /// DONT CALL FROM CODE HOLDING A WRITE LOCK TO THE
    /// USER GAME, call [Self::clear_game_gm] instead which
    /// doesn't let the drop run preventing a deadlock
    pub fn clear_game(&self) {
        self.write_publish(|data| data.game = None);
    }

    /// Clears the game the session is apart of, safe to
    /// call from the game manager as it prevents the default
    /// drop behavior of trying to remove the player from the game
    pub fn clear_game_gm(&self) {
        self.write_publish(|data| {
            let game = data.game.take();
            std::mem::forget(game);
        });
    }

    /// Obtains the ID and reference to the game the session is currently apart of
    pub fn get_game(&self) -> Option<(GameID, GameRef)> {
        let guard = self.read();

        let data = guard.auth.as_ref()?;
        let game_data = data.game.as_ref()?;

        let game_ref = match game_data.game_ref.upgrade() {
            Some(value) => value,
            // We have a dangling game ref, clean it up
            None => {
                // Drop the guard before writing
                drop(guard);

                // Clear the dangling game ref
                self.clear_game_gm();
                return None;
            }
        };

        Some((game_data.game_id, game_ref))
    }

    pub fn get_lookup_response(&self) -> Option<LookupResponse> {
        self.read().auth.as_ref().map(|data| LookupResponse {
            user: OwnedUserIdentification {
                id: data.player_assoc.user.id as u64,
                name: data.player_assoc.user.username.clone(),
            },
            extended_data: data.ext_data(),
        })
    }

    /// Adds a subscriber to the session
    pub fn add_subscriber(&self, player_id: UserId, subscriber: WeakSessionLink) {
        self.write_silent(|data| data.add_subscriber(player_id, subscriber));
    }

    /// Removes a subscriber from the session
    pub fn remove_subscriber(&self, player_id: UserId) {
        self.write_silent(|data| data.remove_subscriber(player_id));
    }

    pub fn publish_update(&self) {
        self.write_silent(|data| data.publish_update());
    }

    pub fn net(&self) -> Option<Arc<NetData>> {
        let ext = self.read();
        let auth = ext.auth.as_ref()?;
        Some(auth.net.clone())
    }
}
