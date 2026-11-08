use anyhow::Context;
use gpui_kit::component::{Theme, ThemeMode, TitleBar};
use gpui_kit::*;
use pocket_relay_desktop_server::services::Services;

use crate::config::{Config, load_config_from_path};
use crate::screens::app::AppScreen;
use crate::state::config::ConfigState;
use crate::utils::gpui_tokio::{Tokio, init_tokio};
use crate::utils::logging;
use crate::utils::paths::get_project_dirs;

pub mod config;
pub mod screens;
pub mod state;
pub mod utils;

fn init_theme(cx: &mut App) {
    // Set the theme to dark
    Theme::change(ThemeMode::Dark, None, cx);

    // Move the notifications to the bottom right
    let theme = Theme::global_mut(cx);
    theme.notification.placement = Anchor::BottomRight;
}

fn main() -> anyhow::Result<()> {
    logging::setup();

    let paths = get_project_dirs().context("failed to load project dirs")?;

    application().with_assets(assets::Assets).run(move |cx| {
        init(cx);
        init_theme(cx);
        init_tokio(cx);

        let config_base_path = paths.config_dir();
        let config_path = config_base_path.join("config.json");

        let read_config_path = config_path.clone();

        let config = Tokio::block_on(
            cx,
            async move { load_config_from_path(&read_config_path).await },
        );

        let services = Tokio::block_on(cx, Services::start());
        let _services = match services {
            Ok(value) => value,
            Err(error) => {
                // TODO: Render a dedicated app window instead
                tracing::error!(?error, "failed to start services");
                cx.show_system_notification(SystemNotification {
                    tag: "startup-failure".into(),
                    title: "Failed to start services".into(),
                    body: "One or more internal service failed to start, Pocket Relay will be unavailable".into(),
                    actions: Default::default(),
                });
                return;
            }
        };

        dbg!(&config_path);

        let config = match config {
            Ok(value) => value,
            Err(error) => {
                tracing::error!(?error, "failed to load configuration");
                Config::default()
            }
        };

        let config_state = ConfigState::new(config, config_path, cx);
        cx.set_global(config_state);

        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("Pocket Relay Desktop".into()),
                    ..TitleBar::title_bar_options()
                }),
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                // Make sure linux lets us handle the window decorations
                // window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(|_| AppScreen::default()),
        )
        .expect("Failed to open window");
    });

    Ok(())
}
