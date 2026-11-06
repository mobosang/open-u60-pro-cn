//! The app's USB Mode sheet (`USBModeSheetView`, which it opens over any
//! tab when a USB-C cable is attached) as a dashboard card after the
//! banners: "USB-C Connected", "Fast Charging" (powerbank mode) and
//! "Dismiss". Only the dashboard shows it: nothing samples `usb` for the
//! other pages.
//!
//! - The text and state are the live fragment `card-usb` (topics `usb`
//!   and `charger`): class `on` while the card is shown, `pb` while fast
//!   charging is active.
//! - The two forms follow it, outside every fragment and always in the
//!   page; CSS shows the card only while the fragment is `on`, and "Fast
//!   Charging" only without `pb`. A cable attached while the page is open
//!   gets the same buttons as one attached at page load.
//! - "Dismiss" hides the card for this session until the cable is seen
//!   detached (`Ctx::dismissed`). The page load and the stream of a page
//!   with the `usb` topic (`screen`, `live`), never a fragment, read the
//!   flag into `Views::usb_dismissed`; a detached reading in any session's
//!   page ends every session's dismissal (`dismissed`), so the next attach
//!   shows the card again, as the app re-presents its sheet. A detach
//!   while no dashboard is open is not seen.
//!
//! Tools › USB Mode (`USBModeView`, read at page load) shows the cable, its
//! CC state and USB mode, and the "Fast Charging (Powerbank)" switch, the
//! only way to turn fast charging off (as in the app); like the app's it
//! is disabled while no cable is attached.

use maud::{html, Markup};
use serde_json::{json, Value};

use crate::auth::SessionId;
use crate::boottime::BootInstant;
use crate::events::Topic;
use crate::handlers::AppState;
use crate::httpd::Ctx;
use crate::web::action::Valid;
use crate::web::fmt;
use crate::web::kit::{self, Act, Icon};
use crate::web::render::{self, Views};
use crate::web::screen::PageData;
use crate::web::store::Dismissed;

use super::dashboard::DASH;

pub const POWERBANK: &str = "/ui/usb/powerbank";
pub const DISMISS: &str = "/ui/usb/dismiss";
pub const MODE: &str = "/ui/tools/usb";
pub const MODE_POWERBANK: &str = "/ui/tools/usb/powerbank";

const FLAG: &str = "usb";

/// The cable, as last read (`None` before a good read); no `typec_cc` is
/// no cable, as in the app.
pub fn attached(v: &Views) -> Option<bool> {
    v.value(Topic::Usb).map(|u| u["typec_cc"].as_str().unwrap_or("no_cc") != "no_cc")
}

/// Whether `owner` dismissed the card for this cable; a detached reading
/// ends every session's dismissal (it was for that cable).
pub fn dismissed(store: &Dismissed, owner: SessionId, v: &Views) -> bool {
    if attached(v) == Some(false) {
        store.remove(FLAG, None, v.now);
        return false;
    }
    store.get(FLAG, owner, v.now).is_some()
}

/// `POST /ui/usb/dismiss`.
pub fn dismiss(ctx: &Ctx, owner: SessionId) {
    set(&ctx.dismissed, owner, ctx.app.auth.now());
}

fn set(store: &Dismissed, owner: SessionId, now: BootInstant) {
    store.remove(FLAG, Some(owner), now);
    store.insert(FLAG.into(), owner, (), now);
}

/// `POST /ui/usb/powerbank`: always `{"state":1}` (the agent hands the body
/// to ubus as is; turning it off is Tools › USB Mode's, as in the app).
pub fn powerbank_body(_: &Valid, _: &[Value]) -> Option<Value> {
    Some(json!({"state": 1}))
}

/// Tools › USB Mode's switch: the checked `state`, 0 or 1.
pub fn mode_body(v: &Valid, _: &[Value]) -> Option<Value> {
    Some(json!({"state": v.int("state")}))
}

/// The app's result lines (green).
pub fn switched(v: &Valid, _: &Value) -> (Option<bool>, String) {
    (Some(true), if v.get("state") == "1" { "启用快速充电" } else { "禁用快速充电" }.to_owned())
}

/// "Fast Charging"'s check before sending: a cable is attached now.
pub fn cable_read(app: &AppState, _: Option<&str>) -> Result<Value, String> {
    let (status, mut v) = crate::usb::usb_status(app);
    if status >= 400 {
        return Err(v["error"].as_str().unwrap_or("error").to_owned());
    }
    let d = v["data"].take();
    if d["typec_cc"].as_str().unwrap_or("no_cc") == "no_cc" {
        return Err("未连接 USB-C 电缆。".to_owned());
    }
    Ok(d)
}

/// `#card-usb`, rendered from `v` alone (with the dismissal it carries).
pub fn fragment(v: &Views) -> Markup {
    let on = attached(v) == Some(true) && !v.usb_dismissed;
    let pb = on && v.value(Topic::Charger).is_some_and(|c| fmt::int(&c["otg_powerbank_state"]) == Some(1));
    let class = if pb { Some("on pb") } else { on.then_some("on") };
    html! {
        div #card-usb class=[class] {
            @if on {
                (kit::icon(Icon::UsbC))
                h2 { "USB-C 已连接" }
                p.cap { "U60 Pro 上连接有一条 USB-C 电缆。" }
                @if pb { p.c-orange { (kit::icon(Icon::Bolt)) " Fast charging is active" } }
            }
        }
    }
}

/// The card: the fragment, then its forms (`csrf`: the session's field).
pub fn card(v: &Views, csrf: &Markup) -> Markup {
    html! {
        section.card.usbc {
            (fragment(v))
            form.fc method="post" action=(POWERBANK) {
                (csrf)
                input type="hidden" name="state" value="1";
                button.btn.btn-wide.btn-or type="submit" { (kit::icon(Icon::Bolt)) " Fast Charging" }
                p.cap { "使用U60 Pro电池给手机充电" }
            }
            form method="post" action=(DISMISS) {
                (csrf)
                button.btn.btn-quiet type="submit" { "驳回" }
            }
        }
    }
}

/// Tools › USB Mode: "USB Status" (Cable, USB-C CC, Mode, as the app),
/// then the switch with the app's footer.
pub fn mode_page(pd: &PageData) -> Markup {
    let v = pd.views;
    let usb = v.value(Topic::Usb);
    let cable = attached(v) == Some(true);
    let mode = usb.and_then(|u| u["mode"].as_str()).filter(|m| !m.is_empty()).unwrap_or(DASH);
    let pb = v.value(Topic::Charger).map(|c| fmt::int(&c["otg_powerbank_state"]) == Some(1));
    let post = |value| Act::Post { action: MODE_POWERBANK, name: "state", value };
    let (on, off) = if cable { (post("1"), post("0")) } else { (Act::Off, Act::Off) };
    let foot = html! {
        p.foot { "启用后，U60 Pro 电池将为您连接的设备充电。这会更快地耗尽路由器的电池。" }
    };
    html! {
        (render::error_line(v, &[Topic::Usb, Topic::Charger]))
        (kit::section(Some("USB状态"), None, html! {
            li.it {
                span { "电缆" }
                span.v { @if cable { span.c-green { "已连接" } } @else { "未连接" } }
            }
            (kit::kv_row(None, "USB-C CC", usb.and_then(|u| u["typec_cc"].as_str()).unwrap_or("no_cc")))
            (kit::kv_row(None, "模式", mode))
        }))
        (kit::section(None, Some(foot), kit::switch_row(&pd.csrf(), "快速充电（移动电源）", pb, on, off)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::hub::SlotView;
    use std::sync::Arc;
    use std::time::Duration;

    const NOW: BootInstant = BootInstant::from_duration(Duration::from_secs(10_000));

    fn views(usb: Option<Value>, charger: Option<Value>) -> Views {
        views_at(NOW, usb, charger)
    }

    fn views_at(now: BootInstant, usb: Option<Value>, charger: Option<Value>) -> Views {
        let slot = |v: Value| SlotView { version: 1, at: now, value_at: now, value: Some(Arc::new(v)), error: None };
        let mut slots = Vec::new();
        slots.extend(usb.map(|u| (Topic::Usb, slot(u))));
        slots.extend(charger.map(|c| (Topic::Charger, slot(c))));
        Views::new(now, slots)
    }

    fn cable() -> Option<Value> {
        Some(json!({"typec_cc": "cc1", "mode": "rndis", "connect": 1}))
    }

    #[test]
    fn hidden_without_a_cable() {
        for usb in [None, Some(json!({"typec_cc": "no_cc"})), Some(json!({"mode": "rndis"}))] {
            let s = fragment(&views(usb.clone(), None)).into_string();
            assert_eq!(s, "<div id=\"card-usb\"></div>", "{usb:?}");
        }
        assert_eq!(attached(&views(None, None)), None);
        assert_eq!(attached(&views(Some(json!({"typec_cc": "no_cc"})), None)), Some(false));
    }

    #[test]
    fn the_apps_sheet_as_a_card() {
        let off = Some(json!({"otg_powerbank_state": 0}));
        let s = fragment(&views(cable(), off)).into_string();
        assert!(s.starts_with("<div id=\"card-usb\" class=\"on\"><svg"), "{s}");
        for want in ["<h2>USB-C 已连接</h2>", "<p class=\"cap\">U60 Pro 上连接有一条 USB-C 电缆。</p>"] {
            assert!(s.contains(want), "{want} in {s}");
        }
        assert!(!s.contains("快速充电已启动"));
        // Powerbank on (a number or a string): the orange line, and the
        // class that hides "Fast Charging".
        for on in [json!({"otg_powerbank_state": 1}), json!({"otg_powerbank_state": "1"})] {
            let s = fragment(&views(cable(), Some(on))).into_string();
            assert!(s.starts_with("<div id=\"card-usb\" class=\"on pb\">"), "{s}");
            assert!(s.contains("<p class=\"c-orange\"><svg"), "{s}");
            assert!(s.contains("</svg> 快速充电已启动</p>"), "{s}");
        }
        // The forms follow the fragment, outside it; the button posts the one value.
        let csrf = html! { input type="hidden" name="csrf" value="t"; };
        let card = card(&views(cable(), None), &csrf).into_string();
        let text = fragment(&views(cable(), None)).into_string();
        assert!(card.starts_with(&format!("<section class=\"card usbc\">{text}<form class=\"fc\" method=\"post\" action=\"/ui/usb/powerbank\">")), "{card}");
        for want in [
            "<input type=\"hidden\" name=\"csrf\" value=\"t\"><input type=\"hidden\" name=\"state\" value=\"1\">",
            "</svg> 快速充电</button>", "<p class=\"cap\">使用U60 Pro电池给手机充电</p>",
            "<form method=\"post\" action=\"/ui/usb/dismiss\"><input type=\"hidden\" name=\"csrf\" value=\"t\"><button class=\"btn btn-quiet\" type=\"submit\">驳回</button>",
        ] {
            assert!(card.contains(want), "{want} in {card}");
        }
        // Dismissed: the card is off, its forms with it (CSS).
        let mut v = views(cable(), None);
        v.usb_dismissed = true;
        assert_eq!(fragment(&v).into_string(), "<div id=\"card-usb\"></div>");
    }

    #[test]
    fn the_body_is_built_here() {
        assert_eq!(powerbank_body(&Valid::of(&[("state", "1")]), &[]), Some(json!({"state": 1})));
        for (state, n, line) in [("0", 0, "Fast charging disabled"), ("1", 1, "Fast charging enabled")] {
            let v = Valid::of(&[("state", state)]);
            assert_eq!(mode_body(&v, &[]), Some(json!({"state": n})));
            assert_eq!(switched(&v, &Value::Null), (Some(true), line.to_owned()));
        }
    }

    fn mode_html(v: &Views) -> String {
        use crate::auth::{Kind, PeerKey};
        let app = AppState::new();
        app.auth.set_password("pw");
        let issued = app.auth.login("pw", Kind::Browser, PeerKey::UNKNOWN, &[]).unwrap();
        let p = app.auth.authenticate(Kind::Browser, issued.token.expose()).unwrap();
        mode_page(&PageData { app: &app, p: &p, views: v, reads: &[], remote: false, form: None, query: None, once: None, at: None, ussd: None, notice: None, token: Default::default(), prefs: Default::default(), secure: false }).into_string()
    }

    /// The app's USB Mode: the status rows, and the switch (the only way to
    /// turn fast charging off), disabled without a cable.
    #[test]
    fn usb_mode_follows_the_app() {
        let on = Some(json!({"otg_powerbank_state": 1}));
        let s = mode_html(&views(cable(), on.clone()));
        for want in [
            "<h2>USB状态</h2>", "<span>电缆</span><span class=\"v\"><span class=\"c-green\">已连接</span></span>",
            "<span>USB-C CC</span><span class=\"v\">cc1</span>", "<span>模式</span><span class=\"v\">rndis</span>",
            "<span>快速充电（移动电源）</span><form class=\"sf\" method=\"post\" action=\"/ui/tools/usb/powerbank\">",
            "<input type=\"hidden\" name=\"state\" value=\"0\"><button class=\"sw\" type=\"submit\" role=\"switch\" aria-checked=\"true\"",
            "<p class=\"foot\">启用后，U60 Pro 电池将为您连接的设备充电。这会更快地耗尽路由器的电池。</p>",
        ] {
            assert!(s.contains(want), "{want} in {s}");
        }
        let s = mode_html(&views(cable(), Some(json!({"otg_powerbank_state": 0}))));
        assert!(s.contains("name=\"state\" value=\"1\"><button class=\"sw\" type=\"submit\" role=\"switch\" aria-checked=\"false\""), "{s}");
        // No cable: "Disconnected", the switch disabled (as the app's), no form.
        let s = mode_html(&views(Some(json!({"typec_cc": "no_cc", "mode": ""})), on));
        assert!(s.contains("<span>电缆</span><span class=\"v\">未连接</span>") && s.contains("<span>模式</span><span class=\"v\">—</span>"), "{s}");
        assert!(s.contains("aria-disabled=\"true\"") && !s.contains("<form"), "{s}");
    }

    /// Dismissed until a detached reading, per session; the views carry it
    /// to the fragment.
    #[test]
    fn a_dismissal_lasts_until_the_cable_is_seen_detached() {
        let store = Dismissed::default();
        let (me, other) = (SessionId::for_test(1), SessionId::for_test(2));
        let attached = views(cable(), None);
        assert!(!dismissed(&store, me, &attached));
        set(&store, me, NOW);
        set(&store, me, NOW);
        assert!(dismissed(&store, me, &attached), "a reload while attached keeps it");
        assert!(!dismissed(&store, other, &attached), "only for the session that dismissed it");
        // An unknown or failed reading ends nothing.
        assert!(dismissed(&store, me, &views(None, None)));
        let mut v = views(cable(), None);
        v.usb_dismissed = dismissed(&store, me, &v);
        assert_eq!(fragment(&v).into_string(), "<div id=\"card-usb\"></div>");
        // Detached, seen by any session: every dismissal ends; the next
        // attach shows the card.
        set(&store, other, NOW);
        assert!(!dismissed(&store, other, &views(Some(json!({"typec_cc": "no_cc"})), None)));
        assert!(!dismissed(&store, me, &attached) && !dismissed(&store, other, &attached));
        // A day later it is forgotten anyway.
        set(&store, me, NOW);
        let later = BootInstant::from_duration(Duration::from_secs(10_000 + 24 * 3600));
        assert!(!dismissed(&store, me, &views_at(later, cable(), None)));
    }
}
