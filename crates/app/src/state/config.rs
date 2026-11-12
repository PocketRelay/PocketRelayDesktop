use std::path::PathBuf;

use gpui_kit::{App, AppContext, Context, Entity, Global};
use pocket_relay_desktop_server::services::ServiceHandle;

use crate::{
    config::{Config, save_config_to_path, save_rpc_state},
    utils::gpui_tokio::Tokio,
};

pub struct ConfigState {
    /// Config
    config: Entity<Config>,

    /// Path of the configuration file to write to
    path: PathBuf,

    /// Service running the RPC server
    rpc_service: ServiceHandle,
}

pub trait ProvideConfigState {
    fn config(&self) -> Entity<Config>;

    fn update_config<U: FnOnce(&mut Config)>(&mut self, update: U);
}

impl ConfigState {
    pub fn new(config: Config, path: PathBuf, rpc_service: ServiceHandle, cx: &mut App) -> Self {
        Self {
            config: cx.new(|_cx| config),
            path,
            rpc_service,
        }
    }

    fn config(&self) -> Entity<Config> {
        self.config.clone()
    }

    fn save_config<C: AppContext>(cx: &mut C) {
        // Extract the entity and path from the global
        let (config_entity, path, rpc_service) =
            cx.read_global(|config: &ConfigState, _cx: &App| {
                (
                    config.config.clone(),
                    config.path.clone(),
                    config.rpc_service.clone(),
                )
            });

        // Read the current config value
        let config = cx.read_entity(&config_entity, |value, _cx| value.clone());

        // Save the updated config to disk
        Tokio::spawn(cx, async move {
            if let Err(error) = save_config_to_path(&config, &path).await {
                tracing::error!(?error, "failed to save config");
            }

            // TODO: Emit changed action instead we also need to handle adding the client plugin
            if let Some(game_executable) = config.game_executable
                && let Err(error) = save_rpc_state(&game_executable, rpc_service.addr).await
            {
                tracing::error!(?error, "failed to save config");
                // TODO: Display to the user somehow
            }
        })
        .detach();
    }
}

impl Global for ConfigState {}

impl<T: 'static> ProvideConfigState for Context<'_, T> {
    fn config(&self) -> Entity<Config> {
        self.global::<ConfigState>().config()
    }

    fn update_config<U: FnOnce(&mut Config)>(&mut self, update: U) {
        let config = self.config();
        self.update_entity(&config, |value, _cx| update(value));
        ConfigState::save_config(self);
    }
}

impl ProvideConfigState for App {
    fn config(&self) -> Entity<Config> {
        self.global::<ConfigState>().config()
    }

    fn update_config<U: FnOnce(&mut Config)>(&mut self, update: U) {
        let config = self.config();
        self.update_entity(&config, |value, _cx| update(value));
        ConfigState::save_config(self);
    }
}
