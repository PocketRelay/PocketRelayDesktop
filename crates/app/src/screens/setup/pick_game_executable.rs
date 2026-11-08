use crate::utils::gpui_tokio::Tokio;
use crate::utils::notification::NotificationExt;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::*;
use rfd::FileHandle;
use tokio::task::JoinError;

// TODO: Route to background load known folders automatically to allow selecting them
pub struct PickGameExecutable;

impl PickGameExecutable {
    /// Handles clicks to the pick game path
    fn on_click_pick_game_path(_event: &ClickEvent, _window: &mut Window, cx: &mut App) {
        let pick_future = rfd::AsyncFileDialog::new()
            .add_filter("MassEffect3.exe", &["exe"])
            .set_title("Pick MassEffect3.exe path")
            .pick_file();

        Tokio::spawn_with_callback(cx, pick_future, Self::on_path_picked);
    }

    fn on_path_picked(cx: &mut App, result: Result<Option<FileHandle>, JoinError>) {
        let result = match result {
            Ok(Some(value)) => value,
            Ok(None) => return,
            Err(error) => {
                tracing::error!(?error, "failed to pick file path");
                cx.notify_error("failed to pick file path");
                return;
            }
        };

        let path = result.path().to_path_buf();
        use crate::state::config::ProvideConfigState;

        cx.update_config(|config| {
            config.game_executable = Some(path);
        });
    }
}

impl Render for PickGameExecutable {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .child(div()
                .child("Please select your MassEffect.exe can be found at {PATH TO YOUR GAME}/Binaries/Win32/MassEffect3.exe")
                .child(Button::new("pick-game-path")
                    .primary()
                    .label("Pick game path")
                    .on_click(Self::on_click_pick_game_path)
                )
            )
            .child(
                div()
                    .child("Automatically detected game folders:")
                    .child("TODO: Not implemented"),
            )
    }
}
