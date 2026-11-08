use gpui_kit::*;

use crate::{
    config::Config, screens::setup::pick_game_executable::PickGameExecutable,
    state::config::ProvideConfigState,
};

pub mod pick_game_executable;

#[derive(Default)]
pub struct Setup {
    pick_game_executable: Option<Entity<PickGameExecutable>>,
}

impl Setup {
    pub fn is_setup_required(config: &Config) -> bool {
        config.game_executable.is_none()
    }
}

impl Render for Setup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = cx.config();
        match &config.read(cx).game_executable {
            Some(executable) => div().child(executable.to_string_lossy().to_string()),
            None => {
                let pick_game_executable = match self.pick_game_executable.as_ref() {
                    Some(value) => value.clone(),
                    None => {
                        let setup =
                            cx.new(|_cx: &mut Context<'_, PickGameExecutable>| PickGameExecutable);
                        _ = self.pick_game_executable.insert(setup.clone());
                        setup
                    }
                };

                div().size_full().child(pick_game_executable)
            }
        }
    }
}
