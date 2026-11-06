//! Settings › "HTTPS 与安装为应用" (owner decision 59; a web
//! addition, the app has no counterpart): LAN HTTPS with the router's own
//! certificate authority (`crate::tls`), so Chrome, Edge and Samsung
//! Internet can install the web UI as an app (Safari adds it to the Home
//! Screen or the Dock over http too).
//!
//! - Settings shows the state: off with "Turn On HTTPS" (and "Reset HTTPS"
//!   when files are left that could not be used); on with the
//!   certificate's end date, the https addresses, "Install on Your
//!   Devices" (`PAGE`: the CA's fingerprint, what it can vouch for, the
//!   downloads and the steps per platform), "Try Again" when not
//!   listening, "Renew Certificate" when fewer than 30 days are left by
//!   the router's clock, and "Reset HTTPS".
//! - Turn On, Renew and Reset are questions (`GET`) whose form carries a
//!   single-use token; the post does it in a gate task and comes back to
//!   Settings with its line. Turn On and Renew show the router's clock
//!   (the certificates are dated from it) and say when it could not be
//!   checked over the internet.
//! - The downloads (`tls::DOWNLOAD_CRT`, `tls::DOWNLOAD_PROFILE`) are
//!   public while HTTPS is on (the CA certificate is public; a phone
//!   installing it may not be logged in) and never stored by caches.
//! - Over plain http, Settings and the login page point to the https
//!   address (once the device trusts the CA: Install on Your Devices).

use std::sync::Arc;

use axum::body::Body;
use axum::extract::Request;
use axum::http::header::{HeaderValue, CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::Method as HttpMethod;
use axum::response::Response;
use maud::{html, Markup};

use crate::handlers::AppState;
use crate::httpd::gate::Lane;
use crate::httpd::tls::Secure;
use crate::httpd::{Ctx, Peer};
use crate::tls::{Status, Utc, DOWNLOAD_CRT, DOWNLOAD_PROFILE};
use crate::web::action::{self, Ask};
use crate::web::job::{self, OnceJob, Sent};
use crate::web::kit::{self, Icon, Tab};
use crate::web::render::{self, PageCtx};
use crate::web::screen::PageData;
use crate::web::session;
use crate::web::store::{Token, SETTLE};
use crate::web::{sms, CspNonce};

/// "Install on Your Devices".
pub const PAGE: &str = "/ui/settings/https";
pub const ON: &str = "/ui/settings/https/on";
pub const RENEW: &str = "/ui/settings/https/renew";
pub const RESET: &str = "/ui/settings/https/reset";
const SETTINGS: &str = "/ui/settings";
const TITLE: &str = "HTTPS 与安装为应用";

/// "Oct 24, 2026" in the router's time zone.
fn date(t: i64) -> String {
    sms::day(t, sms::local_offset(t))
}

/// "Oct 24, 2026, 14:03 UTC".
fn utc_stamp(t: i64) -> String {
    let s = t.rem_euclid(86_400);
    format!("{}, {:02}:{:02} UTC", sms::day(t, 0), s / 3600, s / 60 % 60)
}

/// Whether this request came over HTTPS.
pub fn is_secure<B>(req: &axum::http::Request<B>) -> bool {
    req.extensions().get::<Secure>().is_some()
}

/// The https addresses as links (a row each).
fn url_rows(s: &Status) -> Markup {
    html! {
        @for url in s.urls() {
            li { a.it href=(url) rel="noreferrer" { (kit::icon(Icon::Lock)) span { (url) } } }
        }
    }
}

/// Over plain http with HTTPS on (and not over Tailscale, whose names the
/// certificate does not hold): where the installable copy is (the address,
/// else the name; plain http answers only by address, `HostPolicy`).
pub fn hint(s: &Status, secure: bool, remote: bool) -> Option<String> {
    if !s.on || secure || remote {
        return None;
    }
    s.urls().first().cloned()
}

/// The Settings section.
pub fn section(pd: &PageData) -> Markup {
    let s = pd.app.tls.status();
    let error = html! { @if let Some(e) = &s.error { p.foot role="status" { (e) } } };
    if !s.on {
        let foot = html! {
            p.foot {
                "通过端口 " (s.port) " 提供 HTTPS 管理网页，证书由路由器自己的证书颁发机构签发，以便 Chrome、Edge 和 Samsung Internet 将其安装为应用。每部手机或计算机只需信任该机构一次。"
            }
            @if let Some(ca) = &s.kept {
                p.foot { "证书颁发机构 " (ca) "仍保存在路由器上。开启 HTTPS 会由它签发新证书，已信任它的设备无需重新设置。" }
            }
            (error)
        };
        return kit::section(Some(TITLE), Some(foot), html! {
            (kit::kv_row(Some(Icon::Lock), "HTTPS", "关闭"))
            li { a.it href=(ON) { span { "开启HTTPS" } } }
            @if s.kept.is_some() || s.error.is_some() { li { a.it.danger href=(RESET) { span { "复位HTTPS" } } } }
        });
    }
    let state = if s.listening.is_some() { "开启" } else { "已开启 · 未监听" };
    let foot = html! {
        @if pd.secure {
            p.foot { "你正在使用 HTTPS，可以通过浏览器菜单安装此管理网页。" }
        } @else if let Some(url) = hint(&s, pd.secure, pd.remote) {
            p.foot { "此页面使用明文 HTTP。请先按“在你的设备上安装”说明信任路由器证书颁发机构，然后打开  " a href=(url) rel="noreferrer" { (url) } " 以将管理网页安装为应用。" }
        }
        (error)
    };
    kit::section(Some(TITLE), Some(foot), html! {
        (kit::kv_row(Some(Icon::Lock), "HTTPS", state))
        @if s.listening.is_none() { li { a.it href=(ON) { span { "重试" } } } }
        @if let Some(t) = s.not_after {
            (kit::kv_row(None, if s.renew_due { "证书过期" } else { "证书有效期至" }, &date(t)))
        }
        (url_rows(&s))
        (kit::link_row(PAGE, Some(Icon::Shield), "在您的设备上安装"))
        @if s.renew_due { li { a.it href=(RENEW) { span { "更新证书" } } } }
        li { a.it.danger href=(RESET) { span { "复位HTTPS" } } }
    })
}

/// A numbered list of steps in one row.
fn steps(items: &[Markup]) -> Markup {
    html! { li.it { ol.steps { @for i in items { li { (i) } } } } }
}

/// "Install on Your Devices": the CA, the downloads and how each platform
/// trusts it, then how to install the app.
pub fn install(pd: &PageData) -> Markup {
    let s = pd.app.tls.status();
    if !s.on {
        return html! {
            div.vc {
                (kit::empty_state(Icon::Lock, "HTTPS 已关闭", "首先在“设置”中将其打开。"))
                p.actions.ctr { a.btn href=(SETTINGS) { "设置" } }
            }
        };
    }
    let ca = s.ca_name.as_str();
    let fp_foot = html! {
        @if pd.secure {
            p.foot { "本页通过路由器的 HTTPS 连接提供，以下是其证书指纹。请核对设备上显示的 SHA-256 指纹是否一致，大小写、空格和冒号可忽略。" }
        } @else {
            p.foot {
                "信任前，请核对设备显示的 SHA-256 指纹是否相同，大小写、空格和冒号可忽略。此页面使用明文 HTTP，同一网络上的人可能同时篡改页面和下载内容。请通过已信任此证书机构的设备，用 HTTPS 打开本页核对；或通过 SSH 执行 "
                span.mono { "sha256sum " (DIR_CA) }
                "（通过 SSH，使用相同的十六进制数字）。"
            }
        }
        p.foot { "设备显示位置：iPhone 和 iPad：设置 › 配置文件已下载 › 更多详细信息 › 证书，然后点击安装； Mac：Keychain 访问›证书； Android：安装后，设置 › 受信任凭据 › 用户 › 证书（如果不同，请将其删除）。" }
        p.foot { "Windows 显示 SHA-1 “Thumbprint”（详细信息选项卡）： " span.mono { (s.sha1.replace(':', "").to_ascii_lowercase()) } }
    };
    html! {
        (kit::section(Some("证书颁发机构"), Some(fp_foot), html! {
            (kit::kv_row(None, "名称", ca))
            (kit::kv_row(None, "证书约束范围", &s.covers.join(" 和 ")))
            // Line breaks only between the byte pairs; screen readers get
            // the whole value once.
            li.it { span.cvb { span { "SHA-256 指纹" } span.sr { (s.sha256) } small.mono.fp translate="no" aria-hidden="true" { @for (i, pair) in s.sha256.split(':').enumerate() { @if i > 0 { ":" wbr; } (pair) } } } }
        }))
        (kit::section(Some("下载"), Some(html! { p.foot { "这是同一公开证书的两种格式，不包含私密信息。" } }), html! {
            li { a.it href=(DOWNLOAD_PROFILE) download="u60-pro-ca.mobileconfig" { (kit::icon(Icon::Doc)) span { "iPhone 和 iPad 的配置文件" } } }
            li { a.it href=(DOWNLOAD_CRT) download="u60-pro-ca.crt" { (kit::icon(Icon::Doc)) span { "适用于 Android、Mac 和 Windows 的证书 (.crt)" } } }
        }))
        (kit::section(Some("iPhone 和 iPad"), None, steps(&[
            html! { "在 Safari 中，点击 " b { "iPhone 和 iPad 的配置文件" } "，然后 " b { "允许" } "." },
            html! { "打开 " b { "设置" } " \u{203a} " b { "已下载描述文件" } " \u{203a} " b { "安装" } "." },
            html! { b { "设置" } " \u{203a} " b { "通用" } " \u{203a} " b { "关于" } " \u{203a} " b { "证书信任设置" } "：为以下证书开启完全信任：" b { (ca) } "." },
        ])))
        (kit::section(Some("安卓"), Some(html! { p.foot { "菜单名称因制造商而异：搜索 “CA 证书 ” 的设置。" } }), steps(&[
            html! { "点击 " b { "证书（.crt）" } " 以下载。" },
            html! { b { "设置" } " \u{203a} " b { "安全与隐私" } " \u{203a} " b { "更高的安全性和隐私性" } " \u{203a} " b { "加密和凭证" } " \u{203a} " b { "安装证书" } " \u{203a} " b { "CA证书" } " \u{203a} " b { "无论如何安装" } "，使用锁屏密码确认，然后选择下载的文件。" },
        ])))
        (kit::section(Some("Mac"), None, steps(&[
            html! { "下载证书（.crt）并打开它：Keychain Access 将其添加到您的登录钥匙串中。" },
            html! { "双击 " b { (ca) } " \u{203a} " b { "信任" } " \u{203a} " b { "使用此证书时：始终信任" } "，然后关闭窗口并确认。" },
        ])))
        (kit::section(Some("Windows"), Some(html! { p.foot { "Firefox 120 及更高版本使用您添加到 Windows、macOS 或 Android 的证书。在 Linux 上，或关闭该设置： 设置 › 隐私和安全 › 证书 › 查看证书 … › 权限 › 导入 …，并信任它的网站。" } }), steps(&[
            html! { "下载证书（.crt）并打开› " b { "安装证书" } " \u{203a} " b { "当前用户" } "." },
            html! { b { "将所有证书放入以下存储中" } " \u{203a} " b { "浏览" } " \u{203a} " b { "受信任的根证书颁发机构" } " \u{203a} " b { "完成" } " \u{203a} " b { "是的" } "." },
        ])))
        (kit::section(Some("然后安装应用程序"), Some(html! {
            p.foot { "打开这些地址之一并登录。Chrome：菜单› " b { "安装" } "（Android：" b { "添加到主屏幕" } "）。iPhone 的 Safari：••• › 共享 › 查看更多 › " b { "添加到主屏幕" } "。Mac 的 Safari：文件 › " b { "添加到 Dock" } "." }
            p.foot { "使用自己的 DNS 的设备（Android’s Private DNS 设置为提供商，浏览器中的 DNS over HTTPS，VPN）可能找不到 " (s.name) "；IP 地址仍然可用。每个地址的登录状态独立保存。" }
        }), url_rows(&s)))
    }
}

/// The root's certificate on the router (DER: its SHA-256 is the
/// fingerprint), for the SSH check.
const DIR_CA: &str = "/data/zte-agent-tls/ca.crt";

/// Turn On, Renew, Reset.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Post {
    On,
    Renew,
    Reset,
}

impl Post {
    pub fn of(path: &str) -> Option<Post> {
        match path {
            ON => Some(Post::On),
            RENEW => Some(Post::Renew),
            RESET => Some(Post::Reset),
            _ => None,
        }
    }

    fn path(self) -> &'static str {
        match self {
            Post::On => ON,
            Post::Renew => RENEW,
            Post::Reset => RESET,
        }
    }
}

/// The clock line of Turn On and Renew.
fn clock_line(c: Utc) -> String {
    match c {
        Utc::Verified(t) => format!("证书的日期来自路由器的时钟，通过互联网检查：{}。", utc_stamp(t)),
        _ => format!(
            "证书的日期是从路由器的时钟开始的：{}。无法通过互联网进行检查；如果此日期错误，请取消并稍后重试。",
            utc_stamp(c.secs())
        ),
    }
}

/// That plain http keeps working, on the port it is bound to.
fn plain_http() -> String {
    format!("端口 {} 上的纯 http 继续工作，对于应用程序也是如此。", crate::httpd::http_port())
}

/// The question: its title, lines and button (none: nothing to do).
fn question(app: &AppState, k: Post, secure: bool) -> (String, Vec<String>, Option<&'static str>) {
    let tls = &app.tls;
    let s = tls.status();
    let port = s.port;
    match k {
        Post::On if s.on && s.listening.is_some() => ("HTTPS 已开启".into(), Vec::new(), None),
        Post::On if s.on => (
            "重新开始监听？".into(),
            vec![
                format!("HTTPS 已开启，但路由器未监听 {port} 端口{}", s.error.as_deref().map_or(".".into(), |e| format!(": {e}"))),
                "请停止占用此端口的程序后重试。其余设置不会改变。".into(),
            ],
            Some("重试"),
        ),
        Post::Renew if !s.on => ("HTTPS 已关闭".into(), Vec::new(), None),
        Post::Reset if !s.on && s.kept.is_none() && s.error.is_none() => ("HTTPS 已关闭".into(), Vec::new(), None),
        Post::On | Post::Renew if s.clock.usable().is_none() => {
            ("路由器时钟尚未设置".into(), vec![crate::tls::CLOCK_UNSET.into()], None)
        }
        Post::On if s.kept.is_some() => (
            "开启 HTTPS？".into(),
            vec![
                format!(
                    "路由器将为其 LAN 地址及 {} 签发新证书，签发机构为 {}，并通过 {port} 端口提供 HTTPS 管理网页。已信任该机构的设备无需重新设置。",
                    tls.name(),
                    s.kept.as_deref().unwrap_or_default()
                ),
                clock_line(s.clock),
                plain_http(),
            ],
            Some("开启HTTPS"),
        ),
        Post::On => (
            "开启 HTTPS？".into(),
            vec![
                format!(
                    "路由器为其 LAN 地址和 {} 创建自己的证书颁发机构和证书，并通过端口 {port} 上的 HTTPS 提供此 Web UI。",
                    tls.name()
                ),
                format!("然后，每部手机或计算机只需信任此证书颁发机构一次（请参见“在您的设备上安装”）。它只能为路由器的 LAN 地址和{}（及其下的名称）提供担保，而不能为其他网站或设备提供担保。如果您稍后更改 LAN 地址，则只有 {} 继续工作，直到您重置 HTTPS。", tls.name(), tls.name()),
                clock_line(s.clock),
                plain_http(),
            ],
            Some("开启HTTPS"),
        ),
        Post::Renew => (
            "更新证书？".into(),
            vec![
                format!("路由器从同一证书颁发机构颁发未来 {} 天的新证书；信任它的设备不需要任何新东西。", crate::tls::SERVER_DAYS - 2),
                clock_line(s.clock),
            ],
            Some("更新证书"),
        ),
        Post::Reset if !s.on => (
            "重置HTTPS？".into(),
            vec![
                format!("这将删除路由器{}上留下的证书文件。", s.kept.as_deref().map_or(String::new(), |c| format!(" ({c})"))),
                "从每台信任该证书颁发机构的设备上删除该证书颁发机构。如果您再次打开 HTTPS，则每台设备都必须信任新的证书颁发机构。".into(),
            ],
            Some("复位HTTPS"),
        ),
        Post::Reset => {
            let mut lines = vec![
                format!("这将删除路由器的证书颁发机构 ({}) 和证书，并停止端口 {port} 上的 HTTPS，也适用于通过其打开的页面。", s.ca_name),
                "从信任它的每个设备中删除该证书颁发机构：旧证书已无法继续使用。如果您再次打开 HTTPS，则每台设备都必须信任新的证书颁发机构。".into(),
            ];
            if secure {
                lines.push(format!("此页面通过 HTTPS 打开并停止工作：继续使用 http 地址（端口 {}）。", crate::httpd::http_port()));
            }
            ("重置HTTPS？".into(), lines, Some("复位HTTPS"))
        }
    }
}

/// `GET` of a question: its form carries a new single-use token.
pub fn get(ctx: &Ctx, p: &crate::auth::Principal, req: &Request, k: Post) -> Response {
    let nonce = CspNonce::of(req);
    let (title, lines, label) = question(&ctx.app, k, is_secure(req));
    let token = label.map(|_| ctx.once.fresh(&ctx.app.auth, p.id)).unwrap_or_default();
    let hidden = [("once", token.as_str())];
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    let a = Ask { title: &title, lines: &lines, action: k.path(), hidden: &hidden, label, danger: k == Post::Reset, cancel: SETTINGS };
    let pc = PageCtx {
        title: &title,
        tab: Some(Tab::Settings),
        nonce: &nonce,
        lead: kit::back(SETTINGS, "设置"),
        heading: Some(html! { h1 { "确认" } }),
        ..PageCtx::default()
    };
    render::html(200, render::page(&pc, action::ask(render::csrf_field(&ctx.app.auth, p), &a)))
}

/// 303 to Settings with `text` for this session (`?r=<id>`).
fn line(ctx_flash: &crate::web::store::Flash, app: &AppState, owner: crate::auth::SessionId, ok: Option<bool>, text: String) -> Response {
    let Ok(id) = app.auth.random_hex(8) else { return session::see_other(SETTINGS) };
    ctx_flash.insert(id.clone(), owner, (ok, text), app.auth.now());
    session::see_other(&format!("{SETTINGS}?r={id}"))
}

fn done(k: Post) -> &'static str {
    match k {
        Post::On => "HTTPS 已开启。下一步：在你的设备上安装。已信任路由器证书颁发机构的设备无需重新设置。",
        Post::Renew => "证书更新。",
        Post::Reset => "HTTPS 已重置，请从你的设备上移除旧的证书颁发机构。",
    }
}

/// `POST` of a question: the session (else the login), the form, origin and
/// CSRF token, `confirm=1`, then once per token: the change, and Settings
/// with its line.
pub async fn post(ctx: Ctx, req: Request, k: Post) -> Response {
    let nonce = CspNonce::of(&req);
    let (parts, body) = req.into_parts();
    let Some(p) = session::find_session(&ctx.app.auth, &parts.headers) else {
        let to_login = session::login_redirect(&HttpMethod::POST, k.path(), None);
        return session::reply_unread(&parts.headers, body, to_login).await;
    };
    let form = match session::read_form(&ctx, &parts.headers, body, &nonce).await {
        Ok(f) => f,
        Err(resp) => return resp,
    };
    if !session::verify_post(&ctx.app.auth, &parts.headers, &p, form.get("csrf")) {
        return render::form_refused(&nonce, SETTINGS);
    }
    if form.get("confirm") != Some("1") {
        return session::see_other(k.path());
    }
    let token = form.get("once").unwrap_or("").to_owned();
    let flash = Arc::clone(&ctx.flash);
    let owner = p.id;
    let work: OnceJob = Box::new(move |app: &AppState, claim| {
        claim.disarm();
        let r = match k {
            Post::On => app.tls.turn_on(),
            Post::Renew => app.tls.renew(),
            Post::Reset => app.tls.reset(),
        };
        claim.set(match &r {
            Ok(()) => Token::Sent(claim.digest()),
            Err(e) => Token::Failed(claim.digest(), 500, e.clone()),
        });
        match r {
            Ok(()) => line(&flash, app, owner, Some(true), done(k).into()),
            Err(e) => line(&flash, app, owner, Some(false), e),
        }
    });
    let note = |ok: Option<bool>, text: &str| line(&ctx.flash, &ctx.app, owner, ok, text.to_owned());
    match job::once(&ctx, owner, &nonce, &token, job::digest(&[k.path()]), Lane::General, SETTLE, work).await {
        Sent::Ran(resp) => resp,
        Sent::Done(Ok(())) => note(Some(true), done(k)),
        Sent::Done(Err((_, e))) => note(Some(false), &e),
        Sent::Expired => note(Some(false), "该页面已过期，因此没有任何更改。再试一次。"),
        Sent::Unknown => note(None, "路由器可能仍在处理它。重试前请检查。"),
        Sent::Busy => note(Some(false), "失败：路由器正忙。几秒钟后重试。"),
    }
}

fn plain(status: u16, text: &'static str) -> Response {
    let mut resp = Response::new(Body::from(text));
    *resp.status_mut() = axum::http::StatusCode::from_u16(status).unwrap_or_default();
    resp.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("text/plain; charset=utf-8"));
    resp
}

/// The public downloads (static files: no session, no nonce): the file
/// while HTTPS is on, else plain-text 404; 405 for anything but GET and
/// HEAD. `None` for any other path.
pub fn download(ctx: &Ctx, method: &HttpMethod, path: &str) -> Option<Response> {
    let crt = match path {
        DOWNLOAD_CRT => true,
        DOWNLOAD_PROFILE => false,
        _ => return None,
    };
    if method != HttpMethod::GET && method != HttpMethod::HEAD {
        return Some(plain(405, "方法不允许"));
    }
    let file = if crt {
        ctx.app.tls.ca_der().map(|der| (der, "application/x-x509-ca-cert", "u60-pro-ca.crt"))
    } else {
        ctx.app.tls.mobileconfig().map(|p| (p.into_bytes(), "application/x-apple-aspen-config", "u60-pro-ca.mobileconfig"))
    };
    let Some((body, ct, file)) = file else { return Some(plain(404, "未找到")) };
    let mut resp = Response::new(Body::from(body));
    let h = resp.headers_mut();
    h.insert(CONTENT_TYPE, HeaderValue::from_static(ct));
    h.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if let Ok(v) = HeaderValue::from_str(&format!("attachment; filename=\"{file}\"")) {
        h.insert(CONTENT_DISPOSITION, v);
    }
    Some(resp)
}

/// The login page's line over plain http while HTTPS is on.
pub fn login_hint(ctx: &Ctx, parts: &axum::http::request::Parts) -> Option<String> {
    let ext = &parts.extensions;
    hint(&ctx.app.tls.status(), ext.get::<Secure>().is_some(), session::is_remote(ext.get::<Peer>()))
}
