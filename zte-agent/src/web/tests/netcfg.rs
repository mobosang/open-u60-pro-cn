//! Router › WiFi, Guest WiFi, APN, LAN / DHCP and DNS through the whole
//! router: the forms, their drafts and questions, the confirmed posts and
//! where they lead. The Wi-Fi status reads come from `pages::fake`, the
//! other page-load reads from `pages::device`'s `gets`, the changes go to
//! `pages::device`. Every value is a made-up fixture.

use serde_json::json;

use super::*;
use crate::web::pages::{device, fake};

const WIFI: &str = "/ui/router/wifi";
const WIFI_APPLY: &str = "/ui/router/wifi/apply";
const GUEST: &str = "/ui/router/guest-wifi";
const GUEST_APPLY: &str = "/ui/router/guest-wifi/apply";
const APN: &str = "/ui/router/apn";
const LAN: &str = "/ui/router/lan";
const LAN_APPLY: &str = "/ui/router/lan/apply";
const DNS: &str = "/ui/router/dns";

fn calls(c: &Ctx) -> Vec<(String, Value)> {
    device::calls(&c.app)
}

fn tailnet(port: u16) -> Peer {
    Peer(SocketAddr::from(([127, 0, 0, 1], port)))
}

/// `location` of a 303.
fn to(r: &Reply) -> String {
    assert_eq!(r.status, 303, "{}", r.text());
    r.header("location").unwrap().to_owned()
}

/// A form's fields as posted: `csrf`, `once` and these.
fn body(csrf: &str, once: &str, fields: &[(&str, &str)]) -> String {
    let mut out = format!("csrf={csrf}&once={once}");
    for (k, v) in fields {
        out.push_str(&format!("&{k}={}", enc(v)));
    }
    out
}

/// The router's Wi-Fi state (keys included: the pages must never show them).
fn wifi_status() -> Value {
    json!({
        "wifi_onoff": "1", "wifi6_switch": "0", "radio2_disabled": "0", "radio5_disabled": "0",
        "ssid_2g": "Fake-Net", "ssid_5g": "Fake-Net-5G", "key_2g": "FAKEKEY-2g-0001", "key_5g": "FAKEKEY-5g-0001",
        "channel_2g": "auto", "channel_5g": "36", "htmode_2g": "EHT20", "htmode_5g": "EHT80", "txpower_2g": "100", "txpower_5g": "100",
        "encryption_2g": "psk2+ccmp", "encryption_5g": "sae-mixed", "hidden_2g": "0", "hidden_5g": "0",
        "disabled_2g": "0", "disabled_5g": "1", "ssid": "Fake-Guest", "key": "FAKEKEY-guest-01", "encryption": "psk2+ccmp",
        "hidden": "0", "isolate": "1", "guest_active_time": "60", "remaining_seconds": "3909",
    })
}

/// The WiFi form as the page draws it, with these fields changed.
fn wifi_fields(over: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    let mut f = vec![
        ("wifi", "1"), ("r2", "1"), ("ssid_2g", "Fake-Net"), ("key_2g", ""), ("channel_2g", "auto"), ("htmode_2g", "EHT20"),
        ("txpower_2g", "100"), ("encryption_2g", "psk2+ccmp"), ("r5", "1"), ("ssid_5g", "Fake-Net-5G"), ("key_5g", ""),
        ("channel_5g", "36"), ("htmode_5g", "EHT80"), ("txpower_5g", "100"), ("encryption_5g", "sae-mixed"),
    ];
    for (k, v) in over {
        match f.iter_mut().find(|(n, _)| n == k) {
            Some(x) => x.1 = v,
            None => f.push((k, v)),
        }
    }
    // An unchecked switch is not posted at all.
    f.retain(|(k, v)| !v.is_empty() || !["wifi", "wifi6_switch", "r2", "r5", "hidden_2g", "hidden_5g"].contains(k));
    f
}

/// WiFi: the app's form without any password; a refused post keeps a
/// valid typed password in the draft (never in the page); a valid one
/// becomes a question that carries only the token and the revision;
/// Cancel shows the form as typed; the confirmed post sends only what
/// changed, once, then "Reconnect to …".
#[tokio::test]
async fn wifi_form_question_and_reconnect_page() {
    let (c, cookie, id) = signed_in(200).await;
    let _fake = fake::set(&c.app, wifi_status());
    let _dev = device::set(&c.app, device::Device::default());
    let r = get_with(&c, WIFI, peer(200, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    for want in [
        "<h1>WiFi 设置</h1>", "href=\"/ui/router\">路由器</a>", "<form class=\"rf wsf\" method=\"post\" action=\"/ui/router/wifi/apply\" autocomplete=\"off\">",
        "<input class=\"sr\" id=\"wifi\" type=\"checkbox\" role=\"switch\" name=\"wifi\" value=\"1\" checked>", "<h2>2.4 GHz</h2>", "<h2>5 GHz</h2>",
        "<input class=\"sr ro\" id=\"r2\" type=\"checkbox\" role=\"switch\" name=\"r2\" value=\"1\" checked>",
        "name=\"ssid_2g\" value=\"Fake-Net\"", "<input id=\"key_2g\" type=\"password\" name=\"key_2g\" placeholder=\"留空保留\"",
        "<option value=\"sae-mixed\" selected>WPA2/WPA3 混合</option>", "<option value=\"EHT80\" selected>80 MHz</option>",
        "80 MHz: channels 36\u{2013}64, 100\u{2013}128, or 149\u{2013}161", "<h2>高级</h2>", "WiFi 7 (802.11be)",
        "应用更改将在重新启动设置时短暂断开 WiFi 的连接",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("FAKEKEY"), "{html}");
    let (csrf, once) = (r.input("csrf"), r.input("once"));
    // Origin and CSRF first: nothing drafted, nothing sent.
    let fields = body(&csrf, &once, &wifi_fields(&[]));
    let bad = form(WIFI_APPLY, peer(200, 3), &cookie, &fields).header("origin", "http://evil.example").body(Body::from(fields.clone())).unwrap();
    assert_eq!(send(&c, bad).await.status, 403);
    assert_eq!(post(&c, WIFI_APPLY, peer(200, 3), &cookie, &fields.replacen(&csrf, "0", 1)).await.status, 403);
    // A value no form sends.
    let r = post(&c, WIFI_APPLY, peer(200, 3), &cookie, &body(&csrf, &once, &wifi_fields(&[("channel_2g", "13")]))).await;
    assert_eq!(r.status, 400);
    // Refused: a too-long name and a 165/80 MHz pair; the typed password is kept, not shown.
    const LONG: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let typed = wifi_fields(&[("ssid_2g", "Fake-Net-New"), ("key_2g", "new-fake-pass-1"), ("channel_5g", "165"), ("ssid_5g", LONG)]);
    let r = post(&c, WIFI_APPLY, peer(200, 3), &cookie, &body(&csrf, &once, &typed)).await;
    r.assert_secure_html(400);
    let html = r.text();
    let same_token = format!("name=\"once\" value=\"{once}\"");
    for want in [
        "Use at most 32 bytes", "通道 165 最多支持 20 MHz。", "name=\"ssid_2g\" value=\"Fake-Net-New\"",
        "placeholder=\"留空以保留您输入的密码\"", same_token.as_str(),
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("new-fake-pass") && !html.contains("FAKEKEY"), "{html}");
    assert!(calls(&c).is_empty());
    // Fixed, the password left empty (kept): the question.
    let fixed = wifi_fields(&[("ssid_2g", "Fake-Net-New"), ("hidden_2g", "1")]);
    let r = post(&c, WIFI_APPLY, peer(200, 3), &cookie, &body(&csrf, &once, &fixed)).await;
    assert_eq!(to(&r), format!("{WIFI_APPLY}?t={once}"));
    let q = get_with(&c, &to(&r), peer(200, 3), &cookie).await;
    q.assert_secure_html(200);
    let html = q.text();
    let cancel = format!("href=\"{WIFI}?t={once}\">Cancel</a>");
    for want in [
        "<h1>确认</h1>", "<h2 class=\"q\">应用WiFi设置？</h2>",
        "On 2.4 GHz, 设备 must join “Fake-Net-New” again ，使用新密码.",
        "“Fake-Net-New” is hidden: 设备 must type its name to join.", "WiFi restarts to apply the changes",
        "<button class=\"btn btn-danger\" type=\"submit\">应用</button>", cancel.as_str(),
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("new-fake-pass") && !html.contains("ssid_2g") && !html.contains("FAKEKEY"), "{html}");
    let rev = q.input("d");
    // Cancel: the form as typed, its token kept (the password still kept).
    let back = get_with(&c, &format!("{WIFI}?t={once}"), peer(200, 3), &cookie).await.text();
    for want in ["name=\"ssid_2g\" value=\"Fake-Net-New\"", "留空以保留您输入的密码", same_token.as_str()] {
        assert!(back.contains(want), "{want} in {back}");
    }
    assert!(!back.contains("new-fake-pass"));
    // Confirmed: only the changes (the kept password included), once.
    let confirm = format!("csrf={csrf}&once={once}&d={rev}&confirm=1");
    let r = post(&c, WIFI_APPLY, peer(200, 3), &cookie, &confirm).await;
    let done = to(&r);
    assert_eq!(done, format!("/ui/router/wifi/applied?t={once}"));
    assert_eq!(calls(&c), vec![("/api/wifi/settings".to_owned(), json!({"ssid_2g": "Fake-Net-New", "key_2g": "new-fake-pass-1", "hidden_2g": "1"}))]);
    let page = get_with(&c, &done, peer(200, 3), &cookie).await;
    page.assert_secure_html(200);
    let html = page.text();
    for want in [
        "<title>重新连接WiFi · U60 Pro</title>", "WiFi 设置已应用 — WiFi 将短暂重启",
        "Join “Fake-Net-New” (2.4 GHz)，使用新密码 ，若未自动重连，请手动连接。",
        "Join “Fake-Net-5G” (5 GHz)，若未自动重连，请手动连接。", "href=\"/ui/router/wifi\">返回WiFi设置</a>",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    // The draft lost its password once sent.
    match c.drafts.get(&once, id, c.app.auth.now()) {
        Some(crate::web::store::Draft::Router(d)) => assert!(!d.vals.iter().any(|(_, v)| v.contains("new-fake-pass"))),
        _ => panic!("draft gone"),
    }
    // A replay sends nothing and lands on the same page; the question is gone.
    assert_eq!(to(&post(&c, WIFI_APPLY, peer(200, 3), &cookie, &confirm).await), done);
    assert_eq!(calls(&c).len(), 1);
    assert_eq!(to(&get_with(&c, &format!("{WIFI_APPLY}?t={once}"), peer(200, 3), &cookie).await), WIFI);
    // Another session never sees the page's draft.
    let other = format!("zte_ui_session={}", login(&c, 200, 9).await);
    assert!(!get_with(&c, &done, peer(200, 9), &other).await.text().contains("Fake-Net-New"));
}

/// WiFi: one name per radio on the reconnect page; TX power alone is
/// applied without a restart; a device failure shows the form as typed;
/// turning WiFi off says what it cuts; nothing changed asks nothing.
#[tokio::test]
async fn wifi_off_tx_power_failures_and_nothing_to_apply() {
    let (c, cookie, _) = signed_in(201).await;
    let _fake = fake::set(&c.app, wifi_status());
    let page = get_with(&c, WIFI, peer(201, 2), &cookie).await;
    let csrf = page.input("csrf");
    let ask = |fields: Vec<(&'static str, &'static str)>| {
        let (c, cookie, csrf) = (c.clone(), cookie.clone(), csrf.clone());
        async move {
            let once = get_with(&c, WIFI, peer(201, 2), &cookie).await.input("once");
            let r = post(&c, WIFI_APPLY, peer(201, 2), &cookie, &body(&csrf, &once, &fields)).await;
            let q = get_with(&c, &to(&r), peer(201, 2), &cookie).await;
            (once, q)
        }
    };
    let (_, q) = ask(wifi_fields(&[])).await;
    assert!(q.text().contains("<h2 class=\"q\">没有什么可申请的</h2>") && !q.text().contains("name=\"confirm\""), "{}", q.text());
    let (_, q) = ask(wifi_fields(&[("wifi", "")])).await;
    assert!(q.text().contains("<h2 class=\"q\">关闭WiFi?</h2>") && q.text().contains(">关闭WiFi</button>"), "{}", q.text());
    // TX power alone: no restart, no reconnect page.
    let _dev = device::set(&c.app, device::Device { replies: vec![("/api/wifi/settings", (200, json!({"ok": true, "data": {"status": "ok", "hot": true}})))], ..Default::default() });
    let (once, q) = ask(wifi_fields(&[("txpower_5g", "20")])).await;
    assert!(q.text().contains("TX功率立即改变； WiFi不重启。") && q.text().contains("<button class=\"btn\" type=\"submit\">应用</button>"), "{}", q.text());
    let r = post(&c, WIFI_APPLY, peer(201, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    let line = to(&r);
    assert_eq!(line, format!("{WIFI}?r={once}"));
    assert!(get_with(&c, &line, peer(201, 2), &cookie).await.text().contains("<p class=\"msg ok\" role=\"status\">应用 WiFi 设置</p>"));
    // A device failure: the form again as typed, with the line.
    let _dev = device::set(&c.app, device::Device { replies: vec![("/api/wifi/settings", (500, json!({"ok": false, "error": "uci busy (fake)"})))], ..Default::default() });
    let (once, q) = ask(wifi_fields(&[("ssid_5g", "Fake-5G-B")])).await;
    let r = post(&c, WIFI_APPLY, peer(201, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    let back = to(&r);
    assert_eq!(back, format!("{WIFI}?t={once}&r={once}"));
    let html = get_with(&c, &back, peer(201, 2), &cookie).await.text();
    assert!(html.contains("<p class=\"msg err\" role=\"alert\">失败：uci busy (fake)</p>") && html.contains("value=\"Fake-5G-B\""), "{html}");
    // A new token for the form (the failed one is spent).
    assert!(!html.contains(&format!("name=\"once\" value=\"{once}\"")));
}

/// A failed Apply keeps a newly typed password (never in a page): the form
/// shown again says so and posts with a new token that holds it, so a
/// retry with the field left empty sends it; the spent draft keeps no copy.
#[tokio::test]
async fn a_failed_apply_keeps_the_typed_password_for_the_retry() {
    let (c, cookie, id) = signed_in(208).await;
    let _fake = fake::set(&c.app, wifi_status());
    let fail_once = device::Device { replies: vec![("/api/wifi/settings#1", (500, json!({"ok": false, "error": "uci busy (fake)"})))], ..Default::default() };
    let _dev = device::set(&c.app, fail_once);
    let page = get_with(&c, WIFI, peer(208, 2), &cookie).await;
    let (csrf, once) = (page.input("csrf"), page.input("once"));
    let typed = wifi_fields(&[("ssid_2g", "FakeNet-Fail"), ("key_2g", "new-fake-pass-2")]);
    let r = post(&c, WIFI_APPLY, peer(208, 2), &cookie, &body(&csrf, &once, &typed)).await;
    let q = get_with(&c, &to(&r), peer(208, 2), &cookie).await;
    let r = post(&c, WIFI_APPLY, peer(208, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    let back = to(&r);
    assert_eq!(back, format!("{WIFI}?t={once}&r={once}"));
    let page = get_with(&c, &back, peer(208, 2), &cookie).await;
    page.assert_secure_html(200);
    let html = page.text();
    for want in [
        "<p class=\"msg err\" role=\"alert\">失败：uci busy (fake)</p>", "name=\"ssid_2g\" value=\"FakeNet-Fail\"",
        "<input id=\"key_2g\" type=\"password\" name=\"key_2g\" placeholder=\"留空以保留您输入的密码\"",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("new-fake-pass") && !html.contains("FAKEKEY"), "{html}");
    // A new token (the failed one is spent); a reload keeps it.
    let next = page.input("once");
    assert!(!next.is_empty() && next != once);
    assert_eq!(get_with(&c, &back, peer(208, 2), &cookie).await.input("once"), next);
    // One copy of the password: with the new token, not in the spent draft.
    match c.drafts.get(&once, id, c.app.auth.now()) {
        Some(crate::web::store::Draft::Router(d)) => assert!(!d.vals.iter().any(|(_, v)| v.contains("new-fake-pass"))),
        _ => panic!("draft gone"),
    }
    // The retry, the password field left empty: the question says the new
    // password, and the device gets it.
    let r = post(&c, WIFI_APPLY, peer(208, 2), &cookie, &body(&csrf, &next, &wifi_fields(&[("ssid_2g", "FakeNet-Fail")]))).await;
    assert_eq!(to(&r), format!("{WIFI_APPLY}?t={next}"));
    let q = get_with(&c, &to(&r), peer(208, 2), &cookie).await;
    assert!(q.text().contains("设备 must join “FakeNet-Fail” again ，使用新密码."), "{}", q.text());
    assert!(!q.text().contains("new-fake-pass"));
    let r = post(&c, WIFI_APPLY, peer(208, 2), &cookie, &format!("csrf={csrf}&once={next}&d={}&confirm=1", q.input("d"))).await;
    assert_eq!(to(&r), format!("/ui/router/wifi/applied?t={next}"));
    let sent = json!({"ssid_2g": "FakeNet-Fail", "key_2g": "new-fake-pass-2"});
    assert_eq!(calls(&c), vec![("/api/wifi/settings".to_owned(), sent.clone()), ("/api/wifi/settings".to_owned(), sent)]);
    // Sent: gone from the draft; the old page again has nothing to keep.
    for t in [&once, &next] {
        match c.drafts.get(t, id, c.app.auth.now()) {
            Some(crate::web::store::Draft::Router(d)) => assert!(!d.vals.iter().any(|(_, v)| v.contains("new-fake-pass")), "{t}"),
            _ => panic!("draft gone"),
        }
    }
    let html = get_with(&c, &back, peer(208, 2), &cookie).await.text();
    assert!(html.contains("placeholder=\"留空保留\"") && !html.contains("the password you entered"), "{html}");
}

/// A carried password whose token was pushed out of the store (64 newer
/// ones) is dropped at the next look, not kept unreachable for the hour.
#[tokio::test]
async fn a_carried_password_goes_with_its_token() {
    let (c, cookie, id) = signed_in(209).await;
    let _fake = fake::set(&c.app, wifi_status());
    let fail_once = device::Device { replies: vec![("/api/wifi/settings#1", (500, json!({"ok": false, "error": "uci busy (fake)"})))], ..Default::default() };
    let _dev = device::set(&c.app, fail_once);
    let page = get_with(&c, WIFI, peer(209, 2), &cookie).await;
    let (csrf, once) = (page.input("csrf"), page.input("once"));
    let typed = wifi_fields(&[("ssid_2g", "FakeNet-Gone"), ("key_2g", "new-fake-pass-3")]);
    let r = post(&c, WIFI_APPLY, peer(209, 2), &cookie, &body(&csrf, &once, &typed)).await;
    let q = get_with(&c, &to(&r), peer(209, 2), &cookie).await;
    let r = post(&c, WIFI_APPLY, peer(209, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    let back = to(&r);
    let next = get_with(&c, &back, peer(209, 2), &cookie).await.input("once");
    let held = |t: &str| match c.drafts.get(t, id, c.app.auth.now()) {
        Some(crate::web::store::Draft::Router(d)) => d.vals.iter().any(|(_, v)| v.contains("new-fake-pass")),
        _ => false,
    };
    assert!(held(&next));
    // The token store's eviction, as 64 newer tokens would.
    c.once.remove(&next, Some(id), c.app.auth.now());
    let html = get_with(&c, &back, peer(209, 2), &cookie).await.text();
    assert!(html.contains("placeholder=\"留空保留\"") && !html.contains("the password you entered"), "{html}");
    assert!(!held(&next) && c.drafts.get(&next, id, c.app.auth.now()).is_none());
}

/// Guest WiFi: the app's sections and timer, no password; Apply asks
/// first and sends only what changed.
#[tokio::test]
async fn guest_wifi_form_and_apply() {
    let (c, cookie, _) = signed_in(202).await;
    let _fake = fake::set(&c.app, wifi_status());
    let _dev = device::set(&c.app, device::Device::default());
    let r = get_with(&c, GUEST, peer(202, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    for want in [
        "<h1>访客 WiFi</h1>", "<h2>无线电频段</h2>", "<input class=\"sr gb\" id=\"g2\" type=\"checkbox\" role=\"switch\" name=\"g2\" value=\"1\" checked>",
        "<input class=\"sr gb\" id=\"g5\" type=\"checkbox\" role=\"switch\" name=\"g5\" value=\"1\">", "<h2>网络</h2>", "name=\"gssid\" value=\"Fake-Guest\"",
        "placeholder=\"留空保留\"", "客户端隔离", "<h2>计时器</h2>", "<option value=\"60\" selected>1 hour</option>",
        "访客WiFi在1 hour之后将自动关闭", "1h 05m 09s remaining", "href=\"/ui/router/guest-wifi\">刷新</a>",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("FAKEKEY"));
    let (csrf, once) = (r.input("csrf"), r.input("once"));
    let fields = [("g2", "1"), ("gssid", "Fake-Guest-2"), ("gkey", "guest-fake-pass"), ("genc", "psk2+ccmp"), ("giso", "1"), ("gtime", "120")];
    let r = post(&c, GUEST_APPLY, peer(202, 2), &cookie, &body(&csrf, &once, &fields)).await;
    let q = get_with(&c, &to(&r), peer(202, 2), &cookie).await;
    let html = q.text();
    for want in [
        "<h2 class=\"q\">应用访客WiFi设置？</h2>", "客人必须再次加入“Fake-Guest-2” with the new password。",
        "访客WiFi在2 hours之后自行关闭。", "guests included",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("guest-fake-pass"));
    let r = post(&c, GUEST_APPLY, peer(202, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    let line = to(&r);
    assert_eq!(
        calls(&c),
        vec![("/api/wifi/guest".to_owned(), json!({"guest_ssid": "Fake-Guest-2", "guest_key": "guest-fake-pass", "guest_active_time": "120"}))]
    );
    // Every Wi-Fi network restarts: the reconnect page (from the draft, no
    // device read, never the password); a replay lands on it again.
    assert_eq!(line, format!("/ui/router/guest-wifi/applied?t={once}"));
    let page = get_with(&c, &line, peer(202, 2), &cookie).await;
    page.assert_secure_html(200);
    let html = page.text();
    for want in [
        "<title>重新连接WiFi · U60 Pro</title>", "<p class=\"msg ok\" role=\"status\">已应用访客 WiFi 设置</p>",
        "Join “Fake-Net”，若未自动重连，请手动连接。", "Join “Fake-Net-5G”，若未自动重连，请手动连接。",
        "Guests join “Fake-Guest-2”，使用新密码.", "href=\"/ui/router/guest-wifi\">返回嘉宾WiFi</a>",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("guest-fake-pass") && !html.contains("FAKEKEY"), "{html}");
    let confirm = format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"));
    assert_eq!(to(&post(&c, GUEST_APPLY, peer(202, 2), &cookie, &confirm).await), line);
    assert_eq!(calls(&c).len(), 1);
    // A band of a running guest network: named; an open network: said so.
    let once = get_with(&c, GUEST, peer(202, 2), &cookie).await.input("once");
    let fields = [("g2", "1"), ("g5", "1"), ("gssid", "Fake-Guest"), ("genc", "none"), ("giso", "1"), ("gtime", "60")];
    let r = post(&c, GUEST_APPLY, peer(202, 2), &cookie, &body(&csrf, &once, &fields)).await;
    let html = get_with(&c, &to(&r), peer(202, 2), &cookie).await.text();
    for want in [
        "<h2 class=\"q\">应用访客WiFi设置？</h2>", "5 GHz 访客网络打开。",
        "\u{201c}Fake-Guest\u{201d} is an open network: anyone nearby can join it, without a password, and reach this router.",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    // A secured network needs a password the router can keep.
    let once = get_with(&c, GUEST, peer(202, 2), &cookie).await.input("once");
    let r = post(&c, GUEST_APPLY, peer(202, 2), &cookie, &body(&csrf, &once, &[("g5", "1"), ("gssid", "Fake&Guest"), ("gkey", "short"), ("genc", "sae"), ("gtime", "0")])).await;
    r.assert_secure_html(400);
    assert!(r.text().contains("The router can&#39;t store these 个字符") || r.text().contains("The router can't store these 个字符"), "{}", r.text());
    assert!(r.text().contains("Use 8 to 63 个字符"));
}

/// The router's APN lists (made-up profiles; a password that must never
/// reach a page).
fn apn_device(mode: &str) -> device::Device {
    let list = json!({"apnListArray": [
        {"profileId": "manu1", "profilename": "Fake Home", "wanapn": "home.fake.example", "pdpType": "3", "pppAuthMode": "1", "username": "fakeuser", "password": "FAKE-APN-PW", "isEnable": "1"},
        {"profileId": "manu2", "profilename": "Fake Travel", "wanapn": "travel.fake.example", "pdpType": 1, "pppAuthMode": 0, "username": "", "password": "", "isEnable": "0"},
        {"profileId": "manu3", "profilename": "", "wanapn": ""},
    ]});
    let auto = json!({"apnListArray": [{"profileId": "auto1", "profilename": "Fake Auto", "wanapn": "auto.fake.example", "pdpType": "3", "pppAuthMode": "0", "isEnable": "0"}]});
    device::Device {
        gets: vec![("/api/router/apn/mode", Ok(json!({"apn_mode": mode}))), ("/api/router/apn/profiles", Ok(list)), ("/api/router/apn/auto-profiles", Ok(auto))],
        ..Default::default()
    }
}

/// APN: the app's list and sheets, no password anywhere; activate,
/// delete and the mode ask first (with the remote line) and send once;
/// Edit merges the stored password when the field is left empty.
#[tokio::test]
async fn apn_list_sheets_and_changes() {
    let (c, cookie, _) = signed_in(203).await;
    let _dev = device::set(&c.app, apn_device("1"));
    let r = get_with(&c, APN, peer(203, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    for want in [
        "<h1>APN 设置</h1>", "<h2>当前APN</h2>", "<b>Fake Home</b>", "<h2>APN模式</h2>",
        "<a class=\"sw\" href=\"/ui/router/apn/mode?manual=0\" role=\"switch\" aria-checked=\"true\" aria-label=\"说明书APN\">",
        "<h2>手动配置文件</h2>", "<a class=\"it bt\" href=\"/ui/router/apn/new\">", "<span>添加APN</span>", "href=\"/ui/router/apn/edit?id=manu1\"",
        "<small class=\"c-green\">已启用</small>", "IPv4v6 / PAP", "IPv4 / None", "href=\"/ui/router/apn/activate?id=manu2\"",
        "href=\"/ui/router/apn/delete?id=manu2\"",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("FAKE-APN-PW") && !html.contains("activate?id=manu1") && !html.contains("manu3"), "{html}");
    // Edit APN: filled, but never the password.
    let r = get_with(&c, "/ui/router/apn/edit?id=manu1", peer(203, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    for want in [
        "<h1>编辑APN</h1>", "<a class=\"nl\" href=\"/ui/router/apn\">取消</a>", "name=\"name\" value=\"Fake Home\"", "name=\"apn\" value=\"home.fake.example\"",
        "<option value=\"1\" selected>PAP</option>", "name=\"user\" value=\"fakeuser\"", "<input id=\"pass\" type=\"password\" name=\"pass\" placeholder=\"留空保留\"",
        "name=\"def\" value=\"1\" checked", ">保存</button>",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    assert!(!html.contains("FAKE-APN-PW"));
    assert_eq!(get_with(&c, "/ui/router/apn/edit?id=manu9", peer(203, 2), &cookie).await.status, 404);
    // Without an id: 404 and only "APN not found", no form that would add one.
    let r = get_with(&c, "/ui/router/apn/edit", peer(203, 2), &cookie).await;
    r.assert_secure_html(404);
    let html = r.text();
    assert!(html.contains("APN 未找到") && html.contains("href=\"/ui/router/apn\">返回APN设置</a>"), "{html}");
    assert!(!html.contains("<form class=\"rf apf\"") && !html.contains("action=\"/ui/router/apn/save\""), "{html}");
    let auto = get_with(&c, "/ui/router/apn/auto?id=auto1", peer(203, 2), &cookie).await.text();
    assert!(auto.contains("<h1>汽车APN</h1>") && auto.contains("auto.fake.example") && auto.contains("<span class=\"v\">未启用</span>"), "{auto}");
    // Activate: asked (remote line), sent once.
    let q = get_with(&c, "/ui/router/apn/activate?id=manu2", tailnet(40203), &cookie).await;
    let html = q.text();
    assert!(html.contains("<h2 class=\"q\">使用“Fake Travel”？</h2>") && html.contains("You are connected remotely") && html.contains(">激活</button>"), "{html}");
    let csrf = q.input("csrf");
    let go = format!("csrf={csrf}&id=manu2&once={}&confirm=1", q.input("once"));
    let r = post(&c, "/ui/router/apn/activate", peer(203, 2), &cookie, &go).await;
    let line = to(&r);
    assert!(get_with(&c, &line, peer(203, 2), &cookie).await.text().contains("<p class=\"msg ok\" role=\"status\">APN 已激活</p>"));
    assert_eq!(to(&post(&c, "/ui/router/apn/activate", peer(203, 2), &cookie, &go).await), line);
    assert_eq!(calls(&c), vec![("/api/router/apn/profiles/activate".to_owned(), json!({"profileId": "manu2"}))]);
    // Without confirm=1: the question; a crafted id: 400.
    assert_eq!(to(&post(&c, "/ui/router/apn/delete", peer(203, 2), &cookie, &format!("csrf={csrf}&id=manu2")).await), "/ui/router/apn/delete?id=manu2");
    assert_eq!(post(&c, "/ui/router/apn/delete", peer(203, 2), &cookie, &format!("csrf={csrf}&id=a%20b&confirm=1")).await.status, 400);
    // The active profile cannot be deleted: no button, and a post sends nothing.
    let q = get_with(&c, "/ui/router/apn/delete?id=manu1", peer(203, 2), &cookie).await.text();
    assert!(q.contains("<h2 class=\"q\">无法删除活动的APN</h2>") && !q.contains("name=\"confirm\""), "{q}");
    let q = get_with(&c, "/ui/router/apn/delete?id=manu2", peer(203, 2), &cookie).await;
    let token = q.input("once");
    let r = post(&c, "/ui/router/apn/delete", peer(203, 2), &cookie, &format!("csrf={csrf}&id=manu1&once={token}&confirm=1")).await;
    assert!(get_with(&c, &to(&r), peer(203, 2), &cookie).await.text().contains("无法删除活动的APN"));
    assert_eq!(calls(&c).len(), 1);
    // The mode: asked, then the app's body.
    let q = get_with(&c, "/ui/router/apn/mode?manual=0", peer(203, 2), &cookie).await;
    assert!(q.text().contains("<h2 class=\"q\">切换到automatic APN？</h2>"));
    post(&c, "/ui/router/apn/mode", peer(203, 2), &cookie, &format!("csrf={csrf}&manual=0&once={}&confirm=1", q.input("once"))).await;
    assert_eq!(calls(&c).last().unwrap(), &("/api/router/apn/mode".to_owned(), json!({"apn_mode": 0})));
    // Edit with the password left empty: the stored one, merged on the router.
    let once = get_with(&c, "/ui/router/apn/edit?id=manu1", peer(203, 2), &cookie).await.input("once");
    let fields = [("id", "manu1"), ("name", "Fake Home"), ("apn", "home2.fake.example"), ("pdp", "3"), ("auth", "1"), ("user", "fakeuser"), ("def", "1")];
    let r = post(&c, "/ui/router/apn/save", peer(203, 2), &cookie, &body(&csrf, &once, &fields)).await;
    let q = get_with(&c, &to(&r), peer(203, 2), &cookie).await;
    assert!(q.text().contains("<h2 class=\"q\">保存“Fake Home”？</h2>") && q.text().contains("href=\"/ui/router/apn/edit?id=manu1&amp;t="), "{}", q.text());
    let r = post(&c, "/ui/router/apn/save", peer(203, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    assert!(get_with(&c, &to(&r), peer(203, 2), &cookie).await.text().contains("APN更新"));
    assert_eq!(
        calls(&c).last().unwrap(),
        &("/api/router/apn/profiles".to_owned(), json!({"profileId": "manu1", "profilename": "Fake Home", "wanapn": "home2.fake.example", "pdpType": 3,
            "pppAuthMode": 1, "username": "fakeuser", "password": "FAKE-APN-PW"}))
    );
    // New APN: the app's checks.
    let once = get_with(&c, "/ui/router/apn/new", peer(203, 2), &cookie).await.input("once");
    let r = post(&c, "/ui/router/apn/save", peer(203, 2), &cookie, &body(&csrf, &once, &[("name", "Fake Travel"), ("apn", "bad apn"), ("pdp", "3"), ("auth", "0")])).await;
    r.assert_secure_html(400);
    assert!(r.text().contains("同名的 APN 已存在") && r.text().contains("Use up to 100 letters"), "{}", r.text());
    let r = post(&c, "/ui/router/apn/save", peer(203, 2), &cookie, &body(&csrf, &once, &[("name", ""), ("apn", ""), ("pdp", "3"), ("auth", "0")])).await;
    assert_eq!(r.text().matches("Name and APN are required").count(), 2);
}

/// LAN / DHCP: the start shown as an address; the checks; the question
/// names the new address (with this browser's port); the body carries
/// full addresses; the page after it links the new address.
#[tokio::test]
async fn lan_checks_asks_and_links_the_new_address() {
    let (c, cookie, _) = signed_in(204).await;
    let cur = json!({"lan_ipaddr": "192.168.0.1", "lan_netmask": "255.255.255.0", "dhcp_enable": "1", "dhcp_start": "100", "dhcp_end": "192.168.0.149", "dhcp_lease_time": "12h"});
    let _dev = device::set(&c.app, device::Device { gets: vec![("/api/router/lan", Ok(cur))], ..Default::default() });
    let r = get_with(&c, LAN, peer(204, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    for want in [
        "<h1>LAN / DHCP</h1>", "<h2>LAN</h2>", "name=\"ip\" value=\"192.168.0.1\"", "<h2>DHCP服务器</h2>", "name=\"start\" value=\"192.168.0.100\"",
        "name=\"end\" value=\"192.168.0.149\"", "name=\"lease\" value=\"12h\"", "租用时间（秒）",
        // The range and lease time are hidden while DHCP is off (CSS).
        "<form class=\"rf lnf\"", "<li class=\"it fld dh\"><label for=\"start\">",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    let (csrf, once) = (r.input("csrf"), r.input("once"));
    let fields = [("ip", "192.168.8.1"), ("mask", "255.255.255.0"), ("dhcp", "1"), ("start", "192.168.0.100"), ("end", "192.168.0.149"), ("lease", "12h")];
    let r = post(&c, LAN_APPLY, peer(204, 2), &cookie, &body(&csrf, &once, &fields)).await;
    r.assert_secure_html(400);
    assert_eq!(r.text().matches("Use an address from 192.168.8.1 to 192.168.8.254.").count(), 2, "{}", r.text());
    let fields = [("ip", "192.168.8.1"), ("mask", "255.255.255.0"), ("dhcp", "1"), ("start", "192.168.8.100"), ("end", "192.168.8.149"), ("lease", "12h")];
    let r = post(&c, LAN_APPLY, peer(204, 2), &cookie, &body(&csrf, &once, &fields)).await;
    let q = get_with(&c, &to(&r), peer(204, 2), &cookie).await;
    let html = q.text();
    assert!(html.contains("The router\u{2019}s address changes from 192.168.0.1 to 192.168.8.1.") && html.contains("open http://192.168.8.1:9090/ui/ afterwards"), "{html}");
    assert!(html.contains("disconnects for a moment, this browser too.") && !html.contains("connected remotely"), "{html}");
    // Over Tailscale: the address on the router's network, the remote line.
    let remote = get_with(&c, &to(&r), tailnet(40204), &cookie).await.text();
    assert!(remote.contains("路由器’在其网络上的地址从192.168.0.1更改为192。168.8.1.") && !remote.contains("this browser too"), "{remote}");
    assert!(remote.contains("You are connected remotely:") && !remote.contains("open http://"), "{remote}");
    let r = post(&c, LAN_APPLY, peer(204, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    let done = to(&r);
    assert_eq!(done, format!("/ui/router/lan/applied?t={once}"));
    assert_eq!(
        calls(&c),
        vec![("/api/router/lan".to_owned(), json!({"lan_ipaddr": "192.168.8.1", "lan_netmask": "255.255.255.0", "dhcp_enable": "1",
            "dhcp_start": "100", "dhcp_end": "192.168.8.149", "dhcp_lease_time": "12h", "ipaddr": "192.168.8.1", "netmask": "255.255.255.0",
            "ignore": 0, "zte_start": "192.168.8.100", "zte_end": "192.168.8.149", "leasetime": "12h"}))]
    );
    let html = get_with(&c, &done, peer(204, 2), &cookie).await.text();
    assert!(html.contains("局域网设置已更新") && html.contains("<a class=\"btn\" href=\"http://192.168.8.1:9090/ui/\" rel=\"noreferrer\">Open 192.168.8.1</a>"), "{html}");
}

/// DNS: the app's mode and sections from the DoH status; presets fill the
/// form; Apply asks and makes the app's calls; the cache inspector sorts,
/// searches by a post and pages; Clear Cache asks first.
#[tokio::test]
async fn dns_modes_presets_cache_and_clear() {
    let (c, cookie, _) = signed_in(205).await;
    let doh = json!({"running": true, "config": {"upstream_url": "https://1.1.1.1/dns-query"}, "stats": {"cache_entries": 60, "cache_hits": 3, "cache_misses": 1}});
    let entries: Vec<Value> = (0..60).map(|i| json!({"domain": format!("site{i}.fake.example"), "type": if i % 2 == 0 { "A" } else { "AAAA" }, "type_id": 1, "ttl": i * 10})).collect();
    let gets = vec![
        ("/api/router/dns", Ok(json!({"dns_mode": "manual", "prefer_dns_manual": "9.9.9.9", "standby_dns_manual": ""}))),
        ("/api/doh/status", Ok(doh)),
        ("/api/doh/cache", Ok(Value::Array(entries))),
    ];
    let _dev = device::set(&c.app, device::Device { gets, ..Default::default() });
    let r = get_with(&c, DNS, peer(205, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    for want in [
        "<h1>DNS设置</h1>", "<h2>DNS 模式</h2>", "id=\"dm-doh\" name=\"mode\" value=\"doh\" checked", "通过 HTTPS 加密 DNS 查询。",
        "name=\"dns1\" value=\"9.9.9.9\"", "<h2>DoH 上游服务器</h2>", "name=\"url\" value=\"https://1.1.1.1/dns-query\"", "<h2>快速设置</h2>",
        "href=\"/ui/router/dns?preset=google\"><span>谷歌(8.8.8.8)</span>", "填充 DNS 字段。点击“应用”进行保存。", "<h2>DoH 缓存</h2>",
        "<span>命中率</span><span class=\"v\">75.0%</span>", "href=\"/ui/router/dns/cache\">检查缓存</a>", "href=\"/ui/router/dns/cache-clear\">清除缓存</a>",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    let preset = get_with(&c, &format!("{DNS}?preset=quad9"), peer(205, 2), &cookie).await.text();
    assert!(preset.contains("name=\"dns2\" value=\"149.112.112.112\"") && preset.contains("value=\"https://9.9.9.9:5053/dns-query\""), "{preset}");
    let (csrf, once) = (r.input("csrf"), r.input("once"));
    let r = post(&c, "/ui/router/dns/apply", peer(205, 2), &cookie, &body(&csrf, &once, &[("mode", "custom"), ("dns1", ""), ("dns2", ""), ("url", "")])).await;
    r.assert_secure_html(400);
    assert!(r.text().contains("主DNS不能为空"));
    // Custom while DoH runs: asked (it turns DoH off), then the app's two calls.
    let fields = [("mode", "custom"), ("dns1", "1.1.1.1"), ("dns2", "1.0.0.1"), ("url", "")];
    let r = post(&c, "/ui/router/dns/apply", peer(205, 2), &cookie, &body(&csrf, &once, &fields)).await;
    let q = get_with(&c, &to(&r), peer(205, 2), &cookie).await;
    let html = q.text();
    assert!(html.contains("<h2 class=\"q\">使用自定义 DNS 服务器？</h2>") && html.contains("through 1.1.1.1 and 1.0.0.1") && html.contains("DNS 通过 HTTPS 关闭。"), "{html}");
    assert!(!html.contains("You are connected remotely"), "{html}");
    let r = post(&c, "/ui/router/dns/apply", peer(205, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    assert!(get_with(&c, &to(&r), peer(205, 2), &cookie).await.text().contains("<p class=\"msg ok\" role=\"status\">DNS 设置为自定义 (1.1.1.1)</p>"));
    assert_eq!(
        calls(&c),
        vec![
            ("/api/router/dns".to_owned(), json!({"dns_mode": "manual", "prefer_dns_manual": "1.1.1.1", "standby_dns_manual": "1.0.0.1"})),
            ("/api/doh/disable".to_owned(), json!({})),
        ]
    );
    // What the router has already: nothing to apply; only the servers: said so.
    let ask = |fields: Vec<(&'static str, &'static str)>| {
        let (c, cookie, csrf) = (c.clone(), cookie.clone(), csrf.clone());
        async move {
            let once = get_with(&c, DNS, peer(205, 2), &cookie).await.input("once");
            let r = post(&c, "/ui/router/dns/apply", peer(205, 2), &cookie, &body(&csrf, &once, &fields)).await;
            get_with(&c, &to(&r), peer(205, 2), &cookie).await.text()
        }
    };
    let q = ask(vec![("mode", "doh"), ("dns1", "9.9.9.9"), ("dns2", ""), ("url", "https://1.1.1.1/dns-query")]).await;
    assert!(q.contains("<h2 class=\"q\">没有什么可申请的</h2>") && !q.contains("name=\"confirm\""), "{q}");
    let q = ask(vec![("mode", "doh"), ("dns1", "1.1.1.1"), ("dns2", ""), ("url", "https://1.1.1.1/dns-query")]).await;
    assert!(q.contains("<h2 class=\"q\">更换DNS服务器？</h2>") && q.contains("路由器的 DNS 服务器将改为 1。1.1.1.") && !q.contains("encrypted"), "{q}");
    // A remote browser: the router's own lookups use the new servers too.
    let once = get_with(&c, DNS, tailnet(40205), &cookie).await.input("once");
    let r = post(&c, "/ui/router/dns/apply", tailnet(40205), &cookie, &body(&csrf, &once, &[("mode", "custom"), ("dns1", "8.8.8.8"), ("dns2", ""), ("url", "")])).await;
    let q = get_with(&c, &to(&r), tailnet(40205), &cookie).await.text();
    assert!(
        q.contains("<h2 class=\"q\">使用自定义 DNS 服务器？</h2>")
            && q.contains("您已远程连接：路由器’自己的查找（Tailscale重新连接、短信转发webhook）也使用这些服务器，因此可以到达’的服务器可以切断远程访问。"),
        "{q}"
    );
    // DoH: they go through the DoH server.
    let once = get_with(&c, DNS, tailnet(40205), &cookie).await.input("once");
    let r = post(&c, "/ui/router/dns/apply", tailnet(40205), &cookie, &body(&csrf, &once, &[("mode", "doh"), ("dns1", "1.1.1.1"), ("url", "https://9.9.9.9/dns-query")])).await;
    let q = get_with(&c, &to(&r), tailnet(40205), &cookie).await.text();
    assert!(
        q.contains("您已远程连接：路由器’自己的查找（Tailscale重新连接、短信转发webhook）也经过此DoH服务器，因此可以到达’的服务器可以切断远程访问。")
            && !q.contains("use these servers too"),
        "{q}"
    );
    let n = calls(&c).len();
    // DoH while it runs: the servers and the upstream, no enable.
    let once = get_with(&c, DNS, peer(205, 2), &cookie).await.input("once");
    let r = post(&c, "/ui/router/dns/apply", peer(205, 2), &cookie, &body(&csrf, &once, &[("mode", "doh"), ("dns1", "1.1.1.1"), ("url", "https://9.9.9.9/dns-query")])).await;
    let q = get_with(&c, &to(&r), peer(205, 2), &cookie).await;
    assert!(q.text().contains("<h2 class=\"q\">更改 DoH 服务器？</h2>") && q.text().contains("路由器的 DNS 服务器将改为 1。1.1.1."), "{}", q.text());
    assert!(!q.text().contains("connected remotely"), "{}", q.text());
    post(&c, "/ui/router/dns/apply", peer(205, 2), &cookie, &format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"))).await;
    let sent: Vec<String> = calls(&c).into_iter().map(|(p, _)| p).collect();
    assert_eq!(sent[n..], ["/api/router/dns", "/api/doh/config"]);
    // The cache: by TTL, 50 at a time, a posted search.
    let r = get_with(&c, "/ui/router/dns/cache", peer(205, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    assert!(html.contains("<div class=\"lead\"><a class=\"nl\" href=\"/ui/router/dns\">完成</a></div><h1>DNS 缓存（60）</h1>") && !html.contains("60 entries"), "{html}");
    assert!(html.contains("<title>DNS 缓存 · U60 Pro</title>"), "{html}");
    assert!(html.find("site59.fake.example").unwrap() < html.find("site58.fake.example").unwrap());
    assert!(!html.contains("site9.fake.example<") && html.contains("href=\"/ui/router/dns/cache?n=100\">加载更多</a>"), "{html}");
    assert!(html.contains("<span class=\"badge pu\">AAAA</span>") && html.contains("<small>9m 50s</small>"));
    assert!(get_with(&c, "/ui/router/dns/cache?n=100", peer(205, 2), &cookie).await.text().contains("site0.fake.example"));
    let found = post(&c, "/ui/router/dns/cache", peer(205, 2), &cookie, &format!("csrf={csrf}&q=SITE5")).await;
    assert_eq!(found.status, 200);
    assert!(found.text().contains("<h1>DNS 缓存（11）</h1>") && !found.text().contains("site4.fake.example"), "{}", found.text());
    // Clear: asked, then the app's call and line.
    let q = get_with(&c, "/ui/router/dns/cache-clear", peer(205, 2), &cookie).await;
    assert!(q.text().contains("<h2 class=\"q\">清除DNS缓存？</h2>") && q.text().contains("The 60 cached answers are removed"), "{}", q.text());
    let r = post(&c, "/ui/router/dns/cache-clear", peer(205, 2), &cookie, &format!("csrf={csrf}&once={}&confirm=1", q.input("once"))).await;
    assert!(get_with(&c, &to(&r), peer(205, 2), &cookie).await.text().contains("DNS 缓存清除"));
    assert_eq!(calls(&c).last().unwrap().0, "/api/doh/cache/clear");
}

/// Posts without a session never run; an expired form or question sends
/// nothing; logging out drops the session's drafts (typed passwords).
#[tokio::test]
async fn sessions_tokens_and_logout() {
    let (c, cookie, id) = signed_in(206).await;
    let _fake = fake::set(&c.app, wifi_status());
    let _dev = device::set(&c.app, device::Device::default());
    let r = post(&c, WIFI_APPLY, peer(206, 2), "", "csrf=x&once=y").await;
    assert_eq!(to(&r), "/ui/login?next=%2Fui%2Frouter%2Fwifi&expired=1");
    let page = get_with(&c, WIFI, peer(206, 2), &cookie).await;
    let csrf = page.input("csrf");
    // An unknown token: nothing drafted.
    let r = post(&c, WIFI_APPLY, peer(206, 2), &cookie, &body(&csrf, "00000000000000ff", &wifi_fields(&[("ssid_2g", "X-Fake")]))).await;
    assert!(get_with(&c, &to(&r), peer(206, 2), &cookie).await.text().contains("This form expired, so nothing was sent."));
    // A question's post with another revision: nothing sent.
    let once = page.input("once");
    let r = post(&c, WIFI_APPLY, peer(206, 2), &cookie, &body(&csrf, &once, &wifi_fields(&[("key_2g", "another-fake-pass")]))).await;
    assert_eq!(r.status, 303);
    let r = post(&c, WIFI_APPLY, peer(206, 2), &cookie, &format!("csrf={csrf}&once={once}&d=0000000000000000&confirm=1")).await;
    assert!(get_with(&c, &to(&r), peer(206, 2), &cookie).await.text().contains("This page expired, so nothing was sent."));
    assert!(calls(&c).is_empty());
    assert!(c.drafts.get(&once, id, c.app.auth.now()).is_some());
    let logout = get_with(&c, "/ui/logout", peer(206, 2), &cookie).await.input("csrf");
    post(&c, "/ui/logout", peer(206, 2), &cookie, &format!("csrf={logout}")).await;
    assert!(c.drafts.get(&once, id, c.app.auth.now()).is_none());
}

/// Every Router settings post: without a session the login (nothing
/// run), without the CSRF token or from another origin `403`; nothing is
/// sent in any case.
#[tokio::test]
async fn every_settings_post_checks_session_csrf_and_origin() {
    let (c, cookie, _) = signed_in(207).await;
    let _fake = fake::set(&c.app, wifi_status());
    let _dev = device::set(&c.app, apn_device("1"));
    let csrf = get_with(&c, "/ui/logout", peer(207, 2), &cookie).await.input("csrf");
    for path in [
        WIFI_APPLY, GUEST_APPLY, "/ui/router/apn/mode", "/ui/router/apn/save", "/ui/router/apn/activate", "/ui/router/apn/delete", LAN_APPLY,
        "/ui/router/dns/apply", "/ui/router/dns/cache-clear",
    ] {
        let fields = "manual=1&id=manu2&confirm=1&once=00000000000000ff";
        let r = post(&c, path, peer(207, 3), "", &format!("csrf={csrf}&{fields}")).await;
        assert!(to(&r).starts_with("/ui/login?next=%2Fui%2Frouter%2F") && to(&r).ends_with("&expired=1"), "{path}");
        assert_eq!(post(&c, path, peer(207, 3), &cookie, fields).await.status, 403, "{path}");
        let b = format!("csrf={csrf}&{fields}");
        let cross = form(path, peer(207, 3), &cookie, &b).header("origin", "http://evil.example").body(Body::from(b.clone())).unwrap();
        assert_eq!(send(&c, cross).await.status, 403, "{path}");
        // A form token that was never issued: nothing sent.
        let r = post(&c, path, peer(207, 3), &cookie, &b).await;
        assert!(r.status == 303 || r.status == 400, "{path}: {}", r.status);
    }
    assert!(calls(&c).is_empty(), "{:?}", calls(&c));
}
