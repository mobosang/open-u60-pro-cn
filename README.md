# 中兴 U60 Pro 中文工具箱

基于 [jesther-ai/open-u60-pro](https://github.com/jesther-ai/open-u60-pro) 的简体中文 fork，适用于 **ZTE U60 Pro / MU5250**。保留上游作者署名和许可证。

设备端 Rust Agent 提供 REST API、实时事件和内置管理网页，配合 Android / iOS 应用查看信号、锁定频段与小区、管理短信、WiFi、APN、DNS、防火墙及 eSIM。

[英文原始说明](README.en.md) · [更新记录](CHANGELOG.md) · [设备端网页说明](zte-agent/docs/web-ui.md) · [API 文档](zte-agent/docs) · [移动端说明](mobile/README.md)

## 汉化范围

- 设备端网页：登录、仪表盘、信号、短信与转发、锁频/锁小区、SIM/eSIM、路由设置、设备控制、工具及操作确认提示。
- 手机应用：补齐 SwiftUI / Jetpack Compose 界面和操作提示，保留已有中文资源。
- 官网：首页、功能介绍、下载、API 说明、常见问题及故障排查，默认简体中文，使用本地系统字体。
- 开发资料：本页提供中文入门；底层协议研究、历史更新、源码注释和完整技术文档仍保留原文。API 路径、JSON 字段、AT 命令、协议值和产品名称不翻译。

汉化需从此 fork 的源码构建。上游网站、上游 Release 二进制及自动更新通道仍由上游提供，安装它们可能恢复英文界面。本 fork 尚未发布独立的中文二进制或 OTA 更新通道。

## 支持的设备

| 项目 | 规格 |
| --- | --- |
| 型号 | ZTE U60 Pro（MU5250） |
| 上游验证固件 | CN_ZTE_MU5250V1.0.0B27 |
| 系统 | ZWRT / OpenWrt 23.05.4，Linux 5.15，aarch64 |
| 芯片 | Qualcomm Snapdragon X75，4 核 Cortex-A55 |
| 无线网络 | 5G-A、LTE、WiFi 7 |
| 电池 | 10000 mAh |
| SIM | 单 nano-SIM 卡槽，没有内置 eUICC |

使用 eSIM 管理需要在 SIM 卡槽中插入兼容的 eSIM 卡（上游验证过 9eSIM B1），普通 SIM 卡不会因此变成 eSIM。

## 编译和安装

构建设备端 Agent 需要 Linux 或 macOS、Rust、ADB，以及 **aarch64 musl 交叉编译器**。仅安装普通 ARM GNU 编译器不能替代 musl 链接器。Windows 建议在 WSL2 的 Linux 环境中构建。

```bash
git clone https://github.com/mobosang/open-u60-pro-cn.git
cd open-u60-pro-cn
rustup target add aarch64-unknown-linux-musl
# 先安装 aarch64-linux-musl-gcc，并确保它在 PATH 中
cargo build --release --target aarch64-unknown-linux-musl -p zte-agent
```

macOS 可使用上游推荐的 musl-cross 工具链：

```bash
brew install filosottile/musl-cross/musl-cross
brew install android-platform-tools
```

首次安装使用仓库自带脚本。连接 USB 后，在终端执行以下命令，将占位内容换成自己的密码：

```bash
./setup.sh '<路由器管理密码>' '<Agent 密码>'
```

路由器管理密码用于原厂管理页面，Agent 密码用于本工具，均不是 WiFi 密码。脚本会启用 ADB、编译并部署 Agent，配置启动脚本，并可选择安装 SSH。请先阅读脚本输出及[英文安装说明](README.en.md#quick-start)。

后续通过 SSH（端口 2222）更新：

```bash
./deploy.sh '<Agent 密码>'
```

部署后，连接路由器的 WiFi 或网线，在浏览器打开 **http://192.168.0.1:9090/**，使用 Agent 密码登录。修改过 LAN 地址时请使用实际地址。HTTPS 可在设置中开启，安装证书前应核对指纹。

## 手机应用和官网

- Android：用 Android Studio 打开 `mobile/android/OpenU60`，同步 Gradle 后构建。
- iOS：按[移动端文档](mobile/README.md)使用 Xcode / XcodeGen 构建 `mobile/ios/OpenU60`，需要 macOS 和相应签名配置。
- 官网：使用 Node.js 22 或更高版本，在 `web` 目录运行：

```bash
npm ci
npm run dev
# 生产构建
npm run build
```

官网是项目介绍和文档站；路由器上的管理网页由 `zte-agent/src/web` 提供，两者分别构建。

## 验证

```bash
# Linux 环境，使用模拟设备数据运行测试
cargo test --locked -p zte-agent -- --skip web::qr_decode::tests::decodes_in_a_child_process_with_a_limit
# 官网类型检查与生产构建
cd web
npx tsc --noEmit
npm run build
```

GitHub Actions 的 Localization checks 会检查设备端测试与官网构建。原始提交 5596b8d 的二维码子进程超时测试在当前 CI 环境也会失败，因此该检查单独跳过这一项，未修改或删除原有测试；需要复核时可运行不带 `--skip` 的命令。软件测试不能代替真机验证，尤其是锁频、切换 eSIM、修改 LAN、重启及恢复出厂设置。

## 使用注意

修改 WiFi、APN、频段、DNS 或防火墙可能中断连接；恢复出厂设置可能删除配置和 Agent。保持设备供电，并保留本地 WiFi/USB 恢复途径。

不要直接禁用 `zte_topsw_daemon` 同步列表中的原厂守护进程，否则可能导致开机卡标志、触屏不响应或移动网络无法连接。参见[项目设备说明](CLAUDE.md)。

## 许可证和归属

Agent 及应用遵循 [MIT](LICENSE)；内置 eSIM 库的许可见 [euicc](euicc) 和[第三方声明](THIRD_PARTY_NOTICES.md)。本项目与中兴通讯无从属、认可或赞助关系。ZTE、U60 Pro、MU5250 及其他商标归其各自权利人所有。
