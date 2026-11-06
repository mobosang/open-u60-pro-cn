import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "隐私政策 — OpenU60",
  description: "OpenU60 配套应用程序的隐私政策。",
};

export default function PrivacyPage() {
  return (
    <div className="mx-auto max-w-3xl px-6 py-16">
      <h1 className="mb-2 font-display text-2xl font-bold">隐私政策</h1>
      <p className="mb-8 text-sm text-text-dim">最后更新：2026 年 3 月 10 日</p>

      <div className="space-y-6 text-sm leading-relaxed text-text-dim">
        <section>
          <h2 className="mb-2 text-base font-semibold text-text">概述</h2>
          <p>
            OpenU60 是 ZTE U60 Pro (MU5250) 的开源配套应用程序
            5G移动路由器。它仅通过您的本地网络进行通信
            (LAN)，并在路由器上运行 zte-agent REST API。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            访问的数据
          </h2>
          <p className="mb-2">
            该应用程序通过本地路由器访问以下数据
            网络：
          </p>
          <ul className="list-inside list-disc space-y-1 pl-2">
            <li>信号信息（RSRP、SINR、RSRQ、频段、小区ID）</li>
            <li>SMS messages stored on the router&apos;s SIM card</li>
            <li>网络配置（WiFi、APN、DNS、防火墙设置）</li>
            <li>电池状态和热量信息</li>
            <li>流量统计及连接设备列表</li>
          </ul>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            未收集数据
          </h2>
          <ul className="list-inside list-disc space-y-1 pl-2">
            <li>不收集任何个人数据</li>
            <li>无需用户帐户或注册</li>
            <li>没有分析或遥测</li>
            <li>没有崩溃报告</li>
            <li>没有云存储或远程服务器</li>
            <li>没有广告或跟踪</li>
          </ul>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            数据存储
          </h2>
          <p>
            所有数据都保留在您的本地网络上。该应用程序通过 HTTP 进行通信
            zte-agent 服务器在您的路由器的 LAN IP 上运行
            地址（通常为 192.168.0.1:9090）。没有数据传输到任何
            外部服务器、云服务或第三方。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            第三方共享
          </h2>
          <p>
            不与任何第三方共享任何数据。该应用程序没有第三方
            SDK、没有分析服务、也没有广告网络。
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            权限
          </h2>
          <p className="mb-2">该应用程序请求以下权限：</p>
          <ul className="list-inside list-disc space-y-1 pl-2">
            <li>
              <strong>iOS：</strong> 本地网络访问 — 进行通信
              您的路由器通过 WiFi
            </li>
            <li>
              <strong>安卓：</strong> 互联网，ACCESS_WIFI_STATE，
              ACCESS_NETWORK_STATE — 通过 WiFi 与路由器通信
            </li>
          </ul>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">
            开源项目
          </h2>
          <p>
            OpenU60 is fully open-source. You can review the complete source code
            on{" "}
            <a
              href="https://github.com/jesther-ai/open-u60-pro"
              className="text-accent hover:underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub
            </a>
            .
          </p>
        </section>

        <section>
          <h2 className="mb-2 text-base font-semibold text-text">联系方式</h2>
          <p>
            For privacy inquiries, please open an issue on the{" "}
            <a
              href="https://github.com/jesther-ai/open-u60-pro/issues"
              className="text-accent hover:underline"
              target="_blank"
              rel="noopener noreferrer"
            >
              GitHub 存储库
            </a>
            .
          </p>
        </section>

        <section className="border-t border-border pt-6">
          <p className="text-xs">
            OpenU60 不隶属于 ZTE、不由其认可或赞助
            公司。 ZTE 和 U60 Pro 是 ZTE Corporation 的商标。
          </p>
        </section>
      </div>
    </div>
  );
}
