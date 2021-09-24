pub mod assets;
pub mod channel;
pub mod chat_panel;
pub mod editor;
pub mod file_finder;
pub mod fs;
mod fuzzy;
pub mod http;
pub mod language;
pub mod menus;
pub mod people_panel;
pub mod project;
pub mod project_panel;
pub mod rpc;
pub mod settings;
#[cfg(any(test, feature = "test-support"))]
pub mod test;
pub mod theme;
pub mod theme_selector;
mod time;
pub mod user;
mod util;
pub mod workspace;
pub mod worktree;

use crate::util::TryFutureExt;
use channel::ChannelList;
use gpui::{action, keymap::Binding, ModelHandle};
use parking_lot::Mutex;
use postage::watch;
use std::sync::Arc;

pub use settings::Settings;

action!(About);
action!(Quit);
action!(Authenticate);
action!(AdjustBufferFontSize, f32);

const MIN_FONT_SIZE: f32 = 6.0;

pub struct AppState {
    pub settings_tx: Arc<Mutex<watch::Sender<Settings>>>,
    pub settings: watch::Receiver<Settings>,
    pub languages: Arc<language::LanguageRegistry>,
    pub themes: Arc<settings::ThemeRegistry>,
    pub rpc: Arc<rpc::Client>,
    pub user_store: ModelHandle<user::UserStore>,
    pub fs: Arc<dyn fs::Fs>,
    pub channel_list: ModelHandle<ChannelList>,
}

pub fn init(app_state: &Arc<AppState>, cx: &mut gpui::MutableAppContext) {
    cx.add_global_action(quit);

    cx.add_global_action({
        let rpc = app_state.rpc.clone();
        move |_: &Authenticate, cx| {
            let rpc = rpc.clone();
            cx.spawn(|cx| async move { rpc.authenticate_and_connect(&cx).log_err().await })
                .detach();
        }
    });

    cx.add_global_action({
        let settings_tx = app_state.settings_tx.clone();

        move |action: &AdjustBufferFontSize, cx| {
            let mut settings_tx = settings_tx.lock();
            let new_size = (settings_tx.borrow().buffer_font_size + action.0).max(MIN_FONT_SIZE);
            settings_tx.borrow_mut().buffer_font_size = new_size;
            cx.refresh_windows();
        }
    });

    cx.add_bindings(vec![
        Binding::new("cmd-=", AdjustBufferFontSize(1.), None),
        Binding::new("cmd--", AdjustBufferFontSize(-1.), None),
    ])
}

fn quit(_: &Quit, cx: &mut gpui::MutableAppContext) {
    cx.platform().quit();
}
