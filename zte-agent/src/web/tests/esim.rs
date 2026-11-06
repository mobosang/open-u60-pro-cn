//! Router › eSIM Profiles (the list, Switch, Delete, Rename, Send now)
//! through the whole router. Device writes go to `pages::device`, page-load
//! reads to its `gets`, topics to `pages::fake`. Every value is a made-up
//! fixture: ICCIDs 8900000000000000xxx, an EID of zeros, the test APN.

use serde_json::json;

use super::*;
use crate::events::hub::ReadOut;
use crate::events::Topic;
use crate::web::pages::{device, fake};

const LIST: &str = "/ui/router/esim";
const SWITCH: &str = "/ui/router/esim/switch";
const DELETE: &str = "/ui/router/esim/delete";
const RENAME: &str = "/ui/router/esim/rename";
const RECEIPTS: &str = "/ui/router/esim/receipts";
const EID: &str = "00000000000000000000000000000000";
const ICCIDS: [&str; 4] = ["8900000000000000101", "8900000000000000202", "8900000000000000303", "8900000000000000404"];

fn calls(c: &Ctx) -> Vec<(String, Value)> {
    device::calls(&c.app)
}

fn tailnet(port: u16) -> Peer {
    Peer(SocketAddr::from(([127, 0, 0, 1], port)))
}

fn to(r: &Reply) -> String {
    assert_eq!(r.status, 303, "{}", r.text());
    r.header("location").unwrap().to_owned()
}

/// Four profiles as `GET /api/esim/profiles` gives them: the active one,
/// two that warn (the router's manual APN; a travel profile with roaming
/// off) and a test profile that cannot be switched to; two receipts wait.
fn profiles() -> Value {
    json!({
        "profiles": [
            {"iccid": ICCIDS[0], "state": "enabled", "nickname": "GOMO u60", "provider": "GOMO", "name": "GOMO", "class": "operational"},
            {"iccid": ICCIDS[1], "state": "disabled", "nickname": "DITO u60", "provider": "DITO", "name": "DITO", "class": "operational",
             "switch_warnings": ["manual_apn"]},
            {"iccid": ICCIDS[2], "state": "disabled", "nickname": null, "provider": "Travel - PHL", "name": "Klook_app", "class": "operational",
             "switch_warnings": ["manual_apn", "roaming_off"]},
            {"iccid": ICCIDS[3], "state": "disabled", "nickname": null, "provider": "Example", "name": "Test profile", "class": "test"},
        ],
        "enabled_iccid": ICCIDS[0], "observed_at": 1_761_200_000, "observed_seq": 3, "generation": 7, "agent_instance": 1,
        "updating": false, "stale": false, "error": null, "pending_receipts": 2,
        "manual_apn": {"name": "Example", "apn": "internet.example"},
    })
}

fn chip() -> Value {
    json!({"eid": EID, "firmware": "4.2.0", "free_nvm_bytes": 934_475, "sgp22_version": "2.2.2"})
}

fn device_with(replies: Vec<(&'static str, (u16, Value))>, extra: Vec<(&'static str, Result<Value, String>)>) -> device::Device {
    let mut gets = vec![("/api/esim/profiles", Ok(profiles())), ("/api/esim/chip", Ok(chip()))];
    gets.extend(extra);
    device::Device { gets, replies, ..Default::default() }
}

/// The key of profile `i` in the list's links.
fn key_of(html: &str, i: usize) -> String {
    let name = ["GOMO u60", "DITO u60", "Klook_app", "Test profile"][i];
    let at = html.find(&format!("{name} 的选项")).unwrap_or_else(|| panic!("{name} in {html}"));
    html[at..].split("/ui/router/esim/rename?p=").nth(1).unwrap()[..16].to_owned()
}

/// The next frame that patches `id` (heartbeats may come first).
async fn patch(b: &mut axum::body::BodyDataStream, id: &str) -> String {
    loop {
        let f = frame(b).await.unwrap_or_else(|| panic!("a patch of {id}"));
        if f.contains(&format!("id=\"{id}\"")) {
            return f;
        }
    }
}

fn page_csrf(html: &str) -> String {
    html.split("name=\"csrf\" value=\"").nth(1).unwrap().split('"').next().unwrap().to_owned()
}

fn once(html: &str) -> String {
    html.split("name=\"once\" value=\"").nth(1).unwrap().split('"').next().unwrap().to_owned()
}

/// The list as served: the app's cards and words, identifiers masked
/// (ICCIDs to their last 4, the EID until asked for), profiles by opaque
/// keys; the stream patches the same fragments.
#[tokio::test]
async fn profiles_follow_the_app_with_identifiers_masked() {
    let (c, cookie, _) = signed_in(160).await;
    let _dev = device::set(&c.app, device_with(vec![], vec![]));
    let r = get_with(&c, LIST, peer(160, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    for want in [
        "<h1>eSIM 配置文件</h1>", "<a class=\"nl back\" href=\"/ui/router\">路由器</a>",
        "<a class=\"nl ic\" href=\"/ui/router/esim/add\" aria-label=\"添加 eSIM\">",
        "<a class=\"nl ic\" href=\"/ui/router/esim\" aria-label=\"刷新配置文件和连接\">",
        "<h2>在您的 eSIM 上</h2>", "<span class=\"cnt\">4 profiles</span>", "最后阅读 ",
        "<small class=\"active\">", "已启用</small><b>GOMO u60</b><small class=\"cap\">GOMO</small><small class=\"cap num\">…0101</small>",
        "<h2 class=\"esub\">切换至another profile</h2>", "<b>Klook_app</b><small class=\"cap\">Travel - PHL</small>",
        "aria-label=\"不可用\"", "添加 eSIM</a>", "切换会使互联网暂停大约一分钟。您的路由器的 Wi-Fi 保持开启状态。",
        "<b>运营商收据</b><small class=\"cap\">2 waiting · Retried automatically</small>", "为什么这些人在等待？",
        "action=\"/ui/router/esim/receipts\"", "立即发送</button>", "<b>eSIM 卡</b><small class=\"cap\">卡片详情与标识符</small>",
        "<span>固件</span><span class=\"v\">4.2.0</span>", "<span>可用内存</span><span class=\"v\">913 KB</span>",
        "•••• •••• •••• …0000", "href=\"/ui/router/esim?eid=1#chip\" aria-label=\"揭示EID\"", "@get('/ui/live/esim/",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    // The active profile is not switched to or deleted; the test profile not switched to.
    let k: Vec<String> = (0..4).map(|i| key_of(&html, i)).collect();
    assert!(k.iter().all(|k| k.len() == 16 && k.bytes().all(|b| b.is_ascii_hexdigit())));
    assert!(!html.contains(&format!("{SWITCH}?p={}", k[0])) && !html.contains(&format!("{DELETE}?p={}", k[0])));
    assert!(html.contains(&format!("{SWITCH}?p={}", k[1])) && html.contains(&format!("{DELETE}?p={}", k[1])));
    assert!(!html.contains(&format!("{SWITCH}?p={}", k[3])) && html.contains(&format!("{DELETE}?p={}", k[3])));
    for secret in ICCIDS.iter().copied().chain([EID]) {
        assert!(!html.contains(secret), "{secret}");
    }
    // Delete profile with the app's trash icon.
    let del = html.split(&format!("href=\"{DELETE}?p={}\"", k[1])).nth(1).unwrap();
    assert!(del.split("</a>").next().unwrap().contains("M4.5 6.5h15") && del.contains("删除配置文件"), "{del}");
    // The stream: the same fragments, never the EID or an ICCID (this
    // browser's 1 s refresh preference does not change the screen's own).
    let resp = open_live(&c, &live_url(&html, "esim"), peer(160, 3), &format!("{cookie}; zte_ui_refresh=1000")).await;
    for t in [Topic::EsimSwitch, Topic::EsimDownload, Topic::Modem, Topic::Signal] {
        assert_eq!(c.app.hub.demand(t), 1, "{t:?}");
    }
    // The connection card's signal every 10 s (the app's idle 12 s), not 2.
    assert_eq!(c.app.hub.cadence(Topic::Signal), Some(std::time::Duration::from_secs(10)));
    let mut b = resp.into_body().into_data_stream();
    let first = frame(&mut b).await.unwrap();
    for (id, part) in frame_parts(&first).iter().filter(|(id, _)| id.starts_with("esim-")) {
        assert!(html.contains(part.as_str()), "{id} differs from the page");
    }
    // A download: busy (the page's controls dim); once it has added a
    // profile, the list says it changed.
    let offer = |t: Topic, v: Value| c.app.hub.offer(t, ReadOut { value: Some(v), error: None }, std::time::Instant::now());
    offer(Topic::EsimDownload, json!({"state": "running", "step": "es9p_authenticate_client", "started_at": 1_761_200_100}));
    let next = patch(&mut b, "esim-dl").await;
    assert!(next.contains("<div class=\"busy\" id=\"esim-dl\">") && !next.contains("Changed"), "{next}");
    offer(Topic::EsimDownload, json!({"state": "done", "provider": "Example Travel", "iccid": "8900000000000000505", "started_at": 1_761_200_100}));
    let next = patch(&mut b, "esim-upd").await;
    assert!(next.contains("<b>Example Travel added.</b>") && next.contains("修改为· <a href=\"/ui/router/esim\">刷新</a>"), "{next}");
    assert!(!next.contains("class=\"busy\" id=\"esim-dl\"") && !next.contains("0505"), "{next}");
    // A switch starts elsewhere: its card follows, the list says "Updating…".
    let sw = json!({"state": "reloading", "target_iccid": ICCIDS[1], "generation": 8, "sim_epoch": 0});
    c.app.hub.offer(Topic::EsimSwitch, ReadOut { value: Some(sw), error: None }, std::time::Instant::now());
    let next = frame(&mut b).await.unwrap();
    assert!(next.contains("<b>正在重新加载 SIM</b>") && next.contains("切换至DITO u60") && next.contains("更新…"), "{next}");
    assert!(!next.contains(ICCIDS[1]) && !next.contains(EID), "{next}");
    // It ends without data, which comes up later: the card says connected
    // (the app's next check); the list changed.
    offer(Topic::EsimSwitch, json!({"state": "no_data", "target_iccid": ICCIDS[1], "generation": 8, "sim_epoch": 0}));
    let next = patch(&mut b, "esim-conn").await;
    assert!(next.contains("<b>连接需要处理</b>") && next.contains("修改为· "), "{next}");
    offer(Topic::Modem, json!({"data": {"connect_status": "ipv4_ipv6_connected"}}));
    let next = patch(&mut b, "esim-conn").await;
    assert!(next.contains("切换至DITO u60") && next.contains("<b>移动数据已连接</b>"), "{next}");
    drop(b);
    // Loaded while the SIM reloads (the agent answers with the list read
    // before the switch): the target shows as active once its enable went
    // through; not while the outcome is unknown.
    let _sw = fake::set(&c.app, json!({"state": "reloading", "target_iccid": ICCIDS[1], "generation": 9, "note": null}));
    let html = get_with(&c, LIST, peer(160, 5), &cookie).await.text();
    assert!(html.contains("已启用</small><b>DITO u60</b>") && !html.contains("已启用</small><b>GOMO u60</b>"), "{html}");
    let _sw = fake::set(&c.app, json!({"state": "reloading", "target_iccid": ICCIDS[1], "generation": 9, "note": "enable error: timed out"}));
    let html = get_with(&c, LIST, peer(160, 5), &cookie).await.text();
    assert!(html.contains("已启用</small><b>GOMO u60</b>"), "{html}");
    // While it runs, the controls are dimmed (CSS keyed on the busy card).
    assert!(html.contains("class=\"card econn esw busy\" id=\"esim-conn\""), "{html}");
    let css = include_str!("../assets/app.css");
    assert!(css.contains("body:has(#esim-conn.busy, #esim-dl.busy) :is(.prow, a[href=\"/ui/router/esim/add\"]"), "dimmed while busy");
    drop(_sw);
    // Asked for: the EID whole, on this page load only.
    let shown = get_with(&c, &format!("{LIST}?eid=1"), peer(160, 4), &cookie).await.text();
    assert!(shown.contains(&format!("<span class=\"v mono\" translate=\"no\">{EID}</span>")), "{shown}");
    assert!(shown.contains("<details class=\"card chip\" id=\"chip\" open>") && shown.contains("aria-label=\"隐藏EID\""), "{shown}");
}

/// No profiles: the app's empty card; a failed read: its error card.
#[tokio::test]
async fn empty_and_failed_lists() {
    let (c, cookie, _) = signed_in(161).await;
    let none = json!({"profiles": [], "observed_at": 0, "generation": 1, "updating": false, "error": null});
    let _dev = device::set(&c.app, device::Device { gets: vec![("/api/esim/profiles", Ok(none)), ("/api/esim/chip", Err("busy".into()))], ..Default::default() });
    let html = get_with(&c, LIST, peer(161, 2), &cookie).await.text();
    assert!(html.contains("<b>还没有配置文件</b>") && html.contains("使用运营商提供的二维码添加一个。"), "{html}");
    assert!(html.contains("<div class=\"esim none\">") && !html.contains("eSIM 卡"), "{html}");
    assert!(html.contains("<a class=\"btn btn-wide teal\" href=\"/ui/router/esim/add\">"), "the app's prominent teal button: {html}");
    // Not while a download runs (the app's `!download.isBusy`).
    let _dl = fake::set(&c.app, json!({"state": "running", "step": "es9p_authenticate_client"}));
    let html = get_with(&c, LIST, peer(161, 2), &cookie).await.text();
    assert!(!html.contains("还没有配置文件") && html.contains("正在连接运营商…"), "{html}");
    drop(_dl);
    let stale = json!({"profiles": profiles()["profiles"], "observed_at": 0, "generation": 7, "updating": false,
        "error": {"kind": "sim_reloading", "message": "euicc_init failed", "at": 1}});
    let _dev = device::set(&c.app, device::Device { gets: vec![("/api/esim/profiles", Ok(stale))], ..Default::default() });
    let html = get_with(&c, LIST, peer(161, 3), &cookie).await.text();
    assert!(html.contains("无法刷新配置文件") && html.contains("切换后 eSIM 仍在准备中。 Showing the list read at an earlier time."), "{html}");
    let _dev = device::set(&c.app, device::Device { gets: vec![("/api/esim/profiles", Err("eSIM manager busy; retry shortly".into()))], ..Default::default() });
    let html = get_with(&c, LIST, peer(161, 4), &cookie).await.text();
    assert!(html.contains("无法读取 eSIM 配置文件：eSIM manager busy; retry shortly") && html.contains("<span class=\"cnt\">尚未阅读</span>"), "{html}");
}

/// Switch: the app's dialog with its warnings (and, remotely, that the
/// connection drops and nothing switches back); one enable per token; the
/// operation page follows the switch; a replay lands on it again.
#[tokio::test]
async fn switch_asks_then_follows_the_switch_once() {
    let (c, cookie, _) = signed_in(162).await;
    let _dev = device::set(&c.app, device_with(vec![], vec![]));
    let list = get_with(&c, LIST, peer(162, 2), &cookie).await.text();
    let (dito, klook, active, test) = (key_of(&list, 1), key_of(&list, 2), key_of(&list, 0), key_of(&list, 3));
    let r = get_with(&c, &format!("{SWITCH}?p={klook}"), peer(162, 3), &cookie).await;
    r.assert_secure_html(200);
    let q = r.text();
    for want in [
        "<h1>确认</h1>", "<h2 class=\"q\">切换到Klook_app？</h2>",
        "路由器将断开 GOMO u60 并通过 Klook_app 连接。互联网将中断约一分钟，Wi-Fi 保持开启。",
        "路由器当前使用手动 APN“internet.example”。如果 Klook_app 需要其他 APN，请在切换后前往移动网络修改，否则可能无法使用数据。",
        "Klook_app looks like a travel (roaming) profile and 数据漫游 is off, so it will have no data until you turn 数据漫游 on.",
        "<button class=\"btn\" type=\"submit\">切换至Klook_app</button>", "href=\"/ui/router/esim\">取消</a>",
    ] {
        assert!(q.contains(want), "{want} in {q}");
    }
    assert!(!q.contains("connected remotely") && !q.contains(ICCIDS[2]));
    // Remotely: the remote lines.
    let remote = get_with(&c, &format!("{SWITCH}?p={dito}"), tailnet(4100), &cookie).await.text();
    assert!(remote.contains(super::super::pages::confirm::REMOTE) && remote.contains("如果新配置文件没有收到数据，路由器不会自行切换回来。"), "{remote}");
    assert!(!remote.contains("travel (roaming)"));
    // Nothing to ask: the active profile, a test profile, an unknown key.
    for (p, title) in [(&active, "GOMO u60 is already active."), (&test, "Test profile can’t be switched to here.")] {
        let page = get_with(&c, &format!("{SWITCH}?p={p}"), peer(162, 4), &cookie).await.text();
        assert!(page.contains(&format!("<h2 class=\"q\">{title}</h2>")) && !page.contains("name=\"once\""), "{page}");
    }
    let gone = get_with(&c, &format!("{SWITCH}?p=0123456789abcdef"), peer(162, 4), &cookie).await;
    assert_eq!(gone.status, 404);
    assert!(gone.text().contains("此配置文件不再位于您的 eSIM 上。"));
    // A post without `confirm=1` goes to the question; nothing is sent.
    let (csrf, token) = (page_csrf(&q), once(&q));
    let r = post(&c, SWITCH, peer(162, 5), &cookie, &format!("csrf={csrf}&p={klook}&once={token}")).await;
    assert_eq!(to(&r), format!("{SWITCH}?p={klook}"));
    assert!(calls(&c).is_empty());
    // Two posts at once: one enable, both on the operation page.
    let body = format!("csrf={csrf}&p={klook}&once={token}&confirm=1");
    let (a, b) = tokio::join!(post(&c, SWITCH, peer(162, 5), &cookie, &body), post(&c, SWITCH, peer(162, 5), &cookie, &body));
    assert_eq!(to(&a), format!("/ui/op/{token}"));
    assert_eq!(to(&b), format!("/ui/op/{token}"));
    assert_eq!(calls(&c), [("/api/esim/enable".to_owned(), json!({"iccid": ICCIDS[2]}))]);
    let r = post(&c, SWITCH, peer(162, 5), &cookie, &body).await;
    assert_eq!(to(&r), format!("/ui/op/{token}"));
    assert_eq!(calls(&c).len(), 1);
    // The operation page: the app's card and steps; a way back, never a switch back.
    let _sw = fake::set(&c.app, json!({"state": "card_confirmed", "target_iccid": ICCIDS[2], "generation": 8}));
    let op = get_with(&c, &format!("/ui/op/{token}"), peer(162, 6), &cookie).await;
    op.assert_secure_html(200);
    let op = op.text();
    for want in [
        "<h1 id=\"op-title\">正在注册网络</h1>", "切换至Klook_app", "重新加载SIM</span><small>完成</small>",
        "连接到网络</span><small>进行中</small>", "Wi-Fi 保持连接 · 通常 15–45 秒",
        "href=\"/ui/router/esim\">返回 eSIM 配置文件</a>", "@get('/ui/live/op/",
    ] {
        assert!(op.contains(want), "{want} in {op}");
    }
    assert!(!op.contains(ICCIDS[2]) && !op.to_ascii_lowercase().contains("如果新配置文件没有收到数据，路由器不会自行切换回来。"), "{op}");
    // Its stream: only the switch and the signal, until the switch ends
    // (with "Check again" when it needs attention); nothing sampled after.
    let resp = open_live(&c, &live_url(&op, "op"), peer(162, 6), &cookie).await;
    for t in Topic::ALL {
        let want = u16::from(matches!(t, Topic::EsimSwitch | Topic::Signal));
        assert_eq!(c.app.hub.demand(t), want, "{t:?}");
    }
    let mut b = resp.into_body().into_data_stream();
    let _ = frame(&mut b).await.unwrap();
    let end = json!({"state": "not_applied", "reason": "card_busy", "target_iccid": ICCIDS[2], "generation": 8});
    c.app.hub.offer(Topic::EsimSwitch, ReadOut { value: Some(end), error: None }, std::time::Instant::now());
    let mut last = String::new();
    while let Some(f) = frame(&mut b).await {
        last.push_str(&f);
    }
    assert!(last.contains("<b>开关需要注意的地方</b>") && last.contains("href=\"\">再次检查</a>"), "{last}");
    assert!(last.contains("<p class=\"live\" id=\"live-status\"></p>") && !last.contains(ICCIDS[2]), "{last}");
    assert!(Topic::ALL.into_iter().all(|t| c.app.hub.demand(t) == 0));
    let _sw = fake::set(&c.app, json!({"state": "no_data", "reason": "roaming_off", "target_iccid": ICCIDS[2], "generation": 8}));
    let done = get_with(&c, &format!("/ui/op/{token}"), peer(162, 6), &cookie).await.text();
    assert!(done.contains("<h1 id=\"op-title\">连接需要处理</h1>") && !done.contains("/ui/live/op/"), "{done}");
    assert!(done.contains("此配置文件在本地网络上漫游，并且数据漫游已关闭。在移动网络中打开数据漫游。"));
}

/// The agent's refusals: busy reads as the app's busy card, others as
/// "Switch failed: …" with the app's guidance; nothing is offered or sent
/// while a switch or a download runs.
#[tokio::test]
async fn switch_refusals_read_as_the_app() {
    let (c, cookie, _) = signed_in(163).await;
    let busy = (409, json!({"ok": false, "error": "eSIM manager busy; retry shortly"}));
    let _dev = device::set(&c.app, device_with(vec![("/api/esim/enable", busy)], vec![]));
    let list = get_with(&c, LIST, peer(163, 2), &cookie).await.text();
    let dito = key_of(&list, 1);
    let q = get_with(&c, &format!("{SWITCH}?p={dito}"), peer(163, 3), &cookie).await.text();
    let body = format!("csrf={}&p={dito}&once={}&confirm=1", page_csrf(&q), once(&q));
    let r = post(&c, SWITCH, peer(163, 3), &cookie, &body).await;
    let line = to(&r);
    assert!(line.starts_with("/ui/router/esim?r="), "{line}");
    let html = get_with(&c, &line, peer(163, 3), &cookie).await.text();
    assert!(html.contains("<b>eSIM in use</b><p>eSIM 忙；稍后再试。</p>") && !html.contains("重新加载配置文件"), "{html}");
    // Airplane mode: the app's words and guidance.
    let lpm = (409, json!({"ok": false, "error": "the modem is in airplane / low-power mode; turn it back on before switching profiles"}));
    let _dev = device::set(&c.app, device_with(vec![("/api/esim/enable", lpm)], vec![]));
    let q = get_with(&c, &format!("{SWITCH}?p={dito}"), peer(163, 4), &cookie).await.text();
    let body = format!("csrf={}&p={dito}&once={}&confirm=1", page_csrf(&q), once(&q));
    let html = get_with(&c, &to(&post(&c, SWITCH, peer(163, 4), &cookie, &body).await), peer(163, 4), &cookie).await.text();
    for want in [
        "<b>Can’t 现在就完成这个</b><p>切换失败： the modem is in airplane / low-power mode",
        "打开路由器设置→移动网络检查调制解调器模式。", "href=\"/ui/router/esim\">重新加载配置文件</a>",
        "重新加载只会读取列表，不会重复执行操作。",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
    // A switch runs: the question offers nothing and a post sends nothing.
    let _sw = fake::set(&c.app, json!({"state": "reloading", "generation": 9}));
    let q2 = get_with(&c, &format!("{SWITCH}?p={dito}"), peer(163, 5), &cookie).await.text();
    assert!(q2.contains("<h2 class=\"q\">eSIM 忙；稍后再试。</h2>") && !q2.contains("name=\"once\""), "{q2}");
    let token = c.once.fresh(&c.app.auth, c.app.auth.authenticate(Kind::Browser, cookie.trim_start_matches("zte_ui_session=")).unwrap().id);
    let n = calls(&c).len();
    let body = format!("csrf={}&p={dito}&once={token}&confirm=1", page_csrf(&q));
    let r = post(&c, SWITCH, peer(163, 5), &cookie, &body).await;
    assert!(to(&r).starts_with("/ui/router/esim?r="));
    assert_eq!(calls(&c).len(), n);
    // The token was given back (nothing sent): it still works once the switch ends.
    assert_eq!(c.once.get(&token, c.app.auth.authenticate(Kind::Browser, cookie.trim_start_matches("zte_ui_session=")).unwrap().id, c.app.auth.now()), Some(crate::web::store::Token::Fresh));
}

/// Delete: the app's dialog (inactive profiles only), one call, its line;
/// Rename: the sheet, the 64-byte check, its lines, nothing for no change.
#[tokio::test]
async fn delete_and_rename_follow_the_app() {
    let (c, cookie, _) = signed_in(164).await;
    let deleted = (200, json!({"ok": true, "data": {"deleted_iccid": ICCIDS[1], "operator_notified": false}}));
    let _dev = device::set(&c.app, device_with(vec![("/api/esim/delete", deleted)], vec![]));
    let list = get_with(&c, LIST, peer(164, 2), &cookie).await.text();
    let (gomo, dito) = (key_of(&list, 0), key_of(&list, 1));
    let q = get_with(&c, &format!("{DELETE}?p={dito}"), peer(164, 3), &cookie).await.text();
    for want in [
        "<h2 class=\"q\">永久删除 DITO u60？</h2>",
        "此操作无法撤销。要恢复此 eSIM，你需要从 DITO 获取新的二维码。",
        "<button class=\"btn btn-danger\" type=\"submit\">删除 DITO u60</button>",
    ] {
        assert!(q.contains(want), "{want} in {q}");
    }
    let active = get_with(&c, &format!("{DELETE}?p={gomo}"), peer(164, 3), &cookie).await.text();
    assert!(active.contains("<h2 class=\"q\">GOMO u60 is active.</h2>") && !active.contains("name=\"once\""), "{active}");
    let body = format!("csrf={}&p={dito}&once={}&confirm=1", page_csrf(&q), once(&q));
    let (a, b) = tokio::join!(post(&c, DELETE, peer(164, 4), &cookie, &body), post(&c, DELETE, peer(164, 4), &cookie, &body));
    assert_eq!(to(&a), to(&b));
    assert_eq!(calls(&c), [("/api/esim/delete".to_owned(), json!({"iccid": ICCIDS[1]}))]);
    let html = get_with(&c, &to(&a), peer(164, 4), &cookie).await.text();
    assert!(html.contains("<b>配置文件已更新</b><p>DITO u60 deleted. The operator will be notified on the next eSIM change.</p>"), "{html}");
    // Rename: the sheet.
    let r = get_with(&c, &format!("{RENAME}?p={dito}"), peer(164, 5), &cookie).await;
    r.assert_secure_html(200);
    let sheet = r.text();
    for want in [
        "<h1>重命名配置文件</h1>", "<div class=\"lead\"><a class=\"nl\" href=\"/ui/router/esim\">取消</a></div>", "<h2>昵称</h2>",
        "name=\"nickname\" value=\"DITO u60\" placeholder=\"DITO\" maxlength=\"64\"", "留空以使用运营商名称。",
        "<span class=\"num\">8 / 64 bytes</span>", "名称中的表情符号或重音字母每个字符可能占用多个字节。", ">保存</button>",
    ] {
        assert!(sheet.contains(want), "{want} in {sheet}");
    }
    let rename = |nick: &str, token: &str| format!("csrf={}&p={dito}&once={token}&nickname={}", page_csrf(&sheet), enc(nick));
    // Over 64 bytes (32 two-byte letters and one more): the sheet again, nothing sent.
    let long = "é".repeat(33);
    let r = post(&c, RENAME, peer(164, 5), &cookie, &rename(&long, &once(&sheet))).await;
    assert_eq!(r.status, 400);
    assert!(r.text().contains("请使用单行文本，最多 64 字节。") && r.text().contains("66 / 64 bytes") && r.text().contains("aria-invalid=\"true\""), "{}", r.text());
    let n = calls(&c).len();
    // Unchanged: back, nothing sent.
    let r = post(&c, RENAME, peer(164, 5), &cookie, &rename("DITO u60", &once(&sheet))).await;
    assert_eq!(to(&r), LIST);
    assert_eq!(calls(&c).len(), n);
    // A new name, then none: the app's lines.
    let r = post(&c, RENAME, peer(164, 5), &cookie, &rename("  Home  ", &once(&sheet))).await;
    let html = get_with(&c, &to(&r), peer(164, 5), &cookie).await.text();
    assert!(html.contains("<p>更名为Home。</p>"), "{html}");
    assert_eq!(calls(&c).last().unwrap(), &("/api/esim/nickname".to_owned(), json!({"iccid": ICCIDS[1], "nickname": "首页"})));
    let sheet2 = get_with(&c, &format!("{RENAME}?p={dito}"), peer(164, 6), &cookie).await.text();
    let body = format!("csrf={}&p={dito}&once={}&nickname=", page_csrf(&sheet2), once(&sheet2));
    let html = get_with(&c, &to(&post(&c, RENAME, peer(164, 6), &cookie, &body).await), peer(164, 6), &cookie).await.text();
    assert!(html.contains("<p>昵称已清除。</p>"), "{html}");
}

/// Send now: the agent sends in the background; the line after the wait
/// says what the count came to.
#[tokio::test]
async fn send_now_reports_the_receipts() {
    let (c, cookie, _) = signed_in(165).await;
    let left = ("/api/esim/notifications", Ok(json!({"count": 1, "pending": []})));
    let _dev = device::set(&c.app, device_with(vec![("/api/esim/notifications/process", (202, json!({"ok": true, "data": {}})))], vec![left]));
    let list = get_with(&c, LIST, peer(165, 2), &cookie).await.text();
    let body = format!("csrf={}&once={}", page_csrf(&list), once(&list));
    let html = get_with(&c, &to(&post(&c, RECEIPTS, peer(165, 3), &cookie, &body).await), peer(165, 3), &cookie).await.text();
    assert!(html.contains("<b>运营商收据</b><p>Some receipts couldn&#39;t be sent yet. The router keeps retrying.</p>")
        || html.contains("<b>运营商收据</b><p>部分收据尚未发送。路由器不断重试。</p>"), "{html}");
    assert_eq!(calls(&c), [("/api/esim/notifications/process".to_owned(), json!({}))]);
    let sent = ("/api/esim/notifications", Ok(json!({"count": 0, "pending": []})));
    let _dev = device::set(&c.app, device_with(vec![], vec![sent]));
    let list = get_with(&c, LIST, peer(165, 4), &cookie).await.text();
    let body = format!("csrf={}&once={}", page_csrf(&list), once(&list));
    let html = get_with(&c, &to(&post(&c, RECEIPTS, peer(165, 4), &cookie, &body).await), peer(165, 4), &cookie).await.text();
    assert!(html.contains("<b>运营商收据</b><p>发送的运营商收据。</p>"), "{html}");
}

/// Every eSIM post refuses a missing CSRF token or a foreign origin and
/// needs a session; nothing is sent.
#[tokio::test]
async fn esim_posts_check_session_and_csrf() {
    let (c, cookie, _) = signed_in(166).await;
    let _dev = device::set(&c.app, device_with(vec![], vec![]));
    let list = get_with(&c, LIST, peer(166, 2), &cookie).await.text();
    let dito = key_of(&list, 1);
    for path in [SWITCH, DELETE, RENAME, RECEIPTS] {
        let body = format!("p={dito}&once={}&confirm=1&nickname=x", once(&list));
        assert_eq!(post(&c, path, peer(166, 3), &cookie, &body).await.status, 403, "{path}");
        let r = form(path, peer(166, 3), &cookie, &format!("csrf={}&{body}", page_csrf(&list)))
            .header("origin", "http://evil.example")
            .body(Body::from(format!("csrf={}&{body}", page_csrf(&list))))
            .unwrap();
        assert_eq!(send(&c, r).await.status, 403, "{path}");
        let r = post(&c, path, peer(166, 4), "", &format!("csrf={}&{body}", page_csrf(&list))).await;
        assert!(to(&r).starts_with("/ui/login"), "{path}");
        // A key that is not one: 400.
        if path != RECEIPTS {
            let r = post(&c, path, peer(166, 5), &cookie, &format!("csrf={}&p=89000&once=x&confirm=1", page_csrf(&list))).await;
            assert_eq!(r.status, 400, "{path}");
        }
    }
    assert!(calls(&c).is_empty());
}
