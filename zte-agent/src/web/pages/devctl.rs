//! Router › Device Controls (iOS `DeviceControlView`), read at page load
//! (the app reads once; no stream): "Charge Limit" (the app's slider and
//! stepper as two lists: 50–100 % by 5, 1–20 %; `tabs::router::
//! CHARGE_LIMIT`), "Power-save Mode", "Fast Boot", "Auto Sleep" with its
//! "Idle Timeout", "Reboot Router" and "Factory Reset", each with the
//! app's footer. Every form carries the page's single-use token.
//!
//! - The three switches and the idle timeout post from the page with its
//!   single-use token and are sent at once, as the app's (`pages::confirm`
//!   kinds; a double click sends once), with the app's bodies; the page
//!   then shows what the router reads back, and a failure the app's
//!   "Failed: …". Turning Auto Sleep on is asked first for a remote browser
//!   (a web addition): with no Wi-Fi device connected the router sleeps
//!   until its power button is pressed, so this page can't be reached.
//! - Reboot Router and Factory Reset are the app's password sheets as
//!   pages; so is Mobile Network's "Reboot Now" (its "Reboot Required").
//!   The router password is checked on the router (`AuthState::
//!   check_password`: constant time, counted by the login rate limit; the
//!   app compares it with its Keychain), never echoed, kept or logged.
//!   Then (single-use token) the call, and the "Rebooting…" page in the
//!   post's answer itself: the router may stop answering right after.
//! - A read that fails shows the app's "Failed to load …" with the error,
//!   and its switch "Unavailable" with both directions; Charge Limit only
//!   "Unavailable" (its Save would write the defaults over the router's).
//! - Factory Reset's password field asks browsers and password managers
//!   not to fill it (the reset stays a deliberate step); Reboot's may be
//!   filled with the agent password the browser keeps.

use axum::extract::Request;
use axum::http::header::{HeaderValue, RETRY_AFTER};
use axum::http::Method as HttpMethod;
use axum::response::Response;
use maud::{html, Markup};
use serde_json::{json, Value};

use super::confirm::{Bad, Input, Kind, Next, Question, REMOTE};
use super::netcfg::{field, foot, on_fields, select_row, switch, text, Vals};
use crate::auth::{retry_after_secs, LoginError, PeerKey};
use crate::handlers::AppState;
use crate::httpd::{Ctx, Peer};
use crate::server::Method;
use crate::web::job::{self, OnceJob, Sent};
use crate::web::kit::{self, Act, Icon, Tab};
use crate::web::render::{self, once_field, PageCtx};
use crate::web::screen::{self, FormState, Notice, PageData, Req, Screen};
use crate::web::session;
use crate::web::store::{Claim, Token, SETTLE_SLOW};
use crate::web::tabs::router;
use crate::web::{fmt, CspNonce};

pub const PATH: &str = "/ui/router/device";
pub const CHARGE: &str = "/ui/router/device/charge-limit";
pub const POWER_SAVE: &str = "/ui/router/device/power-save";
pub const FAST_BOOT: &str = "/ui/router/device/fast-boot";
pub const AUTO_SLEEP: &str = "/ui/router/device/auto-sleep";
pub const REBOOT: &str = "/ui/router/device/reboot";
pub const RESET: &str = "/ui/router/device/factory-reset";
pub const POWER_SAVE_API: &str = "/api/device/power-save";
pub const FAST_BOOT_API: &str = "/api/device/fast-boot";
pub const AUTO_SLEEP_API: &str = "/api/device/auto-sleep";

// --- The page ---

/// The power-save mode (the app's read, which the API takes as a POST:
/// `get_device_info`, called here directly so no page load ever posts).
pub fn power_save(app: &AppState, _: Option<&str>) -> Result<Value, String> {
    #[cfg(test)]
    if let Some(r) = super::device::get(app, POWER_SAVE_API) {
        return r;
    }
    let (status, mut v) = crate::device_ext::device_power_save_get(app, br#"{"deviceInfoList":["power_saver_mode"]}"#);
    if status < 400 {
        Ok(v["data"].take())
    } else {
        Err(v["error"].as_str().unwrap_or("error").to_owned())
    }
}

/// "1" on, "0" (or another value) off, as the app reads it; nothing read:
/// unknown.
fn on_off(v: &Value) -> Option<bool> {
    match text(v).as_str() {
        "" => None,
        s => Some(s == "1"),
    }
}

/// The app's idle timeouts (minutes) and their labels.
const TIMEOUTS: [(&str, &str); 6] = [("5", "5 min"), ("10", "10 min"), ("20", "20 min"), ("30", "30 min"), ("60", "1 hour"), ("120", "2 hours")];

/// A timeout the agent takes: 1–1440 minutes.
fn timeout_ok(t: &str) -> bool {
    (1..=4).contains(&t.len()) && t.bytes().all(|b| b.is_ascii_digit()) && t.parse::<u16>().is_ok_and(|m| (1..=1440).contains(&m))
}

fn timeout_label(t: &str) -> String {
    TIMEOUTS.iter().find(|(v, _)| *v == t).map_or_else(|| format!("{t} min"), |(_, l)| (*l).to_owned())
}

/// The router's idle timeout, else the app's default (5 minutes).
fn timeout_of(d: Option<&Value>) -> String {
    d.map(|d| text(&d["timeout"])).filter(|t| timeout_ok(t)).unwrap_or_else(|| "5".to_owned())
}

/// The app's charge-limit values: 50–100 % by 5 (its slider), 1–20 %.
const STOPS: [&str; 11] = ["50", "55", "60", "65", "70", "75", "80", "85", "90", "95", "100"];
const GAPS: [&str; 20] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20"];

/// A percentage picker: the app's values, with the shown one first when it
/// is a whole number that is not one of them.
fn percent_row(pd: &PageData, name: &str, label: &str, list: &[&str], shown: &str) -> Markup {
    let mut opts: Vec<(String, String)> = Vec::with_capacity(list.len() + 1);
    if !list.contains(&shown) && shown.parse::<i64>().is_ok() {
        opts.push((shown.to_owned(), format!("{shown}%")));
    }
    opts.extend(list.iter().map(|v| ((*v).to_owned(), format!("{v}%"))));
    select_row(name, label, &opts, shown, pd.form.and_then(|f| f.error(name)), "it lim")
}

fn charge(pd: &PageData) -> Markup {
    // Unread: no form (the app sends nothing until it has read the limit).
    if let Some(Err(_)) = pd.reads.first() {
        return kit::section(None, Some(foot("当电池电量达到设定水平时停止充电。延长电池寿命。")), html! {
            li.it { span { "充电上限" } span.v { "不可用" } }
        });
    }
    let d = pd.read(0);
    let saved = |key, default: i64| d.and_then(|c| fmt::int(&c[key])).unwrap_or(default);
    let (limit, hysteresis) = (saved("charge_limit", 80), saved("hysteresis", 5));
    let on = d.is_some_and(|c| c["charge_limit_enabled"].as_bool() == Some(true));
    // As typed after a refused post, else as saved.
    let typed = |name, value: i64| pd.form.and_then(|f| f.value(name)).map_or_else(|| value.to_string(), str::to_owned);
    let checked = pd.form.map_or(on, |f| f.value("enabled") == Some("1"));
    let footer = html! {
        @if on {
            (foot(&format!("充电在 {limit}% 处停止，并在 {}% 处恢复。", limit - hysteresis)))
            (foot("The resume gap prevents the charger from rapidly switching on and off. A smaller gap (e.g. 2%) keeps \
                   the battery closer to your target but toggles the charger more often. A larger gap (e.g. 10%) means \
                   fewer charge cycles but the battery level will swing more."))
            (foot("默认值：5% \u{2014} 对于大多数用户来说是一个很好的平衡。"))
        } @else {
            (foot("当电池电量达到设定水平时停止充电。延长电池寿命。"))
        }
    };
    html! {
        form.rf.chl method="post" action=(CHARGE) {
            (pd.csrf())
            (once_field(&pd.once()))
            (kit::section(None, Some(footer), html! {
                (switch("enabled", "充电上限", checked, "it", "sr"))
                (percent_row(pd, "limit", "停止充电阈值", &STOPS, &typed("limit", limit)))
                (percent_row(pd, "hysteresis", "恢复充电差值", &GAPS, &typed("hysteresis", hysteresis)))
                li { button.it.cv.snd type="submit" { "保存" } }
            }))
        }
    }
}

/// A read's "Failed to load …" line (the app's words, with the error).
fn load_error(pd: &PageData, i: usize, what: &str) -> Markup {
    html! {
        @if let Some(Err(e)) = pd.reads.get(i) {
            p.msg.err role="alert" { "加载失败 " (what) ": " (fmt::truncate(e, 120)) }
        }
    }
}

pub fn page(pd: &PageData) -> Markup {
    let power = pd.read(1).and_then(|d| on_off(&d["power_saver_mode"]));
    let fast = pd.read(2).and_then(|d| on_off(&d["fast_boot"]));
    let sleep = pd.read(3);
    let asleep = sleep.and_then(|d| d["enabled"].as_bool());
    let timeout = timeout_of(sleep);
    let token = pd.once();
    let hidden = html! { (pd.csrf()) (once_field(&token)) };
    // The switch keeps the router's timeout (the app sends it along).
    let with_timeout = html! { (hidden) input type="hidden" name="timeout" value=(timeout); };
    let post = |action, value| Act::Post { action, name: "on", value };
    let ask = format!("{AUTO_SLEEP}?on=1&timeout={timeout}");
    let sleep_on = if pd.remote { Act::Link(&ask) } else { post(AUTO_SLEEP, "1") };
    let mut timeouts: Vec<(String, String)> = Vec::with_capacity(TIMEOUTS.len() + 1);
    if !TIMEOUTS.iter().any(|(v, _)| *v == timeout) {
        timeouts.push((timeout.clone(), timeout_label(&timeout)));
    }
    timeouts.extend(TIMEOUTS.iter().map(|(v, l)| ((*v).to_owned(), (*l).to_owned())));
    html! {
        @for (i, what) in [(0, "charge control status"), (1, "power-save settings"), (2, "fast boot settings"), (3, "auto-sleep settings")] {
            (load_error(pd, i, what))
        }
        (charge(pd))
        (kit::section(None, Some(foot("限制数据通信速度以减少功耗并延长电池寿命。")),
            kit::switch_row(&hidden, "省电模式", power, post(POWER_SAVE, "1"), post(POWER_SAVE, "0"))))
        (kit::section(None, Some(foot("启用后，断电后将暂停至 RAM，以实现近乎即时的启动。禁用使用完全关闭（关闭时节省电池）。")),
            kit::switch_row(&hidden, "快速启动", fast, post(FAST_BOOT, "1"), post(FAST_BOOT, "0"))))
        (kit::section(None, Some(foot("当没有 WiFi 客户端连接并且空闲超时到期时，自动进入低功耗睡眠。按电源按钮唤醒。")), html! {
            (kit::switch_row(&with_timeout, "自动睡眠", asleep, sleep_on, post(AUTO_SLEEP, "0")))
            @if asleep == Some(true) {
                li.it {
                    label for="tmo" { "空闲超时" }
                    form.sf method="post" action=(AUTO_SLEEP) {
                        (hidden)
                        select #tmo name="timeout" {
                            @for (v, l) in &timeouts { option value=(v) selected[*v == timeout] { (l) } }
                        }
                        button.btn.sm type="submit" { "保存" }
                    }
                }
            }
        }))
        (kit::section(None, Some(foot("路由器将重新启动。这大约需要 60 秒。")), html! {
            li { a.it.bt href=(REBOOT) { "重启路由器" } }
        }))
        (kit::section(None, Some(foot("这将清除所有设置并恢复出厂默认设置。此操作无法撤消。")), html! {
            li { a.it.bt.danger href=(RESET) { "恢复出厂设置" } }
        }))
    }
}

// --- The switches and the idle timeout (`pages::confirm` kinds) ---

/// Power-save and Fast Boot: `on`. Auto Sleep: `on=0`; `on=1` with the
/// timeout it keeps; or only `timeout` (the picker: it stays on).
pub fn parse(kind: Kind, pairs: &[(String, String)]) -> Result<Vals, Bad> {
    if kind != Kind::AutoSleep {
        return on_fields(pairs);
    }
    let timeout = field(pairs, "timeout").filter(|t| timeout_ok(t)).map(str::to_owned);
    match (field(pairs, "on"), timeout) {
        (Some("0"), _) => Ok(vec![("on", "0".into())]),
        (Some("1"), Some(t)) => Ok(vec![("on", "1".into()), ("timeout", t)]),
        (None, Some(t)) => Ok(vec![("timeout", t)]),
        _ => Err(Bad::Request),
    }
}

const SLEEP_REMOTE: &str =
    "您已远程连接：当路由器处于睡眠状态时，只有按下其电源按钮才能访问此页面\u{2019}。";

/// Only Auto Sleep on, for a remote browser, is asked.
pub fn question(kind: Kind, input: &Input, remote: bool) -> Option<Question> {
    if kind != Kind::AutoSleep || input.val("on") != "1" || !remote {
        return None;
    }
    let lines = vec![
        format!(
            "当{}没有连接Wi-Fi设备时，路由器进入休眠状态。按电源按钮将其唤醒。",
            timeout_label(input.val("timeout"))
        ),
        SLEEP_REMOTE.into(),
    ];
    Some(super::confirm::asking("开启自动睡眠？", lines, "打开自动睡眠", remote, false))
}

/// The app's bodies; a failure in its words.
pub fn send(app: &AppState, kind: Kind, input: &Input, claim: &mut Claim) -> Result<(), (u16, String)> {
    let bit = if input.val("on") == "1" { "1" } else { "0" };
    let (path, body) = match kind {
        Kind::PowerSave => (POWER_SAVE_API, json!({"deviceInfoList": {"power_saver_mode": bit}})),
        Kind::FastBoot => (FAST_BOOT_API, json!({"fast_boot": bit})),
        _ if input.val("on") == "0" => (AUTO_SLEEP_API, json!({"enabled": false})),
        _ => (AUTO_SLEEP_API, json!({"enabled": true, "timeout": input.val("timeout")})),
    };
    claim.disarm();
    super::call(app, Method::Put, path, &body).map(|_| ()).map_err(|(status, e)| (status, format!("失败：{}", fmt::truncate(&e, 120))))
}

/// Back to the page, which reads the new state (the app shows no line);
/// a failure's line.
pub fn next(r: Result<(), &str>) -> Next {
    match r {
        Ok(()) => Next::To(PATH.into()),
        Err(msg) => Next::Line(PATH.into(), Some(false), msg.to_owned()),
    }
}

// --- Reboot and factory reset: the router password ---

/// The password sheets and their posts.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Power {
    Reboot,
    Reset,
    /// Mobile Network's "Reboot Required" (`pages::mobile`).
    Required,
}

impl Power {
    pub fn of(path: &str) -> Option<Power> {
        match path {
            REBOOT => Some(Power::Reboot),
            RESET => Some(Power::Reset),
            super::mobile::REBOOT => Some(Power::Required),
            _ => None,
        }
    }

    /// Its page and its post.
    pub fn path(self) -> &'static str {
        match self {
            Power::Reboot => REBOOT,
            Power::Reset => RESET,
            Power::Required => super::mobile::REBOOT,
        }
    }

    fn screen(self) -> &'static Screen {
        match self {
            Power::Reboot => &router::DEVICE_REBOOT,
            Power::Reset => &router::DEVICE_RESET,
            Power::Required => &router::REBOOT_REQUIRED,
        }
    }

    fn api(self) -> &'static str {
        match self {
            Power::Reset => "/api/device/factory-reset",
            _ => "/api/device/reboot",
        }
    }
}

pub fn reboot_sheet(pd: &PageData) -> Markup {
    sheet(pd, Power::Reboot)
}

pub fn reset_sheet(pd: &PageData) -> Markup {
    sheet(pd, Power::Reset)
}

/// The router password field: never filled by the page. `fill`: the
/// browser may fill the agent password it keeps (it is the router
/// password); else it and password managers are asked not to.
pub fn password_field(pd: &PageData, fill: bool) -> Markup {
    let err = pd.form.and_then(|f| f.error("password"));
    let (auto, ignore) = if fill { ("current-password", None) } else { ("off", Some("true")) };
    html! {
        li.it.fld {
            label.sr for="rpw" { "路由器管理密码" }
            input #rpw type="password" name="password" placeholder="路由器管理密码" autocomplete=(auto) required
                "data-1p-ignore"=[ignore] data-lpignore=[ignore] data-bwignore=[ignore]
                aria-invalid=[err.map(|_| "true")] aria-describedby=[err.map(|_| "rpw-err")];
            @if let Some(e) = err { p.ferr #rpw-err { (e) } }
        }
    }
}

/// The app's `PasswordConfirmView`: its message, "Router Password", the
/// red button; what happens (the page's footer) and, for a remote browser,
/// how it ends for this connection.
fn sheet(pd: &PageData, w: Power) -> Markup {
    let (message, label, what, remote) = match w {
        Power::Reset => (
            "这将删除所有设置。输入您的路由器密码进行确认。",
            "恢复出厂设置",
            "这将清除所有设置并恢复出厂默认设置。此操作无法撤消。",
            "您已远程连接：恢复出厂设置后，路由器可能无法返回您的尾网；然后只能通过 Wi-Fi 设备再次进行设置。",
        ),
        _ => ("输入您的路由器密码以确认重启。", "重启", "路由器将重新启动。这大约需要 60 秒。", REMOTE),
    };
    html! {
        form.rf.req method="post" action=(w.path()) {
            (pd.csrf())
            (once_field(&pd.once()))
            (kit::section(None, None, html! { li.it { p.cap { (message) } } }))
            (kit::section(None, Some(html! { (foot(what)) @if pd.remote { (foot(remote)) } }), password_field(pd, w != Power::Reset)))
            (kit::section(None, None, html! { li { button.it.cv.snd.danger type="submit" { (label) } } }))
        }
    }
}

/// After the call: the app's line and what happens next, with "Reload"
/// (the dashboard, which asks for a login once the router is back: a
/// restart ends every session).
fn done(nonce: &str, w: Power) -> Response {
    let reset = w == Power::Reset;
    let body = html! {
        div.narrow.rbt {
            (kit::icon(Icon::Power))
            @if reset {
                p.msg.ok role="status" { "恢复出厂设置已启动..." }
                p { "路由器清除其设置并重新启动。其 Wi-Fi 名称、Wi-Fi 密码和地址将恢复为出厂默认值，因此您可能需要先加入其默认 Wi-Fi 网络。" }
                p { "如果几分钟后管理网页仍无法打开，重置可能已删除 Agent，请按 README 说明重新安装。" }
            } @else {
                p.msg.ok role="status" { "路由器正在重启..." }
                p { "路由器将在约一分钟后恢复。重启会使所有浏览器退出登录，需要重新登录。" }
            }
            p.actions { a.btn href="/ui/" { "重新加载" } }
        }
    };
    let pc = PageCtx { title: if reset { "正在重置\u{2026}" } else { "正在重启\u{2026}" }, tab: Some(Tab::Router), nonce, ..PageCtx::default() };
    render::html(200, render::page(&pc, body))
}

/// `POST` of a password sheet: the session, the form and its CSRF token,
/// then the router password (a wrong one counts against the login rate
/// limit), then (single-use token: a double click sends once) the call;
/// the "Rebooting…" page as the answer, also to a replay.
pub async fn post(ctx: Ctx, req: Request, w: Power) -> Response {
    let nonce = CspNonce::of(&req);
    let peer = req.extensions().get::<Peer>().map(|p| PeerKey::from(p.0)).unwrap_or(PeerKey::UNKNOWN);
    let s = w.screen();
    let mut r = Req { query: None, ..Req::of(&ctx, &req) };
    let shown = Req { query: None, ..Req::of(&ctx, &req) };
    let (parts, body) = req.into_parts();
    let Some(p) = session::find_session(&ctx.app.auth, &parts.headers) else {
        let to_login = session::login_redirect(&HttpMethod::POST, w.path(), None);
        return session::reply_unread(&parts.headers, body, to_login).await;
    };
    let form = match session::read_form(&ctx, &parts.headers, body, &nonce).await {
        Ok(f) => f,
        Err(resp) => return resp,
    };
    if !session::verify_post(&ctx.app.auth, &parts.headers, &p, form.get("csrf")) {
        return render::form_refused(&nonce, w.path());
    }
    let refused = |msg: String| FormState { values: Vec::new(), errors: vec![("password", msg)] };
    match ctx.app.auth.check_password(form.get("password").unwrap_or(""), peer) {
        Ok(()) => {}
        Err(LoginError::RateLimited { retry_after }) => {
            let secs = retry_after_secs(retry_after);
            r.typed = Some(refused(format!("尝试次数过多，请在 {secs} 秒后重试。")));
            let mut resp = screen::show(&ctx, p, r, s, None, 429).await;
            resp.headers_mut().insert(RETRY_AFTER, HeaderValue::from(secs));
            return resp;
        }
        Err(_) => {
            r.typed = Some(refused("密码错误".into()));
            return screen::show(&ctx, p, r, s, None, 401).await;
        }
    }
    let lane = crate::server::lane(Method::Post, w.api());
    let token = form.get("once").unwrap_or("").to_owned();
    let (pw, n) = (p.clone(), nonce.clone());
    let work: OnceJob = Box::new(move |app: &AppState, claim| {
        claim.disarm();
        let out = super::call(app, Method::Post, w.api(), &json!({}));
        claim.set(match &out {
            Ok(_) => Token::Sent(claim.digest()),
            Err((status, e)) => Token::Failed(claim.digest(), *status, e.clone()),
        });
        match out {
            Ok(_) => done(&n, w),
            Err((status, e)) => screen::render(app, s, &pw, &r, Some(Notice::Failed(e)), None, status),
        }
    });
    let again = |text: &str| Some(Notice::Flash(Some(false), text.to_owned()));
    match job::once(&ctx, p.id, &nonce, &token, job::digest(&[w.path()]), lane, SETTLE_SLOW, work).await {
        Sent::Ran(resp) => resp,
        Sent::Done(Ok(())) => done(&nonce, w),
        Sent::Done(Err((status, e))) => screen::show(&ctx, p, shown, s, Some(Notice::Failed(e)), status).await,
        Sent::Expired => screen::show(&ctx, p, shown, s, again("该页面已过期，因此未发送任何内容。再试一次。"), 409).await,
        Sent::Busy => screen::show(&ctx, p, shown, s, again("失败：路由器正忙。几秒钟后重试。"), 503).await,
        Sent::Unknown => {
            let line = Notice::Flash(None, "路由器可能仍在处理它。重试前请检查。".into());
            screen::show(&ctx, p, shown, s, Some(line), 202).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::session::Form;

    fn pairs(q: &str) -> Vec<(String, String)> {
        Form::parse(q.as_bytes()).pairs().to_vec()
    }

    #[test]
    fn fields_and_bodies_follow_the_app() {
        let vals = |k, q| parse(k, &pairs(q)).ok();
        assert_eq!(vals(Kind::PowerSave, "on=1"), Some(vec![("on", "1".to_owned())]));
        assert_eq!(vals(Kind::FastBoot, "on=2"), None);
        assert_eq!(vals(Kind::AutoSleep, "on=1&timeout=10"), Some(vec![("on", "1".to_owned()), ("timeout", "10".to_owned())]));
        assert_eq!(vals(Kind::AutoSleep, "on=0&timeout=x"), Some(vec![("on", "0".to_owned())]));
        assert_eq!(vals(Kind::AutoSleep, "timeout=120"), Some(vec![("timeout", "120".to_owned())]));
        for bad in ["on=1", "on=1&timeout=0", "timeout=1441", "timeout=+5", "timeout=05000", "on=x&timeout=5", ""] {
            assert_eq!(vals(Kind::AutoSleep, bad), None, "{bad}");
        }
        assert_eq!((timeout_label("60"), timeout_label("15")), ("1 hour".to_owned(), "15 min".to_owned()));
        assert_eq!(timeout_of(Some(&json!({"timeout": "-1"}))), "5");
        assert_eq!(timeout_of(Some(&json!({"timeout": "30"}))), "30");
        assert_eq!((on_off(&json!("1")), on_off(&json!("0")), on_off(&json!(""))), (Some(true), Some(false), None));
        // Asked only when a remote browser turns Auto Sleep on.
        let on = Input::of(&[("on", "1"), ("timeout", "20")]);
        assert!(question(Kind::AutoSleep, &on, false).is_none() && question(Kind::PowerSave, &on, true).is_none());
        let q = question(Kind::AutoSleep, &on, true).unwrap();
        assert_eq!((q.title.as_str(), q.label), ("开启自动睡眠？", Some("打开自动睡眠")));
        assert!(q.lines[0].contains("for 20 min,") && q.lines[1].starts_with("您已远程连接："));
        assert!(question(Kind::AutoSleep, &Input::of(&[("timeout", "20")]), true).is_none());
        assert_eq!(Power::of(REBOOT).map(Power::api), Some("/api/device/reboot"));
        assert_eq!(Power::of(RESET).map(Power::api), Some("/api/device/factory-reset"));
        assert_eq!(Power::of(super::super::mobile::REBOOT), Some(Power::Required));
    }
}
