//! Login and logout (no JavaScript). The login form is bound to a pre-login
//! cookie by an HMAC'd, time-limited token, so it needs no server state; a
//! successful login rotates every presented session cookie.

use axum::extract::{Request, State};
use axum::http::header::{HeaderMap, HeaderValue, RETRY_AFTER};
use axum::response::Response;
use maud::{html, Markup};

use crate::auth::{retry_after_secs, Kind, LoginError, PeerKey};
use crate::httpd::guard::ui_post_allowed;
use crate::httpd::{api, Ctx, Peer};
use crate::web::kit::{self, Icon, Tab};
use crate::web::render::{self, PageCtx};
use crate::web::session::{self, Session};
use crate::web::CspNonce;

/// A message above the login form.
enum Notice {
    Info(&'static str),
    Error(String),
}

/// `GET /ui/login[?next=…][&expired=1]`. A live session goes straight to
/// `next`; the marker hop (`session::without_session`) applies here too, so
/// a launch from outside the site that lands on the login page finds the
/// session as well.
pub async fn form(State(ctx): State<Ctx>, req: Request) -> Response {
    let nonce = CspNonce::of(&req);
    let query = req.uri().query();
    let next = session::query_param(query, "next");
    let next = session::safe_next(next.as_deref()).to_owned();
    if session::find_session(&ctx.app.auth, req.headers()).is_some() {
        return session::see_other(&next);
    }
    let notice = (session::query_param(query, "expired").as_deref() == Some("1"))
        .then_some(Notice::Info("您的会话已过期。一切都没有改变。"));
    let (parts, _) = req.into_parts();
    let hint = super::https::login_hint(&ctx, &parts);
    session::without_session(&parts.headers, &parts.method, parts.uri.path(), parts.uri.query(), &nonce, |_| {
        login_page(&ctx, &parts.headers, &nonce, 200, &next, notice, hint.as_deref())
    })
}

/// The login form with a fresh `login_csrf`, re-setting the pre-login
/// cookie (a well-formed one is reused, so a second tab keeps working).
/// `hint`: the https address, shown over plain http while HTTPS is on.
fn login_page(ctx: &Ctx, headers: &HeaderMap, nonce: &str, status: u16, next: &str, notice: Option<Notice>, hint: Option<&str>) -> Response {
    let auth = &ctx.app.auth;
    let pre = match session::pre_cookie(headers) {
        Some(p) => p.to_owned(),
        None => match auth.new_pre_token() {
            Ok(s) => s.expose().to_owned(),
            Err(_) => return entropy_page(nonce),
        },
    };
    let Ok(token) = auth.login_form_token(&pre) else {
        return entropy_page(nonce);
    };
    let mut resp = render::html(status, login_markup(nonce, &token, next, notice.as_ref(), hint));
    session::add_cookie(&mut resp, &session::set_pre(&pre));
    resp
}

fn login_markup(nonce: &str, token: &str, next: &str, notice: Option<&Notice>, hint: Option<&str>) -> Markup {
    let failed = matches!(notice, Some(Notice::Error(_)));
    let body = html! {
        div.narrow {
            // The app's header; the web's own field and button words.
            div.lh {
                (kit::icon(Icon::Router))
                h1 { "ZTE U60 Pro" }
                p { "路由器配套工具" }
            }
            @match notice {
                Some(Notice::Info(msg)) => p.msg.info role="status" { (msg) },
                Some(Notice::Error(msg)) => p #login-error .msg.err role="alert" { (msg) },
                None => {},
            }
            form.panel method="post" action="/ui/login" {
                input type="hidden" name="login_csrf" value=(token);
                input type="hidden" name="next" value=(next);
                // There is no user name; a fixed one lets password managers
                // save and fill the password. The server ignores it.
                input type="text" name="username" value="agent" autocomplete="username" readonly hidden;
                div.field {
                    label for="password" { "Agent 密码" }
                    input #password type="password" name="password" autocomplete="current-password" required autofocus
                        aria-describedby=[failed.then_some("login-error")] aria-invalid=[failed.then_some("true")];
                }
                button.btn.btn-wide type="submit" { "登录" }
            }
            @if let Some(url) = hint {
                p.note.ctr { "要将此 Web UI 安装为应用程序，请信任此设备上的路由器’s 证书颁发机构（设置 › HTTPS 并作为应用程序安装 › 在您的设备上安装），然后打开 " a href=(url) rel="noreferrer" { (url) } "." }
            }
        }
    };
    // The app's header is the page's heading: no brand over it.
    render::page(&PageCtx { title: "登录", nonce, heading: Some(html! {}), ..PageCtx::default() }, body)
}

fn entropy_page(nonce: &str) -> Response {
    render::error(503, "重试", "路由器现在无法创建安全会话。稍后再试一次。", nonce)
}

/// `POST /ui/login`: origin, pre-login cookie and form token first, then
/// the password (rate limited per peer and globally).
pub async fn submit(State(ctx): State<Ctx>, req: Request) -> Response {
    let nonce = CspNonce::of(&req);
    let peer = req.extensions().get::<Peer>().map(|p| PeerKey::from(p.0)).unwrap_or(PeerKey::UNKNOWN);
    let (parts, body) = req.into_parts();
    let headers = &parts.headers;
    if !ui_post_allowed(&api::req_headers(headers)) {
        return session::reply_unread(headers, body, render::form_refused(&nonce, "/ui/login")).await;
    }
    let form = match session::read_form(&ctx, headers, body, &nonce).await {
        Ok(f) => f,
        Err(resp) => return resp,
    };
    let next = session::safe_next(form.get("next")).to_owned();
    let auth = &ctx.app.auth;
    let hint = super::https::login_hint(&ctx, &parts);
    let error = |status, msg: &str| login_page(&ctx, headers, &nonce, status, &next, Some(Notice::Error(msg.to_owned())), hint.as_deref());

    let Some(pre) = session::pre_cookie(headers) else {
        // The pre-login cookie lives as long as the form token, so a form
        // left open too long comes back without it.
        if form.get("login_csrf").is_some_and(|t| auth.login_form_token_fresh(t)) {
            return error(403, "Cookies must be enabled to log in.");
        }
        return error(403, "登录表单已过期。再试一次。");
    };
    if !form.get("login_csrf").is_some_and(|t| auth.verify_login_form_token(pre, t)) {
        return error(403, "登录表单已过期。再试一次。");
    }
    let password = form.get("password").unwrap_or("");
    let replacing = session::session_cookies(headers);
    let replaced = session::find_session(auth, headers).map(|p| p.id);
    match auth.login(password, Kind::Browser, peer, &replacing) {
        Ok(issued) => {
            // The replaced session's AT Terminal history, USSD answers and
            // drafts (typed passwords) go with it.
            if let Some(id) = replaced {
                ctx.at.clear(id);
                ctx.ussd.clear(id);
                ctx.drafts.clear(id);
            }
            let mut resp = session::see_other(&next);
            session::add_cookie(&mut resp, &session::set_session(issued.token.expose(), issued.max_age.as_secs()));
            session::add_cookie(&mut resp, &session::set_marker(issued.max_age.as_secs()));
            session::add_cookie(&mut resp, session::CLEAR_PRE);
            resp
        }
        Err(LoginError::RateLimited { retry_after }) => {
            let secs = retry_after_secs(retry_after);
            let mut resp = error(429, &format!("尝试次数过多，请在 {secs} 秒后重试。"));
            resp.headers_mut().insert(RETRY_AFTER, HeaderValue::from(secs));
            resp
        }
        Err(LoginError::Entropy) => entropy_page(&nonce),
        Err(LoginError::BadPassword | LoginError::NoPassword) => error(401, "密码错误。"),
    }
}

/// `GET /ui/logout`: a one-button form (Settings › Session › Log out).
pub async fn logout_form(State(ctx): State<Ctx>, Session(p): Session, req: Request) -> Response {
    let nonce = CspNonce::of(&req);
    let body = html! {
        div.narrow {
            form.panel method="post" action="/ui/logout" {
                (render::csrf_field(&ctx.app.auth, &p))
                p { "在此浏览器中注销路由器？" }
                p.actions {
                    button.btn.btn-danger type="submit" { "退出登录" }
                    a.btn.btn-quiet href="/ui/settings" { "取消" }
                }
            }
        }
    };
    let pc = PageCtx {
        title: "退出登录",
        tab: Some(Tab::Settings),
        nonce: &nonce,
        lead: kit::back("/ui/settings", "设置"),
        ..PageCtx::default()
    };
    render::html(200, render::page(&pc, body))
}

/// `POST /ui/logout`: ends the session (its live streams end too) and
/// clears the cookie and the marker. Without a valid session there is no session to end: a
/// same-origin post still clears a stale cookie it sent; any other post is
/// refused without touching cookies (a cross-site form must not be able to
/// delete the session cookie, which `SameSite=Strict` keeps it from sending).
pub async fn logout(State(ctx): State<Ctx>, req: Request) -> Response {
    let nonce = CspNonce::of(&req);
    let (parts, body) = req.into_parts();
    let headers = &parts.headers;
    let auth = &ctx.app.auth;
    let Some(p) = session::find_session(auth, headers) else {
        if !ui_post_allowed(&api::req_headers(headers)) {
            return session::reply_unread(headers, body, render::form_refused(&nonce, "/ui/logout")).await;
        }
        let mut resp = session::reply_unread(headers, body, session::see_other("/ui/login")).await;
        if !session::session_cookies(headers).is_empty() {
            session::add_cookie(&mut resp, session::CLEAR_SESSION);
        }
        if session::has_marker(headers) {
            session::add_cookie(&mut resp, session::CLEAR_MARKER);
        }
        return resp;
    };
    let form = match session::read_form(&ctx, headers, body, &nonce).await {
        Ok(f) => f,
        Err(resp) => return resp,
    };
    if !session::verify_post(auth, headers, &p, form.get("csrf")) {
        return render::form_refused(&nonce, "/ui/logout");
    }
    auth.logout_id(p.id);
    // The AT Terminal's history, the USSD answers and the drafts (typed
    // passwords) go with the session.
    ctx.at.clear(p.id);
    ctx.ussd.clear(p.id);
    ctx.drafts.clear(p.id);
    let mut resp = session::see_other("/ui/login");
    session::add_cookie(&mut resp, session::CLEAR_SESSION);
    session::add_cookie(&mut resp, session::CLEAR_MARKER);
    resp
}
