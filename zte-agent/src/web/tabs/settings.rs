//! Settings tab: the app's settings page (`SettingsView`), in its order.
//! The gateway and Keychain sections have no web counterpart (the browser
//! is already on the agent; its password manager keeps the password);
//! "Session" with "Log out" takes their place (as the Android app's).
//! "Polling" and "Appearance" are this browser's preferences
//! (`web::prefs`), each a form that sets its cookie; their footers add only
//! "Saved in this browser." to the app's (the setting is per browser).

use maud::{html, Markup};

use crate::web::kit::{self, Tab};
use crate::web::pages::https;
use crate::web::prefs::{self, Theme};
use crate::web::screen::{Bar, Block, PageData, Screen};

pub static SETTINGS: Screen = Screen {
    path: "/ui/settings",
    id: "settings",
    title: "设置",
    tab: Tab::Settings,
    blocks: &[
        Block::Custom(session),
        Block::Custom(polling),
        Block::Custom(appearance),
        // LAN HTTPS (a web addition: `pages::https`).
        Block::Custom(https::section),
        Block::Rows(
            "关于",
            &[
                ("应用程序", "OpenU60"),
                ("设备", "ZTE U60 Pro (MU5250)"),
                ("API", concat!("zte-agent web UI ", env!("CARGO_PKG_VERSION"))),
            ],
        ),
        Block::Custom(legal),
    ],
    ..Screen::BLANK
};

/// Settings › HTTPS › "Install on Your Devices" (`pages::https`).
pub static INSTALL: Screen = Screen {
    path: https::PAGE,
    id: "install",
    title: "在您的设备上安装",
    tab: Tab::Settings,
    bar: Bar::Back("/ui/settings", "设置"),
    blocks: &[Block::Custom(https::install)],
    ..Screen::BLANK
};

fn session(_: &PageData) -> Markup {
    kit::section(Some("会议"), None, html! {
        li.it { span { "在此浏览器中登录" } }
        li { a.it.danger href="/ui/logout" { span { "退出登录" } } }
    })
}

const SAVED_HERE: &str = "保存在此浏览器中。";

/// The app's slider as a list of its values (a range input shows no value
/// without JavaScript) and "Save".
fn polling(pd: &PageData) -> Markup {
    let cur = pd.prefs.refresh_ms;
    let footer = html! {
        p.foot { "实时上传和下载使用单独的 1 秒刷新，与调制解调器 \u{2019} 的更新速率相匹配。" }
        p.foot { (SAVED_HERE) }
    };
    kit::section(Some("投票"), Some(footer), html! {
        li.it {
            form.sf method="post" action=(prefs::REFRESH) {
                (pd.csrf())
                label for="ms" { "刷新间隔" }
                select #ms name="ms" {
                    @for ms in (1000..=10_000).step_by(500) {
                        option value=(ms) selected[ms == cur] { (prefs::refresh_label(ms)) }
                    }
                }
                button.btn.sm type="submit" { "保存" }
            }
        }
    })
}

/// The app's segmented "Theme": three buttons of one form, the current one
/// pressed.
fn appearance(pd: &PageData) -> Markup {
    let cur = pd.prefs.theme;
    kit::section(Some("外观"), Some(html! { p.foot { (SAVED_HERE) } }), html! {
        li.it {
            form.sf method="post" action=(prefs::THEME) {
                (pd.csrf())
                div.seg.th role="group" aria-label="主题" {
                    @for t in Theme::ALL {
                        button type="submit" name="theme" value=(t.key()) aria-pressed=(if t == cur { "true" } else { "false" }) { (t.label()) }
                    }
                }
            }
        }
    })
}

fn legal(_: &PageData) -> Markup {
    kit::section(Some("法律声明"), None, html! {
        li.it {
            p.cap {
                "This app is not affiliated with, endorsed by, or sponsored by ZTE Corporation. \
                 ZTE and U60 Pro are trademarks of ZTE Corporation."
            }
        }
        li { a.it href="https://open-u60-pro.vercel.app/privacy" rel="noopener noreferrer" { span { "隐私政策" } } }
    })
}
