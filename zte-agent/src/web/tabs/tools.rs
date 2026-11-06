//! Tools tab: the app's tool list (`ToolsListView`), in its sections and
//! order, each row with the Android app's subtitle (the iOS rows have
//! none), and its screens: Automations (`pages::automations`), SMS
//! Forwarding (the SMS tab's screens, mounted here too: `tabs::sms`),
//! Speed Test (`pages::speedtest`), Enable ADB (`pages::tools`), USB Mode
//! (`pages::usb`), Band Lock (`pages::bandlock`), AT Terminal
//! (`pages::at`), Device Info and Connected Devices (`pages::tools`), the
//! app's placeholders for the shell tools, and notes for the two tools that
//! stay in the app (LAN Speed Test, Config Decrypt/Encrypt). The questions
//! of Speed Test, Enable ADB and Band Lock, which hand out single-use
//! tokens, are `pages::confirm`.

use crate::events::Topic;
use crate::server::Method;
use crate::web::action::{Action, After, Field, Kind};
use crate::web::kit::{Icon, Tab};
use crate::web::pages::{at, automations, bandlock, confirm, speedtest, tools, usb};
use crate::web::screen::{Bar, Block, Item, Read, Screen, Search};

const fn to(href: &'static str, label: &'static str, icon: Icon, sub: &'static str) -> Item {
    Item { label, icon, href, sub }
}

const SHELL: &str = "需要 ADB USB 连接";

pub static TOOLS: Screen = Screen {
    path: "/ui/tools",
    id: "tools",
    title: "工具",
    tab: Tab::Tools,
    blocks: &[
        Block::Links(
            "自动化任务",
            &[
                to(automations::LIST, "自动化", Icon::ClockArrow, "安排自动化任务"),
                to("/ui/tools/sms-forward", "短信转发", Icon::EnvelopeBranch, "自动转发短信到Telegram、webhooks等。"),
            ],
        ),
        Block::Links(
            "网络工具",
            &[
                to(speedtest::PATH, "速度测试", Icon::Gauge, "测试 WAN 吞吐量"),
                to("/ui/tools/lan-speedtest", "局域网速度测试", Icon::Wifi, "测试 WiFi 到路由器的链路"),
                to(confirm::ADB, "启用 ADB", Icon::Cable, "启用USB调试模式"),
                to(usb::MODE, "USB 模式", Icon::UsbC, "USB 模式和移动电源控制"),
                to(bandlock::PATH, "频段锁定", Icon::Lock, "锁定NR/LTE频段"),
                to(at::PATH, "AT 终端", Icon::Terminal, "发送原始 AT 命令到调制解调器"),
                to(tools::DEVICE_INFO, "设备信息", Icon::Info, "SIM, IMEI, WAN IPs"),
                to(tools::DEVICES, "已连接设备", Icon::Laptop, "查看已连接的客户端和 DHCP 租约"),
            ],
        ),
        Block::Links("配置", &[to("/ui/tools/config", "配置解密/加密", Icon::Doc, "离线ZXHN配置文件工具")]),
        Block::Links(
            "需要 Shell 访问",
            &[
                to("/ui/tools/ttl", "TTL设置", Icon::Number, SHELL),
                to("/ui/tools/ssh", "启用SSH", Icon::Terminal, SHELL),
                to("/ui/tools/explorer", "设备资源管理器", Icon::Folder, SHELL),
            ],
        ),
    ],
    ..Screen::BLANK
};

const BACK: Bar = Bar::Back("/ui/tools", "工具");
const PUSHED: Screen = Screen { tab: Tab::Tools, bar: BACK, ..Screen::BLANK };

/// The app's USB Mode, read at page load (as the app's, which reads it once).
pub static USB_MODE: Screen = Screen {
    path: usb::MODE,
    id: "usbmode",
    title: "USB 模式",
    topics: &[Topic::Usb, Topic::Charger],
    blocks: &[Block::Custom(usb::mode_page)],
    ..PUSHED
};

/// "Fast Charging (Powerbank)" on or off (the app asks nothing); the body
/// is built from the checked value.
pub static USB_POWERBANK: Action = Action {
    path: usb::MODE_POWERBANK,
    screen: &USB_MODE,
    fields: &[Field { name: "state", kind: Kind::Choice(&["0", "1"]) }],
    confirm: &[],
    confirm_at: None,
    unchanged: None,
    reads: &[],
    call: (Method::Put, "/api/usb/powerbank"),
    body: usb::mode_body,
    after: After::Flash(usb::switched),
    again: None,
    once: false,
};

// --- Speed Test, Enable ADB, Band Lock, AT Terminal ---

/// Live from `speedtest` (re-read only when the test changes) from the
/// page's watch point; the server list is read at page load (slow lane: it
/// may come from speedtest.net). Its one message line is drawn by the page.
pub static SPEEDTEST: Screen = Screen {
    path: speedtest::PATH,
    id: "speedtest",
    title: "速度测试",
    topics: &[Topic::Speedtest],
    reads: &[Read::Api(speedtest::SERVERS)],
    live: &speedtest::FRAGMENTS,
    blocks: &[Block::Custom(speedtest::page)],
    arg: Some(speedtest::since),
    refresh: Some(speedtest::refresh),
    own_notice: true,
    ..PUSHED
};

/// "Stop Test" (the app asks nothing), back at the chosen server (0: none).
pub static SPEEDTEST_STOP: Action = Action {
    path: speedtest::STOP,
    screen: &SPEEDTEST,
    fields: &[Field { name: "server", kind: Kind::Int(0, u32::MAX as i64) }],
    confirm: &[],
    confirm_at: None,
    unchanged: None,
    reads: &[],
    call: (Method::Post, "/api/speedtest/stop"),
    body: speedtest::stop_body,
    after: After::FlashAt(speedtest::stop_at, speedtest::stopped),
    again: None,
    once: false,
};

/// Static; its result line is `?r=<id>` (`pages::confirm`), drawn above
/// the button as the app's.
pub static ADB: Screen = Screen {
    path: confirm::ADB,
    id: "adb",
    title: "启用 ADB",
    blocks: &[Block::Custom(tools::adb)],
    own_notice: true,
    ..PUSHED
};

/// Nothing is read, as in the app.
pub static BAND_LOCK: Screen = Screen {
    path: bandlock::PATH,
    id: "bandlock",
    title: "频段锁定",
    blocks: &[Block::Custom(bandlock::page)],
    ..PUSHED
};

/// The port is read at page load; the history is the session's.
pub static AT: Screen = Screen {
    path: at::PATH,
    id: "at",
    title: "AT 终端",
    reads: &[Read::Api("/api/at/port")],
    blocks: &[Block::Custom(at::page)],
    ..PUSHED
};

/// "Clear": nothing is sent, the session's history goes.
pub static AT_CLEAR: Action = Action {
    path: at::CLEAR,
    screen: &AT,
    fields: &[],
    confirm: &[],
    confirm_at: None,
    unchanged: None,
    reads: &[],
    call: (Method::Post, at::CLEAR),
    body: |_, _| None,
    after: After::Session(at::clear),
    again: None,
    once: false,
};

// --- Device Info, Connected Devices ---

/// Read at page load (the app reads it as the screen opens), never live.
pub static DEVICE_INFO: Screen = Screen {
    path: tools::DEVICE_INFO,
    id: "deviceinfo",
    title: "设备信息",
    topics: &[Topic::Wan, Topic::Signal],
    reads: &[Read::Fn(tools::info_read)],
    blocks: &[Block::Custom(tools::device_info)],
    ..PUSHED
};

/// Live from `clients`; a search is a snapshot.
pub static DEVICES: Screen = Screen {
    path: tools::DEVICES,
    id: "devices",
    title: "已连接设备",
    topics: &[Topic::Clients],
    live: &tools::FRAGMENTS,
    blocks: &[Block::Custom(tools::devices)],
    search: Search::Still,
    ..PUSHED
};

// --- The app's placeholders and the tools that stay in the app ---

pub static LAN_SPEED: Screen = Screen {
    path: "/ui/tools/lan-speedtest",
    id: "lanspeed",
    title: "局域网速度测试",
    blocks: &[Block::Empty(
        Icon::Wifi,
        "局域网速度测试",
        "测量该设备和路由器之间的 WiFi 链路速度。不使用互联网数据。",
        "LAN 速度测试位于 OpenU60 应用程序中。",
    )],
    ..PUSHED
};

/// The app's title for it is "Config Tool".
pub static CONFIG: Screen = Screen {
    path: "/ui/tools/config",
    id: "configtool",
    title: "配置工具",
    blocks: &[Block::Empty(Icon::Doc, "配置工具", "离线ZXHN配置文件工具。", "配置解密/加密位于 OpenU60 应用程序中。")],
    ..PUSHED
};

pub static TTL: Screen = Screen {
    path: "/ui/tools/ttl",
    id: "ttl",
    title: "TTL设置",
    blocks: &[Block::Empty(Icon::Number, "TTL设置", "通过 iptables 设置 TTL 覆盖。需要外壳访问权限。", "")],
    ..PUSHED
};

pub static SSH: Screen = Screen {
    path: "/ui/tools/ssh",
    id: "ssh",
    title: "启用SSH",
    blocks: &[Block::Empty(Icon::Terminal, "启用SSH", "安装并启动dropbear SSH服务器。需要 ADB USB 连接。", "")],
    ..PUSHED
};

pub static EXPLORER: Screen = Screen {
    path: "/ui/tools/explorer",
    id: "explorer",
    title: "设备资源管理器",
    blocks: &[Block::Empty(Icon::Folder, "设备资源管理器", "浏览文件系统并收集设备信息。需要 ADB USB 连接。", "")],
    ..PUSHED
};

// --- Automations ---

const JOBS: &[Read] = &[Read::Api(automations::JOBS)];

pub static AUTOMATIONS: Screen = Screen {
    path: automations::LIST,
    id: "automations",
    title: "自动化",
    reads: JOBS,
    saved: &automations::SAVED,
    saved_ok: true,
    blocks: &[Block::Custom(automations::list)],
    ..PUSHED
};

/// The delete question reads the job's name, so it is a screen; its form
/// posts to the action of the same path.
pub static AUTO_DELETE: Screen = Screen {
    path: automations::DELETE,
    id: "autodelete",
    title: "确认",
    bar: Bar::Back(automations::LIST, "自动化"),
    reads: JOBS,
    blocks: &[Block::Custom(automations::ask_delete)],
    status: Some(automations::ask_status),
    ..PUSHED
};

const JOB_ID: Field = Field { name: "id", kind: Kind::Id(u32::MAX) };
/// The job's creation time: with the id, the job the page showed (the
/// agent reuses a deleted newest job's id).
const JOB_CREATED: Field = Field { name: "c", kind: Kind::Int(0, i64::MAX) };

/// A job's switch (`{"id","enabled"}`, as the app), for the job the list
/// showed (the jobs read first).
const TOGGLE_JOB: Action = Action {
    path: automations::TOGGLE,
    screen: &AUTOMATIONS,
    fields: &[JOB_ID, JOB_CREATED, Field { name: "enabled", kind: Kind::Choice(&["0", "1"]) }],
    confirm: &[],
    confirm_at: None,
    unchanged: None,
    reads: JOBS,
    call: (Method::Put, "/api/scheduler/jobs/toggle"),
    body: automations::toggle_body,
    after: After::Back(automations::at_job),
    again: None,
    once: false,
};

pub static AUTO_TOGGLE: Action = TOGGLE_JOB;

/// Delete a job (the app's swipe deletes at once; the web asks first).
pub static AUTO_DELETE_JOB: Action = Action {
    path: automations::DELETE,
    fields: &[JOB_ID, JOB_CREATED],
    confirm_at: Some(|v| format!("{}?id={}", automations::DELETE, v.int("id"))),
    call: (Method::Delete, automations::JOBS),
    body: automations::delete_body,
    after: After::Saved("deleted"),
    ..TOGGLE_JOB
};
