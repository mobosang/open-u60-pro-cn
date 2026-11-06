//! Router › Firewall (with New Port Forward), Telemetry Blocker, VPN
//! Passthrough and QoS through the whole router: the pages, their
//! questions, the posts and where they lead. Page-load reads come from
//! `pages::device`'s `gets`, the changes go to `pages::device`. Every value
//! is a made-up fixture (documentation addresses, a locally administered
//! MAC).

use serde_json::json;

use super::*;
use crate::web::pages::device;

const FW: &str = "/ui/router/firewall";
const NEW: &str = "/ui/router/firewall/port-forward/new";
const ADD: &str = "/ui/router/firewall/port-forward/add";
const DEL: &str = "/ui/router/firewall/port-forward/delete";
const DMZ: &str = "/ui/router/firewall/dmz";
const TM: &str = "/ui/router/telemetry";
const VPN: &str = "/ui/router/vpn";
const QOS: &str = "/ui/router/qos";
const MAC: &str = "02:00:5e:10:3a:7f";

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

fn body(csrf: &str, once: &str, fields: &[(&str, &str)]) -> String {
    let mut out = format!("csrf={csrf}&once={once}");
    for (k, v) in fields {
        out.push_str(&format!("&{k}={}", enc(v)));
    }
    out
}

fn has(html: &str, wants: &[&str]) {
    for want in wants {
        assert!(html.contains(want), "{want} in {html}");
    }
}

fn rules() -> Value {
    json!({"rule_list": [
        {"id": "0", "name": "Web", "protocol": "tcp", "wan_port": "8080", "lan_ip": "192.168.0.50", "lan_port": "80", "enabled": "1"},
        {"section_id": "cfg0a1b2c", "comment": "", "proto": "udp", "src_dport": "3074", "dest_ip": "192.168.0.60", "dest_port": "3074"},
    ]})
}

/// The router: firewall on at Medium, NAT on, DMZ off, port forwarding on
/// with two rules, one MAC filter, UPnP on, the LAN 192.168.0.0/24.
fn router(fw: Value, pf: Value) -> device::Device {
    let filters = json!({"rule_list": [{"protocol": "tcp", "src_mac": MAC, "src_ip": "192.168.0.70", "src_port": "", "dest_ip": "203.0.113.9", "dest_port": "443", "enabled": "1"}]});
    device::Device {
        gets: vec![
            ("/api/router/firewall", Ok(fw)),
            ("/api/router/firewall/port-forward", Ok(pf)),
            ("/api/router/firewall/filter-rules", Ok(filters)),
            ("/api/router/firewall/upnp", Ok(json!({"upnp_switch": "1"}))),
            ("/api/router/lan", Ok(json!({"lan_ipaddr": "192.168.0.1", "lan_netmask": "255.255.255.0"}))),
        ],
        ..Default::default()
    }
}

fn firewall() -> Value {
    json!({"firewall_switch": "1", "nat_switch": "1", "dmz_enabled": "0", "dmz_ip": "", "firewall_level": "medium", "wan_ping_filter": "0", "port_forward_switch": "1"})
}

/// Firewall: the app's sections and rows, MACs masked; each switch and the
/// level are links to their questions; a confirmed post sends the app's
/// and the firmware's names once and shows the app's line; a remote
/// browser is warned where a change can lock it out.
#[tokio::test]
async fn firewall_page_switches_and_level() {
    let (c, cookie, _) = signed_in(210).await;
    let _dev = device::set(&c.app, router(firewall(), rules()));
    let r = get_with(&c, FW, peer(210, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    let g = html.split("&amp;g=").nth(1).unwrap()[..16].to_owned();
    has(&html, &[
        "<h1>防火墙</h1>", "href=\"/ui/router\">路由器</a>", "<h2>防火墙</h2>", "<span>状态</span><span class=\"c-green\">已启用</span>",
        "<a class=\"it bt\" href=\"/ui/router/firewall/switch?on=0\"><span>禁用防火墙</span></a>",
        "<span aria-current=\"true\">中等</span>", "<a href=\"/ui/router/firewall/level?level=high\">高</a>",
        "<span>万平</span><span class=\"v\">允许</span>", "<h2>网络地址转换/UPnP</h2>",
        "<a class=\"sw\" href=\"/ui/router/firewall/nat?on=0\" role=\"switch\" aria-checked=\"true\" aria-label=\"NAT\">",
        "<a class=\"sw\" href=\"/ui/router/firewall/upnp?on=0\" role=\"switch\" aria-checked=\"true\" aria-label=\"通用即插即用\">",
        "<form class=\"rf dmz\" method=\"post\" action=\"/ui/router/firewall/dmz\" autocomplete=\"off\">",
        "<input class=\"sr\" id=\"dmz\" type=\"checkbox\" role=\"switch\" name=\"dmz\" value=\"1\">", "DMZ主机IP", ">申请DMZ</button>",
        "<h2>端口转发</h2>", "<a class=\"it bt\" href=\"/ui/router/firewall/port-forward/new\">", "添加规则",
        "<b class=\"nm\">Web</b><small class=\"c-green\">已启用</small>", "TCP WAN:8080 \u{2192} 192.168.0.50:80",
        "<b class=\"nm\">规则</b><small>未启用</small>", "UDP WAN:3074 \u{2192} 192.168.0.60:3074",
        "href=\"/ui/router/firewall/port-forward/delete?id=cfg0a1b2c&amp;g=", "aria-label=\"删除 Web\"",
        "<h2>MAC/IP/端口过滤器</h2>", "MAC: \u{2022}\u{2022}:\u{2022}\u{2022}:\u{2022}\u{2022}:\u{2022}\u{2022}:3a:7f",
        "192.168.0.70: \u{2192} 203.0.113.9:443",
    ]);
    assert!(!html.contains("02:00:5e") && !html.contains("Coming to the web UI"), "{html}");
    assert!(g.bytes().all(|b| b.is_ascii_hexdigit()), "{g}");
    let csrf = r.input("csrf");
    // Turn off: the question, then the confirmed post (both names, once).
    let q = get_with(&c, "/ui/router/firewall/switch?on=0", peer(210, 2), &cookie).await;
    q.assert_secure_html(200);
    has(&q.text(), &[
        "<h2 class=\"q\">关闭the firewall?</h2>", "路由器上的设备失去防火墙’ 的互联网保护。",
        "路由器的其他服务（例如 2222 端口的 SSH 和固件自带网页）可能变得可从移动网络访问。此管理网页及 Agent API 仍会拒绝来自移动网络的连接。",
        "<input type=\"hidden\" name=\"on\" value=\"0\">", ">禁用防火墙</button>", "href=\"/ui/router/firewall\">取消</a>",
    ]);
    assert!(!q.text().contains("connected remotely") && !q.text().contains("路由器本身及此管理网页可能变得可从移动网络访问，届时仅由 Agent 密码保护。"), "{}", q.text());
    let once = q.input("once");
    let fields = body(&csrf, &once, &[("on", "0")]);
    // Origin and CSRF first; a value no page sends; no confirm: the question.
    let bad = form("/ui/router/firewall/switch", peer(210, 2), &cookie, &fields).header("origin", "http://evil.example").body(Body::from(fields.clone())).unwrap();
    assert_eq!(send(&c, bad).await.status, 403);
    assert_eq!(post(&c, "/ui/router/firewall/switch", peer(210, 2), &cookie, &fields.replacen(&csrf, "0", 1)).await.status, 403);
    assert_eq!(post(&c, "/ui/router/firewall/switch", peer(210, 2), &cookie, &body(&csrf, &once, &[("on", "2"), ("confirm", "1")])).await.status, 400);
    assert_eq!(to(&post(&c, "/ui/router/firewall/switch", peer(210, 2), &cookie, &fields).await), "/ui/router/firewall/switch?on=0");
    assert!(calls(&c).is_empty());
    let confirmed = format!("{fields}&confirm=1");
    let done = to(&post(&c, "/ui/router/firewall/switch", peer(210, 2), &cookie, &confirmed).await);
    assert_eq!(done, format!("{FW}?r={once}"));
    assert_eq!(calls(&c), vec![("/api/router/firewall/switch".to_owned(), json!({"firewall_switch": "0", "enable": 0}))]);
    has(&get_with(&c, &done, peer(210, 2), &cookie).await.text(), &["<p class=\"msg ok\" role=\"status\">防火墙disabled</p>"]);
    // A replay sends nothing and lands on the same line.
    assert_eq!(to(&post(&c, "/ui/router/firewall/switch", peer(210, 2), &cookie, &confirmed).await), done);
    assert_eq!(calls(&c).len(), 1);
    // Turning it on or raising the level can lock a remote browser out.
    let q = get_with(&c, "/ui/router/firewall/switch?on=1", tailnet(41001), &cookie).await.text();
    has(&q, &["是否开启防火墙？", "防火墙将阻止未被允许的连接，可能导致无法访问路由器或其连接的设备。", "You are connected remotely", ">启用防火墙</button>"]);
    let q = get_with(&c, "/ui/router/firewall/level?level=high", tailnet(41002), &cookie).await.text();
    has(&q, &["<h2 class=\"q\">将防火墙级别设置为High？</h2>", "blocks more connections at High", "You are connected remotely", ">设置级别</button>"]);
    let q = get_with(&c, "/ui/router/firewall/level?level=low", tailnet(41003), &cookie).await.text();
    has(&q, &["将防火墙级别设置为Low？", "lets more connections from the internet through at Low", "路由器的其他服务（例如 2222 端口的 SSH 和固件自带网页）可能变得可从移动网络访问。此管理网页及 Agent API 仍会拒绝来自移动网络的连接。", "路由器的其他服务（例如 2222 端口的 SSH 和固件自带网页）可能变得可从移动网络访问。此管理网页及 Agent API 仍会拒绝来自移动网络的连接。"]);
    assert!(!q.contains("connected remotely") && !q.contains("防火墙将阻止未被允许的连接，可能导致无法访问路由器或其连接的设备。"), "{q}");
    let q = get_with(&c, "/ui/router/firewall/level?level=medium", peer(210, 2), &cookie).await.text();
    assert!(q.contains("防火墙级别已经是Medium") && !q.contains("name=\"confirm\""), "{q}");
    assert_eq!(get_with(&c, "/ui/router/firewall/level?level=max", peer(210, 2), &cookie).await.status, 400);
    // The level, NAT, UPnP and port forwarding: the app's and the firmware's names.
    for (path, field, value, sent, line) in [
        ("/ui/router/firewall/level", "level", "high", json!({"firewall_level": "high", "level": "high"}), "防火墙级别设置为high"),
        ("/ui/router/firewall/nat", "on", "0", json!({"nat_switch": "0", "enable": 0}), "NAT disabled"),
        ("/ui/router/firewall/upnp", "on", "1", json!({"upnp_switch": "1", "enable_upnp": 1}), "UPnP enabled"),
        ("/ui/router/firewall/port-forward/switch", "on", "0", json!({"port_forward_switch": "0", "portforward_enable": 0}), "端口转发disabled"),
    ] {
        let q = get_with(&c, &format!("{path}?{field}={value}"), peer(210, 2), &cookie).await;
        let once = q.input("once");
        let done = to(&post(&c, path, peer(210, 2), &cookie, &format!("{}&confirm=1", body(&csrf, &once, &[(field, value)]))).await);
        let api = format!("/api{}", path.trim_start_matches("/ui"));
        assert_eq!(calls(&c).last().unwrap(), &(api, sent), "{path}");
        has(&get_with(&c, &done, peer(210, 2), &cookie).await.text(), &[line]);
    }
    // A failed read shows only the error.
    let _dev = device::set(&c.app, device::Device { gets: vec![("/api/router/firewall", Err("ubus call failed: Method not found".into()))], ..Default::default() });
    let html = get_with(&c, FW, peer(210, 2), &cookie).await.text();
    assert!(html.contains("加载防火墙失败： ubus call failed: 方法 not found") && !html.contains("<form class=\"rf dmz\""), "{html}");
}

/// DMZ and New Port Forward: checked against the router's network and
/// rules (refused: the form again as typed), asked, then sent once with
/// both names; a device failure shows the form as typed.
#[tokio::test]
async fn dmz_and_new_port_forward() {
    let (c, cookie, _) = signed_in(211).await;
    let _dev = device::set(&c.app, router(firewall(), rules()));
    let page = get_with(&c, FW, peer(211, 2), &cookie).await;
    let (csrf, once) = (page.input("csrf"), page.input("once"));
    // The router's own address, then nothing to apply.
    let r = post(&c, DMZ, peer(211, 2), &cookie, &body(&csrf, &once, &[("dmz", "1"), ("ip", "192.168.0.1")])).await;
    r.assert_secure_html(400);
    has(&r.text(), &["这是路由器’自己的地址；使用设备’的地址。", "name=\"ip\" value=\"192.168.0.1\"", "value=\"1\" checked"]);
    let r = post(&c, DMZ, peer(211, 2), &cookie, &body(&csrf, &once, &[("ip", "")])).await;
    let q = get_with(&c, &to(&r), peer(211, 2), &cookie).await.text();
    assert!(q.contains("没有什么可申请的") && q.contains("DMZ is already off.") && !q.contains("name=\"confirm\""), "{q}");
    let r = post(&c, DMZ, peer(211, 2), &cookie, &body(&csrf, &once, &[("dmz", "1"), ("ip", "192.168.0.77")])).await;
    assert_eq!(to(&r), format!("{DMZ}?t={once}"));
    let q = get_with(&c, &to(&r), peer(211, 2), &cookie).await;
    has(&q.text(), &["<h2 class=\"q\">将192.168.0.77设为DMZ主机？</h2>", "every port of that device can be reached from the internet", ">申请DMZ</button>"]);
    let confirm = format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"));
    let done = to(&post(&c, DMZ, peer(211, 2), &cookie, &confirm).await);
    assert_eq!(calls(&c), vec![("/api/router/firewall/dmz".to_owned(), json!({"dmz_enabled": "1", "dmz_ip": "192.168.0.77", "dmz_enable": 1}))]);
    has(&get_with(&c, &done, peer(211, 2), &cookie).await.text(), &["DMZ 设置更新"]);
    // New Port Forward: the app's sheet.
    let f = get_with(&c, NEW, peer(211, 3), &cookie).await;
    f.assert_secure_html(200);
    has(&f.text(), &[
        "<h1>新建端口转发</h1>", "<a class=\"nl\" href=\"/ui/router/firewall\">取消</a>", "<h2>规则</h2>", "placeholder=\"名称\"",
        "<option value=\"tcp+udp\">TCP + UDP</option>", "<h2>WAN</h2>", "placeholder=\"广域网端口\"", "<h2>LAN</h2>", "placeholder=\"LAN IP\"",
        "placeholder=\"局域网接口\"", ">添加规则</button>",
    ]);
    let once = f.input("once");
    let typed = |name: &'static str, proto: &'static str, wan: &'static str, ip: &'static str, lan: &'static str| {
        body(&csrf, &once, &[("pfname", name), ("proto", proto), ("wan", wan), ("ip", ip), ("lan", lan)])
    };
    let r = post(&c, ADD, peer(211, 3), &cookie, &typed("Cam<1>", "tcp", "", "10.9.9.9", "70000")).await;
    r.assert_secure_html(400);
    has(&r.text(), &[
        "最多使用 32 个字母、数字、空格、点、连字符和下划线。", "输入 1 到 65535 之间的端口。",
        "使用从 192.168.0.1 到 192 的地址。168.0.254.", "value=\"Cam&lt;1&gt;\"",
    ]);
    assert_eq!(post(&c, ADD, peer(211, 3), &cookie, &typed("Cam", "icmp", "9000", "192.168.0.80", "554")).await.status, 400);
    // A WAN port forwarded for the same protocol (UDP 8080 is free).
    let r = post(&c, ADD, peer(211, 3), &cookie, &typed("Cam", "tcp+udp", "8080", "192.168.0.80", "554")).await;
    assert!(r.text().contains("WAN口8080已转发。"), "{}", r.text());
    let r = post(&c, ADD, peer(211, 3), &cookie, &typed("Cam", "udp", "8080", "192.168.0.80", "554")).await;
    let q = get_with(&c, &to(&r), peer(211, 3), &cookie).await;
    has(&q.text(), &["<h2 class=\"q\">添加“Cam”？</h2>", "UDP WAN port 8080 goes to 192.168.0.80:554", ">添加规则</button>"]);
    // A device failure: the form as typed, with the app's "Failed: …".
    let _fail = device::set(&c.app, device::Device {
        replies: vec![("/api/router/firewall/port-forward", (503, json!({"ok": false, "error": "ubus call zwrt_router.api router_set_portforward failed: Invalid argument"})))],
        ..router(firewall(), rules())
    });
    let confirm = format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"));
    let back = to(&post(&c, ADD, peer(211, 3), &cookie, &confirm).await);
    assert!(back.starts_with(&format!("{NEW}?t={once}&r=")), "{back}");
    has(&get_with(&c, &back, peer(211, 3), &cookie).await.text(), &[
        "失败：ubus call zwrt_router.api router_set_portforward failed: Invalid argument", "name=\"pfname\" value=\"Cam\"", "name=\"wan\" value=\"8080\"",
    ]);
    assert_eq!(calls(&c).len(), 1);
    // Sent once with both names.
    let _ok = device::set(&c.app, router(firewall(), rules()));
    let f = get_with(&c, NEW, peer(211, 4), &cookie).await;
    let once = f.input("once");
    let r = post(&c, ADD, peer(211, 4), &cookie, &body(&csrf, &once, &[("pfname", ""), ("proto", "tcp"), ("wan", "9000"), ("ip", "192.168.0.80"), ("lan", "554")])).await;
    let q = get_with(&c, &to(&r), peer(211, 4), &cookie).await;
    assert!(q.text().contains("添加此端口转发？"), "{}", q.text());
    let confirm = format!("csrf={csrf}&once={once}&d={}&confirm=1", q.input("d"));
    let done = to(&post(&c, ADD, peer(211, 4), &cookie, &confirm).await);
    assert_eq!(done, format!("{FW}?r={once}"));
    assert_eq!(calls(&c), vec![(
        "/api/router/firewall/port-forward".to_owned(),
        json!({"action": "add", "name": "", "protocol": "tcp", "wan_port": "9000", "lan_ip": "192.168.0.80", "lan_port": "554", "enabled": 1,
               "comment": "", "proto": "tcp", "src_dport": "9000", "dest_ip": "192.168.0.80"}),
    )]);
    has(&get_with(&c, &done, peer(211, 4), &cookie).await.text(), &["添加端口转发规则"]);
    assert_eq!(to(&post(&c, ADD, peer(211, 4), &cookie, &confirm).await), done);
    assert_eq!(calls(&c).len(), 1);
}

/// A port forward's delete names the list it was chosen from: once the
/// list has changed, the question offers nothing and a post sends nothing
/// (the app's "out of date" line).
#[tokio::test]
async fn port_forward_delete_names_its_list() {
    let (c, cookie, _) = signed_in(212).await;
    let _dev = device::set(&c.app, router(firewall(), rules()));
    let html = get_with(&c, FW, peer(212, 2), &cookie).await.text();
    let at = html.find("href=\"/ui/router/firewall/port-forward/delete?id=0&amp;g=").unwrap();
    let href = html[at + 6..at + 6 + html[at + 6..].find('"').unwrap()].replace("&amp;", "&");
    let q = get_with(&c, &href, peer(212, 2), &cookie).await;
    q.assert_secure_html(200);
    has(&q.text(), &["<h2 class=\"q\">删除“Web”？</h2>", "TCP WAN:8080 \u{2192} 192.168.0.50:80 stops", ">删除</button>"]);
    let (csrf, once) = (q.input("csrf"), q.input("once"));
    let g = q.input("g");
    // Another rule appears: the question and the post refuse.
    let mut more = rules();
    more["rule_list"].as_array_mut().unwrap().insert(0, json!({"id": "9", "name": "New", "protocol": "tcp", "wan_port": "22", "lan_ip": "192.168.0.9", "lan_port": "22"}));
    let _changed = device::set(&c.app, router(firewall(), more));
    let stale = get_with(&c, &href, peer(212, 2), &cookie).await.text();
    assert!(stale.contains("规则列表变更") && !stale.contains("name=\"confirm\""), "{stale}");
    let confirm = body(&csrf, &once, &[("id", "0"), ("g", &g), ("confirm", "1")]);
    let line = to(&post(&c, DEL, peer(212, 2), &cookie, &confirm).await);
    has(&get_with(&c, &line, peer(212, 2), &cookie).await.text(), &["<p class=\"msg err\" role=\"alert\">规则列表已过期，现已重新加载，请重试。</p>"]);
    assert!(calls(&c).is_empty());
    assert_eq!(get_with(&c, &format!("{DEL}?id=0"), peer(212, 2), &cookie).await.status, 400);
    assert_eq!(get_with(&c, &format!("{DEL}?id=a%20b&g={g}"), peer(212, 2), &cookie).await.status, 400);
    // The same list: deleted once.
    let _same = device::set(&c.app, router(firewall(), rules()));
    let once = get_with(&c, &href, peer(212, 2), &cookie).await.input("once");
    let confirm = body(&csrf, &once, &[("id", "0"), ("g", &g), ("confirm", "1")]);
    let done = to(&post(&c, DEL, peer(212, 2), &cookie, &confirm).await);
    assert_eq!(calls(&c), vec![("/api/router/firewall/port-forward".to_owned(), json!({"action": "delete", "id": "0", "section_id": ["0"]}))]);
    has(&get_with(&c, &done, peer(212, 2), &cookie).await.text(), &["端口转发规则已删除"]);
    assert_eq!(to(&post(&c, DEL, peer(212, 2), &cookie, &confirm).await), done);
    assert_eq!(calls(&c).len(), 1);
}

fn filter(list: Value) -> device::Device {
    device::Device { gets: vec![("/api/router/domain-filter", Ok(list))], ..Default::default() }
}

fn blocked() -> Value {
    json!({"enable": "1", "rule_list": [
        {"id": "1", "domain": "dclient.ztems.com", "enabled": "1"},
        {"id": "2", "domain": "ads.example.com", "enabled": "0"},
    ]})
}

/// Telemetry Blocker: the switch, Add and Block All post from the page
/// (once each); Add checks the name and skips one already blocked; Block
/// All adds only the missing known domains; a removal is asked and names
/// its list.
#[tokio::test]
async fn telemetry_blocker() {
    let (c, cookie, _) = signed_in(213).await;
    let _dev = device::set(&c.app, filter(blocked()));
    let r = get_with(&c, TM, peer(213, 2), &cookie).await;
    r.assert_secure_html(200);
    let html = r.text();
    has(&html, &[
        "<h1>遥测拦截</h1>", "<h2>域过滤器</h2>", "<span class=\"c-green\">已启用</span>",
        "<input type=\"hidden\" name=\"on\" value=\"0\"><button class=\"it bt\" type=\"submit\"><span>禁用过滤器</span></button>",
        "<h2>快速行动</h2>", "<span>阻止所有 ZTE 遥测</span>",
        "<span class=\"dom\">dclient.ztems.com</span><span class=\"c-green\">", "<span class=\"sr\">被封锁</span>",
        "<span class=\"dom\">update.ztems.com</span><span class=\"v\">", "<h2>添加自定义域</h2>", "placeholder=\"要阻止的域\"",
        "<h2>被阻止的域</h2>", "<span class=\"dom\">ads.example.com</span><small>未启用</small>", "aria-label=\"删除ads.example.com\"",
    ]);
    assert_eq!(html.matches("Not blocked").count(), 4);
    let (csrf, once) = (r.input("csrf"), r.input("once"));
    // The switch: no question, once.
    let off = body(&csrf, &once, &[("on", "0")]);
    let done = to(&post(&c, "/ui/router/telemetry/switch", peer(213, 2), &cookie, &off).await);
    assert_eq!(done, format!("{TM}?r={once}"));
    assert_eq!(calls(&c), vec![("/api/router/domain-filter".to_owned(), json!({"enable": 0}))]);
    has(&get_with(&c, &done, peer(213, 2), &cookie).await.text(), &["域过滤器disabled"]);
    assert_eq!(to(&post(&c, "/ui/router/telemetry/switch", peer(213, 2), &cookie, &off).await), done);
    assert_eq!(calls(&c).len(), 1);
    // Add: refused as typed; one already blocked (any case) is not sent again.
    let once = get_with(&c, TM, peer(213, 2), &cookie).await.input("once");
    let r = post(&c, "/ui/router/telemetry/add", peer(213, 2), &cookie, &body(&csrf, &once, &[("domain", "not a domain")])).await;
    r.assert_secure_html(400);
    has(&r.text(), &["输入域名，例如 ads.example.com：字母、数字、连字符和点。", "value=\"not a domain\"", "aria-invalid=\"true\"", &format!("name=\"once\" value=\"{once}\"")]);
    let r = post(&c, "/ui/router/telemetry/add", peer(213, 2), &cookie, &body(&csrf, &once, &[("domain", "  ")])).await;
    assert!(r.text().contains("输入域名<"), "{}", r.text());
    let line = to(&post(&c, "/ui/router/telemetry/add", peer(213, 2), &cookie, &body(&csrf, &once, &[("domain", "Ads.Example.com")])).await);
    has(&get_with(&c, &line, peer(213, 2), &cookie).await.text(), &["<p class=\"msg info\" role=\"status\">ads.example.com is already blocked</p>"]);
    assert_eq!(calls(&c).len(), 1);
    let done = to(&post(&c, "/ui/router/telemetry/add", peer(213, 2), &cookie, &body(&csrf, &once, &[("domain", "tracker.example.net")])).await);
    assert_eq!(calls(&c)[1], ("/api/router/domain-filter".to_owned(), json!({"action": "add", "domain": "tracker.example.net", "enabled": "1", "fqdn": "tracker.example.net"})));
    has(&get_with(&c, &done, peer(213, 2), &cookie).await.text(), &["新增tracker.example.net"]);
    // Block All: the four missing ones.
    let once = get_with(&c, TM, peer(213, 2), &cookie).await.input("once");
    let done = to(&post(&c, "/ui/router/telemetry/block-all", peer(213, 2), &cookie, &body(&csrf, &once, &[])).await);
    let added: Vec<Value> = calls(&c)[2..].iter().map(|(_, b)| b["domain"].clone()).collect();
    assert_eq!(added, ["dconfig.ztems.com", "iot.ztems.com", "mcs-cloud.ztems.com", "update.ztems.com"]);
    has(&get_with(&c, &done, peer(213, 2), &cookie).await.text(), &["屏蔽4遥测域s"]);
    // One add refused: the others still sent, and the line says so.
    let failing = device::Device {
        replies: vec![("/api/router/domain-filter#2", (503, json!({"ok": false, "error": "ubus call zwrt_router.api router_set_domain_filter failed: Invalid argument"})))],
        ..filter(blocked())
    };
    let _failing = device::set(&c.app, failing);
    let once = get_with(&c, TM, peer(213, 2), &cookie).await.input("once");
    let line = to(&post(&c, "/ui/router/telemetry/block-all", peer(213, 2), &cookie, &body(&csrf, &once, &[])).await);
    assert_eq!(calls(&c).len(), 4);
    has(&get_with(&c, &line, peer(213, 2), &cookie).await.text(), &[
        "<p class=\"msg info\" role=\"status\">屏蔽3遥测域s; 1 could not be added. Failed: ubus call zwrt_router.api router_set_domain_filter failed: Invalid argument</p>",
    ]);
    let _dev = device::set(&c.app, filter(blocked()));
    // A removal: asked, by its list.
    let html = get_with(&c, TM, peer(213, 2), &cookie).await.text();
    let at = html.find("href=\"/ui/router/telemetry/delete?id=2&amp;g=").unwrap();
    let href = html[at + 6..at + 6 + html[at + 6..].find('"').unwrap()].replace("&amp;", "&");
    let q = get_with(&c, &href, peer(213, 2), &cookie).await;
    has(&q.text(), &["<h2 class=\"q\">删除ads.example.com？</h2>", "路由器上的设备可以再次到达ads.example.com。", ">移除</button>"]);
    let confirm = body(&csrf, &q.input("once"), &[("id", "2"), ("g", &q.input("g")), ("confirm", "1")]);
    let done = to(&post(&c, "/ui/router/telemetry/delete", peer(213, 2), &cookie, &confirm).await);
    assert_eq!(calls(&c).last().unwrap(), &("/api/router/domain-filter".to_owned(), json!({"action": "delete", "id": "2", "section_id": ["2"]})));
    has(&get_with(&c, &done, peer(213, 2), &cookie).await.text(), &["删除ads.example.com"]);
    // Posted from the page: no question page; a failed read shows only the error.
    assert_eq!(to(&get_with(&c, "/ui/router/telemetry/add", peer(213, 2), &cookie).await), TM);
    let _dev = device::set(&c.app, device::Device { gets: vec![("/api/router/domain-filter", Err("ubus call failed: timed out".into()))], ..Default::default() });
    let html = get_with(&c, TM, peer(213, 2), &cookie).await.text();
    assert!(html.contains("加载过滤器失败： ubus call failed: timed out") && !html.contains("阻止所有 ZTE 遥测"), "{html}");
}

/// VPN Passthrough and QoS: the app's switches, posted from the page once
/// with the app's bodies; a failed read shows the error (QoS: the switch
/// "Unavailable", both directions offered; VPN: no form).
#[tokio::test]
async fn vpn_passthrough_and_qos() {
    let (c, cookie, _) = signed_in(214).await;
    let _dev = device::set(&c.app, device::Device {
        gets: vec![
            ("/api/router/vpn", Ok(json!({"l2tp_passthrough": "1", "pptp_passthrough": "0", "ipsec_passthrough": 1}))),
            ("/api/router/qos", Ok(json!({"qos_switch": "0"}))),
        ],
        ..Default::default()
    });
    let r = get_with(&c, VPN, peer(214, 2), &cookie).await;
    r.assert_secure_html(200);
    has(&r.text(), &[
        "<h1>VPN 穿透</h1>", "<h2>VPN 穿透</h2>", "name=\"l2tp\" value=\"1\" checked>", "name=\"pptp\" value=\"1\">",
        "name=\"ipsec\" value=\"1\" checked>", ">应用</button>",
    ]);
    let (csrf, once) = (r.input("csrf"), r.input("once"));
    let apply = body(&csrf, &once, &[("pptp", "1")]);
    let done = to(&post(&c, "/ui/router/vpn/apply", peer(214, 2), &cookie, &apply).await);
    assert_eq!(calls(&c), vec![("/api/router/vpn".to_owned(), json!({"l2tp_passthrough": "0", "pptp_passthrough": "1", "ipsec_passthrough": "0"}))]);
    has(&get_with(&c, &done, peer(214, 2), &cookie).await.text(), &["VPN 直通已更新"]);
    assert_eq!(to(&post(&c, "/ui/router/vpn/apply", peer(214, 2), &cookie, &apply).await), done);
    assert_eq!(calls(&c).len(), 1);
    let r = get_with(&c, QOS, peer(214, 2), &cookie).await;
    r.assert_secure_html(200);
    has(&r.text(), &[
        "<h1>服务质量</h1>", "<h2>服务质量</h2>", "action=\"/ui/router/qos/switch\"", "<input type=\"hidden\" name=\"on\" value=\"1\">",
        "role=\"switch\" aria-checked=\"false\" aria-label=\"启用服务质量\"",
    ]);
    let done = to(&post(&c, "/ui/router/qos/switch", peer(214, 2), &cookie, &body(&csrf, &r.input("once"), &[("on", "1")])).await);
    assert_eq!(calls(&c)[1], ("/api/router/qos".to_owned(), json!({"qos_switch": "1"})));
    has(&get_with(&c, &done, peer(214, 2), &cookie).await.text(), &["服务质量 enabled"]);
    assert_eq!(post(&c, "/ui/router/qos/switch", peer(214, 2), &cookie, &body(&csrf, &r.input("once"), &[("on", "x")])).await.status, 400);
    // The firmware has no such methods: the router's error.
    let missing = || Err("ubus call zwrt_router.api router_get_qos_switch failed: Method not found".to_owned());
    let _dev = device::set(&c.app, device::Device { gets: vec![("/api/router/vpn", missing()), ("/api/router/qos", missing())], ..Default::default() });
    let html = get_with(&c, VPN, peer(214, 2), &cookie).await.text();
    assert!(html.contains("加载VPN失败： ubus call") && !html.contains("<form class=\"rf\""), "{html}");
    let html = get_with(&c, QOS, peer(214, 2), &cookie).await.text();
    has(&html, &["加载QoS失败： ubus call", "<span class=\"v\">不可用</span>", ">开启</button>", ">关闭</button>"]);
}
