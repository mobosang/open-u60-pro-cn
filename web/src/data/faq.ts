export interface FAQItem {
  question: string;
  answer: string;
}

export const faqData: FAQItem[] = [
  {
    question: "这对我的路由器安全吗？",
    answer:
      "代理是一个用户空间进程——它不会修改固件或内核。它使用与 ZTE 自己的应用程序相同的 ubus/AT 接口。破坏性操作（恢复出厂设置、重新启动）需要明确确认。该代理可以随时停止或删除。",
  },
  {
    question: "哪些浏览器支持网页设置？",
    answer:
      "基于 WebUSB 的设置向导需要桌面上的 Chrome 或 Edge。 Safari、Firefox 和移动浏览器不支持 WebUSB。对于非 Chrome 用户，请改用 CLI 设置脚本。",
  },
  {
    question: "这适用于其他 ZTE 路由器吗？",
    answer:
      "目前仅支持 ZTE U60 Pro (MU5250)。其他 ZTE 型号上的 ubus API 结构可能有所不同。欢迎为其他设备做出贡献。",
  },
  {
    question: "如何更新代理？",
    answer:
      "再次运行部署脚本 (deploy.sh) 或使用 Web 设置向导。新的二进制文件将替换旧的二进制文件，并且代理将自动重新启动。",
  },
  {
    question: "我可以使用SSH代替WebUSB吗？",
    answer:
      "是的。安装脚本可以选择在设备上安装 Dropbear SSH。通过 USB 进行初始设置后，后续部署可以通过 WiFi（端口 2222）使用 SSH。",
  },
  {
    question: "如果代理崩溃或者我想删除它怎么办？",
    answer:
      "代理在启动时通过 rc.local 自动重新启动。要删除它：通过 ADB 或 SSH 连接，删除 /data/zte-agent 和 /data/local/tmp/start_zte_agent.sh，然后删除 rc.local 条目。恢复出厂设置也会清除所有内容。",
  },
  {
    question: "代理需要上网吗？",
    answer:
      "否。代理完全在 LAN（端口 9090）上运行。它从不进行出站连接。移动应用程序通过 WiFi 直接连接到代理。",
  },
  {
    question: "移动应用程序怎么样？",
    answer:
      "包括适用于 iOS (SwiftUI) 和 Android (Jetpack Compose) 的本机配套应用程序。它们通过 WiFi 直接连接到代理，无需云服务。 App Store/Play Store 版本即将推出。",
  },
];
