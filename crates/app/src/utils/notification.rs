use gpui_kit::component::WindowExt;
use gpui_kit::component::notification::Notification;
use gpui_kit::{App, AppContext, AsyncApp, SharedString};

pub trait NotificationExt {
    fn push_notification(&mut self, note: impl Into<Notification>);

    fn notify_error(&mut self, message: impl Into<SharedString>);
}

impl NotificationExt for App {
    fn push_notification(&mut self, note: impl Into<Notification>) {
        let note = note.into();
        let windows = self.windows();

        let window = match windows.first() {
            Some(value) => *value,
            None => return,
        };

        _ = self.update_window(window, |_view, window, cx| {
            window.push_notification(note, cx);
        });
    }

    fn notify_error(&mut self, message: impl Into<SharedString>) {
        self.push_notification(
            Notification::new()
                .message(message)
                .with_type(gpui_kit::component::notification::NotificationType::Error),
        );
    }
}

impl NotificationExt for AsyncApp {
    fn push_notification(&mut self, note: impl Into<Notification>) {
        self.update(|cx| {
            cx.push_notification(note);
        });
    }

    fn notify_error(&mut self, message: impl Into<SharedString>) {
        self.update(|cx| {
            cx.notify_error(message);
        });
    }
}
