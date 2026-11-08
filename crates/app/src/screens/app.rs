use gpui_kit::*;

use crate::screens::setup::Setup;
use crate::state::config::ProvideConfigState;

#[derive(Default)]
pub struct AppScreen {
    setup: Option<Entity<Setup>>,
}

impl Render for AppScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = cx.config();

        if Setup::is_setup_required(config.read(cx)) {
            let setup = match self.setup.as_ref() {
                Some(value) => value.clone(),
                None => {
                    let setup = cx.new(|_cx| Setup::default());
                    _ = self.setup.insert(setup.clone());
                    setup
                }
            };

            return div().size_full().child(setup);
        }

        div().child("setup is complete")
    }
}
