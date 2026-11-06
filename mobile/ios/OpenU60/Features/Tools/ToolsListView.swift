import SwiftUI

struct ToolsListView: View {
    let client: AgentClient
    let authManager: AuthManager

    // MARK: - Routes

    private enum Route: Hashable {
        case scheduler
        case smsForwarding
        case speedTest
        case lanSpeedTest
        case enableADB
        case usbMode
        case bandLock
        case atTerminal
        case deviceInfo
        case clients
        case config
        case ttlSettings
        case enableSSH
        case deviceExplorer
    }

    // MARK: - Body

    var body: some View {
        NavigationStack {
            List {
                Section("自动化任务") {
                    NavigationLink(value: Route.scheduler) {
                        Label("自动化", systemImage: "clock.arrow.2.circlepath")
                    }
                    NavigationLink(value: Route.smsForwarding) {
                        Label("短信转发", systemImage: "envelope.arrow.triangle.branch")
                    }
                }

                Section("网络工具") {
                    NavigationLink(value: Route.speedTest) {
                        Label("速度测试", systemImage: "gauge.with.needle")
                    }

                    NavigationLink(value: Route.lanSpeedTest) {
                        Label("局域网速度测试", systemImage: "wifi")
                    }

                    NavigationLink(value: Route.enableADB) {
                        Label("启用 ADB", systemImage: "cable.connector.horizontal")
                    }

                    NavigationLink(value: Route.usbMode) {
                        Label("USB 模式", systemImage: "cable.connector")
                    }

                    NavigationLink(value: Route.bandLock) {
                        Label("频段锁定", systemImage: "lock.fill")
                    }

                    NavigationLink(value: Route.atTerminal) {
                        Label("AT 终端", systemImage: "terminal.fill")
                    }

                    NavigationLink(value: Route.deviceInfo) {
                        Label("设备信息", systemImage: "info.circle")
                    }

                    NavigationLink(value: Route.clients) {
                        Label("已连接设备", systemImage: "laptopcomputer.and.iphone")
                    }
                }

                Section("配置") {
                    NavigationLink(value: Route.config) {
                        Label("配置解密/加密", systemImage: "doc.badge.gearshape")
                    }
                }

                Section("需要 Shell 访问") {
                    NavigationLink(value: Route.ttlSettings) {
                        Label("TTL设置", systemImage: "number")
                    }

                    NavigationLink(value: Route.enableSSH) {
                        Label("启用SSH", systemImage: "terminal")
                    }

                    NavigationLink(value: Route.deviceExplorer) {
                        Label("设备资源管理器", systemImage: "folder")
                    }
                }
            }
            .navigationTitle("工具")
            .navigationDestination(for: Route.self) { route in
                destination(for: route)
            }
        }
    }

    // MARK: - Destinations

    @ViewBuilder
    private func destination(for route: Route) -> some View {
        switch route {
        case .scheduler:
            PushedModelHost(SchedulerViewModel(client: client, authManager: authManager)) {
                SchedulerView(viewModel: $0)
            }
        case .smsForwarding:
            PushedModelHost(SMSForwardViewModel(client: client, authManager: authManager)) {
                SMSForwardConfigView(viewModel: $0)
            }
        case .speedTest:
            PushedModelHost(SpeedTestViewModel(client: client, authManager: authManager)) {
                SpeedTestView(viewModel: $0)
            }
        case .lanSpeedTest:
            PushedModelHost(LANSpeedTestViewModel(client: client)) {
                LANSpeedTestView(viewModel: $0)
            }
        case .enableADB:
            EnableADBView(client: client, authManager: authManager)
        case .usbMode:
            PushedModelHost(USBConnectionViewModel(client: client, authManager: authManager)) {
                USBModeView(viewModel: $0)
            }
        case .bandLock:
            PushedModelHost(BandLockViewModel(client: client, authManager: authManager)) {
                BandLockView(viewModel: $0)
            }
        case .atTerminal:
            PushedModelHost(ATTerminalViewModel(client: client, authManager: authManager)) {
                ATTerminalView(viewModel: $0)
            }
        case .deviceInfo:
            PushedModelHost(DeviceInfoViewModel(client: client, authManager: authManager)) {
                DeviceInfoView(viewModel: $0)
            }
        case .clients:
            PushedModelHost(ClientsViewModel(client: client, authManager: authManager)) {
                ClientsView(viewModel: $0)
            }
        case .config:
            ConfigToolView()
        case .ttlSettings:
            PlaceholderView(title: "TTL设置", icon: "number", description: "通过 iptables 设置 TTL 覆盖。需要外壳访问权限。")
        case .enableSSH:
            PlaceholderView(title: "启用SSH", icon: "terminal", description: "安装并启动dropbear SSH服务器。需要 ADB USB 连接。")
        case .deviceExplorer:
            PlaceholderView(title: "设备资源管理器", icon: "folder", description: "浏览文件系统并收集设备信息。需要 ADB USB 连接。")
        }
    }
}

// MARK: - Pushed model host

/// Holds a pushed screen's view model in `@State` so the model is built once, when the
/// route is actually pushed, and survives re-evaluation of the navigation destination.
///
/// Construction has to happen inside the box: `State.init(wrappedValue:)` is not itself an
/// autoclosure, so building the model in `init` would build and discard one on every
/// re-evaluation of the `.navigationDestination` closure.
private struct PushedModelHost<Model, Content: View>: View {
    @State private var box: LazyModelBox<Model>
    private let content: (Model) -> Content

    init(_ model: @autoclosure @escaping () -> Model, @ViewBuilder content: @escaping (Model) -> Content) {
        _box = State(wrappedValue: LazyModelBox(model))
        self.content = content
    }

    var body: some View {
        content(box.model)
    }
}

/// Defers building a pushed screen's view model until the first time it is read, which only
/// happens for the box `@State` actually keeps.
@MainActor
private final class LazyModelBox<Model> {
    lazy var model: Model = build()

    private let build: () -> Model

    init(_ build: @escaping () -> Model) {
        self.build = build
    }
}
