import SwiftUI

struct RouterSettingsListView: View {
    let client: AgentClient
    let authManager: AuthManager

    // MARK: - Routes

    private enum Route: Hashable {
        case mobileNetwork
        case networkMode
        case cellLock
        case smartTowerConnect
        case signalDetection
        case sim
        case routerESim
        case simServices
        case wifi
        case guestWiFi
        case apn
        case lan
        case dns
        case firewall
        case telemetryBlocker
        case vpnPassthrough
        case qos
        case deviceControls
    }

    // MARK: - Body

    var body: some View {
        NavigationStack {
            List {
                Section("蜂窝网络") {
                    NavigationLink(value: Route.mobileNetwork) {
                        Label("移动网络", systemImage: "cellularbars")
                    }

                    NavigationLink(value: Route.networkMode) {
                        Label("网络模式", systemImage: "antenna.radiowaves.left.and.right")
                    }

                    NavigationLink(value: Route.cellLock) {
                        Label("小区锁定", systemImage: "lock.fill")
                    }

                    NavigationLink(value: Route.smartTowerConnect) {
                        Label("智能基站连接", systemImage: "antenna.radiowaves.left.and.right")
                    }

                    NavigationLink(value: Route.signalDetection) {
                        Label("信号检测", systemImage: "waveform.badge.magnifyingglass")
                    }

                    NavigationLink(value: Route.sim) {
                        Label("SIM 卡", systemImage: "simcard")
                    }

                    NavigationLink(value: Route.routerESim) {
                        Label("eSIM 配置文件", systemImage: "simcard.2")
                    }

                    NavigationLink(value: Route.simServices) {
                        Label("SIM 服务", systemImage: "phone.badge.waveform")
                    }
                }

                Section("网络连接") {
                    NavigationLink(value: Route.wifi) {
                        Label("WiFi", systemImage: "wifi")
                    }

                    NavigationLink(value: Route.guestWiFi) {
                        Label("访客 WiFi", systemImage: "person.2")
                    }

                    NavigationLink(value: Route.apn) {
                        Label("APN", systemImage: "simcard")
                    }

                    NavigationLink(value: Route.lan) {
                        Label("LAN / DHCP", systemImage: "network")
                    }

                    NavigationLink(value: Route.dns) {
                        Label("DNS", systemImage: "globe")
                    }
                }

                Section("安全") {
                    NavigationLink(value: Route.firewall) {
                        Label("防火墙", systemImage: "shield.lefthalf.filled")
                    }

                    NavigationLink(value: Route.telemetryBlocker) {
                        Label("遥测拦截", systemImage: "eye.slash")
                    }

                    NavigationLink(value: Route.vpnPassthrough) {
                        Label("VPN 穿透", systemImage: "lock.shield")
                    }
                }

                Section("服务质量") {
                    NavigationLink(value: Route.qos) {
                        Label("服务质量", systemImage: "speedometer")
                    }
                }

                Section("系统") {
                    NavigationLink(value: Route.deviceControls) {
                        Label("设备控制", systemImage: "power")
                    }
                }
            }
            .navigationTitle("路由器")
            .navigationDestination(for: Route.self) { route in
                destination(for: route)
            }
        }
    }

    // MARK: - Destinations

    @ViewBuilder
    private func destination(for route: Route) -> some View {
        switch route {
        case .mobileNetwork:
            PushedModelHost(MobileNetworkViewModel(client: client, authManager: authManager)) {
                MobileNetworkView(viewModel: $0)
            }
        case .networkMode:
            PushedModelHost(NetworkModeViewModel(client: client, authManager: authManager)) {
                NetworkModeView(viewModel: $0)
            }
        case .cellLock:
            PushedModelHost(CellLockViewModel(client: client, authManager: authManager)) {
                CellLockView(viewModel: $0)
            }
        case .smartTowerConnect:
            PushedModelHost(STCViewModel(client: client, authManager: authManager)) {
                STCView(viewModel: $0)
            }
        case .signalDetection:
            PushedModelHost(SignalDetectViewModel(client: client, authManager: authManager)) {
                SignalDetectView(viewModel: $0)
            }
        case .sim:
            PushedModelHost(SIMViewModel(client: client, authManager: authManager)) {
                SIMView(viewModel: $0)
            }
        case .routerESim:
            PushedModelHost(RouterESimViewModel(client: client, authManager: authManager)) {
                RouterESimView(viewModel: $0)
            }
        case .simServices:
            PushedModelHost(STKViewModel(client: client, authManager: authManager)) {
                STKMenuView(viewModel: $0)
            }
        case .wifi:
            PushedModelHost(WiFiSettingsViewModel(client: client, authManager: authManager)) {
                WiFiSettingsView(viewModel: $0)
            }
        case .guestWiFi:
            PushedModelHost(GuestWiFiSettingsViewModel(client: client, authManager: authManager)) {
                GuestWiFiSettingsView(viewModel: $0)
            }
        case .apn:
            PushedModelHost(APNViewModel(client: client, authManager: authManager)) {
                APNView(viewModel: $0)
            }
        case .lan:
            PushedModelHost(LANSettingsViewModel(client: client, authManager: authManager)) {
                LANSettingsView(viewModel: $0)
            }
        case .dns:
            PushedModelHost(DNSSettingsViewModel(client: client, authManager: authManager)) {
                DNSSettingsView(viewModel: $0)
            }
        case .firewall:
            PushedModelHost(FirewallSettingsViewModel(client: client, authManager: authManager)) {
                FirewallSettingsView(viewModel: $0)
            }
        case .telemetryBlocker:
            PushedModelHost(TelemetryBlockerViewModel(client: client, authManager: authManager)) {
                TelemetryBlockerView(viewModel: $0)
            }
        case .vpnPassthrough:
            PushedModelHost(VPNPassthroughViewModel(client: client, authManager: authManager)) {
                VPNPassthroughView(viewModel: $0)
            }
        case .qos:
            PushedModelHost(QoSViewModel(client: client, authManager: authManager)) {
                QoSView(viewModel: $0)
            }
        case .deviceControls:
            PushedModelHost(DeviceControlViewModel(client: client, authManager: authManager)) {
                DeviceControlView(viewModel: $0)
            }
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
