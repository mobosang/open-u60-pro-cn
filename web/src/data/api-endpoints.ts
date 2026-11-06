export interface APICategory {
  name: string;
  count: number;
  capabilities: string;
  endpoints?: APIEndpoint[];
}

export interface APIEndpoint {
  method: string;
  path: string;
  description: string;
}

export const apiCategories: APICategory[] = [
  {
    name: "授权",
    count: 1,
    capabilities: "基于令牌的身份验证",
    endpoints: [
      { method: "POST", path: "/api/auth/login", description: "认证并获取会话token" },
    ],
  },
  {
    name: "设备",
    count: 16,
    capabilities:
      "系统信息、电池、CPU、内存、热量、充电器、充电控制、重启、恢复出厂设置、省电、快速启动",
    endpoints: [
      { method: "GET", path: "/api/device", description: "系统信息（sysfs）" },
      { method: "GET", path: "/api/battery", description: "电池状态和百分比" },
      { method: "GET", path: "/api/cpu", description: "CPU使用率和频率" },
      { method: "GET", path: "/api/memory", description: "内存使用统计" },
      { method: "GET", path: "/api/device/battery-info", description: "电池详细信息（ubus）" },
      { method: "GET", path: "/api/device/thermal", description: "温度传感器" },
      { method: "GET", path: "/api/device/charger", description: "充电器状态" },
      { method: "GET", path: "/api/device/system", description: "系统详细信息（ubus）" },
      { method: "POST", path: "/api/device/reboot", description: "重启设备" },
      { method: "POST", path: "/api/device/factory-reset", description: "恢复出厂设置" },
      { method: "GET", path: "/api/device/charge-control", description: "充电上限状态" },
      { method: "PUT", path: "/api/device/charge-control", description: "设置充电限额" },
      { method: "POST", path: "/api/device/power-save", description: "省电模式状态" },
      { method: "PUT", path: "/api/device/power-save", description: "切换省电模式" },
      { method: "GET", path: "/api/device/fast-boot", description: "快速启动状态" },
      { method: "PUT", path: "/api/device/fast-boot", description: "切换快速启动" },
    ],
  },
  {
    name: "网络",
    count: 9,
    capabilities:
      "信号强度（RSRP/SINR/RSRQ）、速度、流量统计、WAN/LAN 状态、rmnet、连接的客户端",
    endpoints: [
      { method: "GET", path: "/api/network/signal", description: "信号强度指标" },
      { method: "GET", path: "/api/network/traffic", description: "流量统计" },
      { method: "GET", path: "/api/network/speed", description: "当前速度" },
      { method: "GET", path: "/api/network/wan", description: "广域网 IPv4 状态" },
      { method: "GET", path: "/api/network/wan6", description: "广域网 IPv6 状态" },
      { method: "GET", path: "/api/network/lan-status", description: "局域网状态" },
      { method: "GET", path: "/api/network/clients", description: "连接的客户端" },
      { method: "GET", path: "/api/network/speeds", description: "速度历史记录" },
      { method: "GET", path: "/api/network/rmnet", description: "rmnet interface info" },
    ],
  },
  {
    name: "调制解调器",
    count: 12,
    capabilities:
      "飞行模式、移动数据切换、网络模式（2G–5G）、运营商扫描、手动注册",
    endpoints: [
      { method: "GET", path: "/api/data-usage", description: "数据使用统计" },
      { method: "GET", path: "/api/modem/status", description: "调制解调器状态" },
      { method: "POST", path: "/api/modem/online", description: "上线/下线" },
      { method: "GET", path: "/api/modem/data", description: "移动数据状态" },
      { method: "PUT", path: "/api/modem/data", description: "切换移动数据" },
      { method: "POST", path: "/api/modem/airplane", description: "切换飞行模式" },
      { method: "PUT", path: "/api/modem/network-mode", description: "设置网络模式（2G-5G）" },
      { method: "POST", path: "/api/modem/scan", description: "开始运营商扫描" },
      { method: "GET", path: "/api/modem/scan/status", description: "运营商扫描状态" },
      { method: "GET", path: "/api/modem/scan/results", description: "运营商扫描结果" },
      { method: "POST", path: "/api/modem/register", description: "手动网络注册" },
      { method: "GET", path: "/api/modem/register/result", description: "报名结果" },
    ],
  },
  {
    name: "SMS",
    count: 14,
    capabilities:
      "列表、发送、删除、标记已读、存储容量、短信转发配置/规则/切换/测试/日志",
    endpoints: [
      { method: "POST", path: "/api/sms/list", description: "列出短信" },
      { method: "GET", path: "/api/sms/capacity", description: "短信存储容量" },
      { method: "POST", path: "/api/sms/send", description: "发送短信" },
      { method: "POST", path: "/api/sms/delete", description: "删除短信" },
      { method: "POST", path: "/api/sms/read", description: "将短信标记为已读" },
      { method: "GET", path: "/api/sms/forward/config", description: "转发配置" },
      { method: "PUT", path: "/api/sms/forward/config", description: "更新转发配置" },
      { method: "POST", path: "/api/sms/forward/rules", description: "创建转发规则" },
      { method: "PUT", path: "/api/sms/forward/rules", description: "更新转发规则" },
      { method: "DELETE", path: "/api/sms/forward/rules", description: "删除转发规则" },
      { method: "PUT", path: "/api/sms/forward/rules/toggle", description: "切换转发规则" },
      { method: "POST", path: "/api/sms/forward/test", description: "测试转发" },
      { method: "GET", path: "/api/sms/forward/log", description: "转发日志" },
      { method: "POST", path: "/api/sms/forward/log/clear", description: "清除转发日志" },
    ],
  },
  {
    name: "SIM",
    count: 7,
    capabilities: "SIM信息、IMEI、PIN管理、PUK解锁",
    endpoints: [
      { method: "GET", path: "/api/sim/info", description: "SIM卡信息" },
      { method: "GET", path: "/api/sim/imei", description: "设备 IMEI" },
      { method: "POST", path: "/api/sim/pin/verify", description: "验证SIM PIN" },
      { method: "POST", path: "/api/sim/pin/change", description: "更改 SIM PIN" },
      { method: "POST", path: "/api/sim/pin/mode", description: "切换PIN锁定模式" },
      { method: "POST", path: "/api/sim/unlock", description: "PUK解锁" },
      { method: "GET", path: "/api/sim/lock-trials", description: "PIN/PUK 剩余尝试次数" },
    ],
  },
  {
    name: "小区/频段",
    count: 19,
    capabilities:
      "NR/LTE band locking, cell locking, neighbor scan, STC, signal quality detection",
    endpoints: [
      { method: "POST", path: "/api/cell/lock/nr", description: "锁定NR小区" },
      { method: "POST", path: "/api/cell/lock/lte", description: "锁定LTE小区" },
      { method: "POST", path: "/api/cell/lock/reset", description: "重置小区锁" },
      { method: "POST", path: "/api/cell/neighbors/scan", description: "扫描邻近小区" },
      { method: "GET", path: "/api/cell/neighbors/nr", description: "NR 邻居小区" },
      { method: "GET", path: "/api/cell/neighbors/lte", description: "LTE 邻居小区" },
      { method: "POST", path: "/api/cell/band/nr", description: "锁定 NR 频段" },
      { method: "POST", path: "/api/cell/band/lte", description: "锁定 LTE 频段" },
      { method: "POST", path: "/api/cell/band/reset", description: "重置带锁" },
      { method: "GET", path: "/api/cell/stc/params", description: "STC参数" },
      { method: "PUT", path: "/api/cell/stc/params", description: "设置STC参数" },
      { method: "GET", path: "/api/cell/stc/status", description: "STC状态" },
      { method: "POST", path: "/api/cell/stc/enable", description: "启用STC" },
      { method: "POST", path: "/api/cell/stc/disable", description: "禁用 STC" },
      { method: "POST", path: "/api/cell/stc/reset", description: "重置STC" },
      { method: "POST", path: "/api/cell/signal-detect/start", description: "启动信号检测" },
      { method: "POST", path: "/api/cell/signal-detect/stop", description: "停止信号检测" },
      { method: "GET", path: "/api/cell/signal-detect/results", description: "信号检测结果" },
      { method: "GET", path: "/api/cell/signal-detect/progress", description: "信号检测进展" },
    ],
  },
  {
    name: "路由器",
    count: 29,
    capabilities:
      "DNS, LAN/DHCP, firewall, NAT, DMZ, UPnP, port forwarding, QoS, domain filter, APN profiles, VPN/ALG",
    endpoints: [
      { method: "GET", path: "/api/router/dns", description: "DNS设置" },
      { method: "PUT", path: "/api/router/dns", description: "更新DNS设置" },
      { method: "GET", path: "/api/router/lan", description: "LAN/DHCP settings" },
      { method: "PUT", path: "/api/router/lan", description: "更新局域网设置" },
      { method: "GET", path: "/api/router/firewall", description: "防火墙状态" },
      { method: "PUT", path: "/api/router/firewall/switch", description: "切换防火墙" },
      { method: "PUT", path: "/api/router/firewall/level", description: "设置防火墙级别" },
      { method: "PUT", path: "/api/router/firewall/nat", description: "配置NAT" },
      { method: "PUT", path: "/api/router/firewall/dmz", description: "配置DMZ" },
      { method: "GET", path: "/api/router/firewall/upnp", description: "UPnP 状态" },
      { method: "PUT", path: "/api/router/firewall/upnp", description: "配置UPnP" },
      { method: "GET", path: "/api/router/firewall/port-forward", description: "端口转发规则" },
      { method: "POST", path: "/api/router/firewall/port-forward", description: "添加端口转发规则" },
      { method: "PUT", path: "/api/router/firewall/port-forward/switch", description: "切换端口转发" },
      { method: "GET", path: "/api/router/firewall/filter-rules", description: "防火墙过滤规则" },
      { method: "GET", path: "/api/router/vpn", description: "VPN直通设置" },
      { method: "PUT", path: "/api/router/vpn", description: "更新VPN直通" },
      { method: "GET", path: "/api/router/qos", description: "服务质量设置" },
      { method: "PUT", path: "/api/router/qos", description: "更新QoS设置" },
      { method: "GET", path: "/api/router/domain-filter", description: "域名过滤设置" },
      { method: "PUT", path: "/api/router/domain-filter", description: "更新域过滤器" },
      { method: "GET", path: "/api/router/apn/mode", description: "APN模式" },
      { method: "PUT", path: "/api/router/apn/mode", description: "设置APN模式" },
      { method: "GET", path: "/api/router/apn/profiles", description: "APN 配置文件" },
      { method: "POST", path: "/api/router/apn/profiles", description: "添加APN简介" },
      { method: "PUT", path: "/api/router/apn/profiles", description: "修改APN配置文件" },
      { method: "GET", path: "/api/router/apn/auto-profiles", description: "自动检测到的 APN 配置文件" },
      { method: "POST", path: "/api/router/apn/profiles/delete", description: "删除APN资料" },
      { method: "POST", path: "/api/router/apn/profiles/activate", description: "激活APN配置文件" },
    ],
  },
  {
    name: "WiFi",
    count: 4,
    capabilities: "状态，SSID/密码/通道/发射功率，访客WiFi",
    endpoints: [
      { method: "GET", path: "/api/wifi/status", description: "WiFi状态及信息" },
      { method: "PUT", path: "/api/wifi/settings", description: "更新WiFi设置" },
      { method: "GET", path: "/api/wifi/guest", description: "访客 WiFi状态" },
      { method: "PUT", path: "/api/wifi/guest", description: "更新访客WiFi" },
    ],
  },
  {
    name: "USB",
    count: 3,
    capabilities: "USB模式切换、充电宝控制",
    endpoints: [
      { method: "GET", path: "/api/usb/status", description: "USB端口状态" },
      { method: "PUT", path: "/api/usb/mode", description: "设置USB模式" },
      { method: "PUT", path: "/api/usb/powerbank", description: "切换移动电源模式" },
    ],
  },
  {
    name: "电话",
    count: 11,
    capabilities:
      "语音通话（拨号/挂断/接听/DTMF/静音）、USSD代码、SIM工具包菜单",
    endpoints: [
      { method: "POST", path: "/api/call/dial", description: "拨打号码" },
      { method: "POST", path: "/api/call/hangup", description: "挂断电话" },
      { method: "POST", path: "/api/call/answer", description: "接听来电" },
      { method: "GET", path: "/api/call/status", description: "通话状态" },
      { method: "POST", path: "/api/call/dtmf", description: "发送 DTMF 音" },
      { method: "POST", path: "/api/call/mute", description: "切换静音" },
      { method: "POST", path: "/api/ussd/send", description: "发送USSD代码" },
      { method: "POST", path: "/api/ussd/respond", description: "响应USSD提示" },
      { method: "POST", path: "/api/ussd/cancel", description: "取消 USSD 会话" },
      { method: "GET", path: "/api/stk/menu", description: "SIM 工具包菜单" },
      { method: "POST", path: "/api/stk/select", description: "选择STK项目（item_id，command_id）" },
      { method: "GET", path: "/api/stk/session", description: "当前 SIM 工具包会话" },
      { method: "POST", path: "/api/stk/respond", description: "响应 SIM 提示（command_id、text 或 yes）" },
      { method: "POST", path: "/api/stk/back", description: "从 SIM 命令返回（command_id）" },
      { method: "POST", path: "/api/stk/cancel", description: "结束待处理的 SIM 命令（command_id）" },
    ],
  },
  {
    name: "速度测试",
    count: 4,
    capabilities: "服务器列表、运行测试、进度跟踪",
    endpoints: [
      { method: "GET", path: "/api/speedtest/servers", description: "可用的速度测试服务器" },
      { method: "POST", path: "/api/speedtest/start", description: "启动速度测试" },
      { method: "GET", path: "/api/speedtest/progress", description: "速度测试进度" },
      { method: "POST", path: "/api/speedtest/stop", description: "停止速度测试" },
    ],
  },
  {
    name: "DoH代理",
    count: 6,
    capabilities: "DNS-over-HTTPS代理，缓存管理",
    endpoints: [
      { method: "GET", path: "/api/doh/status", description: "DoH代理状态" },
      { method: "PUT", path: "/api/doh/config", description: "更新 DoH 配置" },
      { method: "POST", path: "/api/doh/enable", description: "启用 DoH 代理" },
      { method: "POST", path: "/api/doh/disable", description: "禁用 DoH 代理" },
      { method: "GET", path: "/api/doh/cache", description: "DNS 缓存条目" },
      { method: "POST", path: "/api/doh/cache/clear", description: "清除DNS缓存" },
    ],
  },
  {
    name: "局域网测试",
    count: 3,
    capabilities: "WiFi ping，下载/上传吞吐量测量",
    endpoints: [
      { method: "GET", path: "/api/lan/ping", description: "局域网ping测试" },
      { method: "GET", path: "/api/lan/download", description: "下载吞吐量测试" },
      { method: "POST", path: "/api/lan/upload", description: "上传吞吐量测试" },
    ],
  },
  {
    name: "调度程序",
    count: 5,
    capabilities: "任何 API 操作的计划/重复作业",
    endpoints: [
      { method: "GET", path: "/api/scheduler/jobs", description: "列出预定的作业" },
      { method: "POST", path: "/api/scheduler/jobs", description: "创建预定作业" },
      { method: "PUT", path: "/api/scheduler/jobs", description: "更新预定作业" },
      { method: "DELETE", path: "/api/scheduler/jobs", description: "删除预定作业" },
      { method: "PUT", path: "/api/scheduler/jobs/toggle", description: "启用切换作业" },
    ],
  },
];

export const totalEndpoints = apiCategories.reduce(
  (sum, cat) => sum + cat.count,
  0
);
export const totalCategories = apiCategories.length;
