import {
  Gauge,
  Wifi,
  MessageSquare,
  MessageSquareShare,
  Signal,
  Radio,
  ShieldCheck,
  Clock,
  BatteryCharging,
  Terminal,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

export interface FeatureDetail {
  icon: LucideIcon;
  title: string;
  category: string;
  description: string;
  howItWorks: string;
}

export const featureDetails: FeatureDetail[] = [
  {
    icon: Gauge,
    title: "速度测试",
    category: "网络",
    description:
      "直接从路由器测量互联网吞吐量。",
    howItWorks:
      "代理从 U60 本身连接到 speedtest.net 服务器。测量延迟、下载和上传 — 为您提供真正的 WAN 速度，而无需 WiFi 开销。结果实时传输到应用程序。",
  },
  {
    icon: Wifi,
    title: "局域网速度测试",
    category: "网络",
    description:
      "测试WiFi手机与路由器之间的链路质量。",
    howItWorks:
      "您的电话通过 WiFi 向代理的 /lan-test/* 端点发送 HTTP 请求。测量 ping（空 200）、下载（代理流 ping 为零）和上传（电话流、代理计数字节）。测试您的 WiFi 链路，而不是 WAN。",
  },
  {
    icon: MessageSquare,
    title: "短信管理",
    category: "电话",
    description:
      "直接从配套应用程序读取、发送和删除短信。",
    howItWorks:
      "消息通过设备上的 ubus 命令读取和发送。对话按联系号码分组。您可以发送到任何号码、删除单条消息或清除整个线程。",
  },
  {
    icon: MessageSquareShare,
    title: "短信转发",
    category: "电话",
    description:
      "自动转发传入短信至 Telegram、webhooks 等。",
    howItWorks:
      "代理侦听 ubus new-SMS 事件 — 无轮询延迟。传入消息将根据规则（发件人、关键字或两者）进行匹配。匹配将转发到 Telegram、webhooks、ntfy、Discord、Slack 或其他电话号码，并在失败时自动重试。",
  },
  {
    icon: Signal,
    title: "信号监测",
    category: "调制解调器",
    description:
      "实时 RSRP、SINR 和 RSRQ 读数以及电池信息。",
    howItWorks:
      "实时从调制解调器读取信号指标。显示服务小区信息（NR 或 LTE）、EARFCN/ARFCN、PCI 和频段号。邻近小区扫描可用于查找您所在区域更好的信号塔。",
  },
  {
    icon: Radio,
    title: "频段锁定",
    category: "调制解调器",
    description:
      "将调制解调器锁定到特定的 NR 或 LTE 频段和小区。",
    howItWorks:
      "强制调制解调器使用特定的 NR 或 LTE 频段。通过 PCI + 频率进行单元锁定。智能塔连接 (STC)，用于自动选择最佳塔。对于避免拥挤的频段或强制建设更好的塔很有用。",
  },
  {
    icon: ShieldCheck,
    title: "DNS-over-HTTPS",
    category: "隐私",
    description:
      "适用于所有 LAN 客户端的内置加密 DNS 代理。",
    howItWorks:
      "代理运行 DoH 代理 — LAN 客户端可以使用 192.168.0.1:9090 作为其 DNS 解析器。查询通过 HTTPS 转发到 Cloudflare 或 Google。响应缓存可加快重复查找速度并减少延迟。",
  },
  {
    icon: Clock,
    title: "调度程序",
    category: "自动化任务",
    description:
      "将任何 API 操作安排为一次性或重复性作业。",
    howItWorks:
      "创建调用任何代理 API 端点的 cron 样式作业。示例：每天凌晨 3 点重新启动、按计划切换飞行模式、每小时运行一次速度测试。作业将持久保存到磁盘并在重新启动后继续存在。",
  },
  {
    icon: BatteryCharging,
    title: "收费政策",
    category: "设备",
    description:
      "iPhone-style charge limiter to protect battery health.",
    howItWorks:
      "设置 80% 的充电上限 - 代理监听 ubus 充电器事件并对状态变化立即做出反应。在极限处暂停充电并在 75% 处恢复（5% 滞后）。使电池保持在最健康的充电范围，以便长期使用。",
  },
  {
    icon: Terminal,
    title: "AT命令",
    category: "调制解调器",
    description:
      "用于高级调制解调器诊断的直接 AT 命令接口。",
    howItWorks:
      "直接发送AT命令到/dev/at_mdm0。查询 IMEI、调制解调器固件版本、详细信号信息，或发送自定义 AT 字符串。用于深度调制解调器诊断和配置的高级用户工具。",
  },
];
