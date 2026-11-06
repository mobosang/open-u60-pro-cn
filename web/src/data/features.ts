import {
  Server,
  Smartphone,
  Globe,
  Signal,
  Radio,
  ShieldCheck,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

export interface Feature {
  icon: LucideIcon;
  title: string;
  description: string;
}

export const features: Feature[] = [
  {
    icon: Server,
    title: "设备端代理",
    description:
      "轻量级 Rust HTTP 服务器 (~0.8 MB RSS) 取代了 ZTE 的 225 MB 守护进程堆栈。 16 个类别的 143 个 REST 端点。",
  },
  {
    icon: Smartphone,
    title: "移动配套应用程序",
    description:
      "原生 iOS (SwiftUI) 和 Android (Compose) 应用程序。仪表板、信号监视器、短信、带锁、设备管理——全部在 WiFi 上。",
  },
  {
    icon: Globe,
    title: "网络引导",
    description:
      "完全通过浏览器部署代理。 WebUSB 供电 — 无需命令行。适用于 Chrome 和 Edge。",
  },
  {
    icon: Signal,
    title: "信号监测",
    description:
      "实时 RSRP、SINR、RSRQ 读数。 NR/LTE小区信息、邻区扫描、信号质量检测。",
  },
  {
    icon: Radio,
    title: "频带锁定",
    description:
      "锁定特定 NR 和 LTE 频段。小区锁定、STC（智能塔连接）和载波聚合控制。",
  },
  {
    icon: ShieldCheck,
    title: "隐私第一",
    description:
      "零遥测。没有回拨到 ZTE 服务器。开源代理仅在您的 LAN 上运行。完全控制您的设备。",
  },
];
