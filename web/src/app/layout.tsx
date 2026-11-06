import type { Metadata } from "next";
import { Analytics } from "@vercel/analytics/next";
import { Navbar } from "@/components/layout/Navbar";
import { Footer } from "@/components/layout/Footer";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "open-u60-pro — 中兴 U60 Pro 中文工具箱",
    template: "%s | open-u60-pro",
  },
  description:
    "中兴 U60 Pro 5G 路由器开源工具箱，提供设备端 Agent、手机配套应用和网页部署工具。",
  keywords: [
    "ZTE U60 Pro",
    "MU5250",
    "5G 路由器",
    "频段锁定",
    "信号监测",
    "OpenWrt",
    "REST API",
    "WebUSB",
  ],
  authors: [{ name: "jesther-ai" }],
  openGraph: {
    title: "open-u60-pro — 中兴 U60 Pro 中文工具箱",
    description:
      "释放 ZTE U60 Pro 5G 移动路由器的全部潜力。",
    url: "https://open-u60-pro.vercel.app",
    siteName: "open-u60-pro",
    locale: "zh_CN",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "open-u60-pro — 中兴 U60 Pro 中文工具箱",
    description:
      "释放 ZTE U60 Pro 5G 移动路由器的全部潜力。",
  },
  robots: { index: true, follow: true },
  metadataBase: new URL("https://open-u60-pro.vercel.app"),
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="zh-CN"
    >
      <head>
        <meta name="theme-color" content="#EAEFEF" />
      </head>
      <body className="grain-overlay">
        <Navbar />
        <main className="relative z-0 overflow-x-hidden">{children}</main>
        <Footer />
        <Analytics />
      </body>
    </html>
  );
}
