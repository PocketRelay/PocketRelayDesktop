use std::path::PathBuf;

use gpui_kit::{App, AppContext, Context, Entity, Global};

use crate::{
    config::{Config, save_config_to_path},
    utils::gpui_tokio::Tokio,
};

pub struct ConfigState {
    /// Config
    config: Entity<Config>,

    /// Path of the configuration file to write to
    path: PathBuf,
}

pub trait ProvideConfigState {
    fn config(&self) -> Entity<Config>;

    fn update_config<U: FnOnce(&mut Config)>(&mut self, update: U);
}

impl ConfigState {
    pub fn new(config: Config, path: PathBuf, cx: &mut App) -> Self {
        Self {
            config: cx.new(|_cx| config),
            path,
        }
    }

    fn config(&self) -> Entity<Config> {
        self.config.clone()
    }

    fn save_config<C: AppContext>(cx: &mut C) {
        // Extract the entity and path from the global
        let (config_entity, path) = cx.read_global(|config: &ConfigState, _cx: &App| {
            (config.config.clone(), config.path.clone())
        });

        // Read the current config value
        let config = cx.read_entity(&config_entity, |value, _cx| value.clone());

        // Save the updated config to disk
        Tokio::spawn(cx, async move {
            if let Err(error) = save_config_to_path(&config, &path).await {
                tracing::error!(?error, "failed to save config");
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
