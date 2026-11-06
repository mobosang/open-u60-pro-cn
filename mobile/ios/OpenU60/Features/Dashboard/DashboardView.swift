import SwiftUI

struct DashboardView: View {
    var viewModel: DashboardViewModel
    let isAuthenticated: Bool
    let client: AgentClient
    let authManager: AuthManager

    @State private var signalMonitorVM: SignalMonitorViewModel?
    @State private var networkModeVM: NetworkModeViewModel?
    @State private var detail: DashboardDetail?
    @State private var showCPUDetails = false
    @Environment(\.dynamicTypeSize) private var dynamicTypeSize
    @State private var showAllDevices = true
    @State private var showWiFiShare = false

    init(viewModel: DashboardViewModel, isAuthenticated: Bool, client: AgentClient, authManager: AuthManager) {
        self.viewModel = viewModel
        self.isAuthenticated = isAuthenticated
        self.client = client
        self.authManager = authManager
    }

    var body: some View {
        let banners = bannerConfigs
        NavigationStack {
            ScrollView {
                // Keep summary view identities stable while samples update and sheets open.
                VStack(spacing: 12) {
                    ForEach(banners) { config in
                        SIMAlertBanner(config: config)
                            .transition(.move(edge: .top).combined(with: .opacity))
                    }
                    Button { detail = .connection } label: {
                        ConnectionOverview(viewModel: viewModel)
                    }
                    .buttonStyle(.plain)
                    Button { detail = .traffic } label: {
                        CellularCardScope(viewModel: viewModel, compact: true)
                    }
                    .buttonStyle(.plain)
                    SignalActivityControl()
                    let healthLayout = dynamicTypeSize.isAccessibilitySize
                        ? AnyLayout(VStackLayout(spacing: 12))
                        : AnyLayout(HStackLayout(alignment: .top, spacing: 12))
                    healthLayout {
                        Button { detail = .battery } label: {
                            BatteryCardScope(viewModel: viewModel)
                        }
                        .buttonStyle(.plain)
                        Button { detail = .health } label: {
                            HealthOverview(viewModel: viewModel)
                        }
                        .buttonStyle(.plain)
                    }
                    Button { detail = .wifi } label: {
                        WiFiOverview(viewModel: viewModel)
                    }
                    .buttonStyle(.plain)
                }
                .padding()
                .animation(.easeInOut, value: banners)
            }
            .background(Color(.systemGroupedBackground))
            .task {
                if signalMonitorVM == nil {
                    signalMonitorVM = SignalMonitorViewModel(client: client, authManager: authManager)
                }
                if networkModeVM == nil {
                    networkModeVM = NetworkModeViewModel(client: client, authManager: authManager)
                }
            }
            .sheet(item: $detail) { selected in
                dashboardDetail(selected)
                    .presentationDetents([.large])
                    .presentationDragIndicator(.visible)
            }
            .navigationTitle("仪表盘")
            .navigationBarTitleDisplayMode(.inline)
            .refreshable { await viewModel.refresh() }
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    LastUpdatedScope(viewModel: viewModel)
                }
                ToolbarItem(placement: .topBarLeading) {
                    connectionIndicator
                }
            }
        }
    }

    @ViewBuilder
    private func dashboardDetail(_ selected: DashboardDetail) -> some View {
        switch selected {
        case .battery:
            BatteryDetailScope(viewModel: viewModel)
        case .health:
            NavigationStack {
                ScrollView {
                    VStack(spacing: 12) {
                        CPUCardScope(viewModel: viewModel)
                        Button("CPU、内存和进程") { showCPUDetails = true }
                        TemperatureCardScope(viewModel: viewModel)
                    }.padding()
                }
                .background(Color(.systemGroupedBackground))
                .sheet(isPresented: $showCPUDetails) {
                    CPUDetailScope(viewModel: viewModel, client: client)
                        .presentationDragIndicator(.visible)
                }
                .navigationTitle("CPU 和温度")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar { ToolbarItem(placement: .confirmationAction) { Button("完成") { detail = nil } } }
            }
        case .connection:
            NavigationStack {
                if let signalMonitorVM {
                    ConnectionSignalDetail(dashboard: viewModel, viewModel: signalMonitorVM,
                                           networkModeViewModel: networkModeVM)
                } else {
                    ProgressView("Loading connection details…")
                }
            }
        case .traffic, .wifi:
            NavigationStack {
                ScrollView {
                    VStack(spacing: 12) {
                        if selected == .traffic {
                            CellularCardScope(viewModel: viewModel, compact: true)
                            Text("这些速度显示所有连接设备上当前的蜂窝流量。速度测试报告测试期间的吞吐量；在此期间，实时数据会发生变化。")
                                .font(.subheadline).foregroundStyle(.secondary)
                        } else {
                            WiFiCardScope(viewModel: viewModel, showWiFiShare: $showWiFiShare)
                            Button(showWiFiShare ? "Hide Wi-Fi sharing" : "共享 Wi-Fi") { showWiFiShare.toggle() }
                            if showWiFiShare {
                                WiFiShareCardScope(viewModel: viewModel, client: client,
                                    authManager: authManager, isExpanded: $showWiFiShare)
                            }
                            DevicesCardScope(viewModel: viewModel, showAllDevices: $showAllDevices)
                        }
                    }.padding()
                }
                .background(Color(.systemGroupedBackground))
                .navigationTitle(selected.rawValue)
                .navigationBarTitleDisplayMode(.inline)
                .toolbar { ToolbarItem(placement: .confirmationAction) { Button("完成") { detail = nil } } }
            }
        }
    }

    private var bannerConfigs: [SIMAlertBanner.Config] {
        var configs: [SIMAlertBanner.Config] = []
        if viewModel.simPukRequired {
            configs.append(SIMAlertBanner.Config(
                id: "puk",
                icon: "exclamationmark.lock.fill",
                title: "需要 SIM PUK",
                message: "PIN 错误次数过多。请前往“路由器 > SIM 卡”输入 PUK。",
                color: .red
            ))
        }
        if viewModel.simPinRequired {
            configs.append(SIMAlertBanner.Config(
                id: "pin",
                icon: "lock.fill",
                title: "需要 SIM PIN",
                message: "SIM 卡已锁定。请前往“路由器 > SIM 卡”输入 PIN。",
                color: .orange
            ))
        }
        if viewModel.isAirplaneMode {
            configs.append(SIMAlertBanner.Config(
                id: "airplane",
                icon: "airplane",
                title: "飞行模式",
                message: "蜂窝无线电已关闭。调制解调器已断电 — 无信号或数据。",
                color: .blue
            ))
        }
        if viewModel.isMobileDataOff && !viewModel.isAirplaneMode {
            configs.append(SIMAlertBanner.Config(
                id: "data",
                icon: "antenna.radiowaves.left.and.right.slash",
                title: "移动数据关闭",
                message: "蜂窝无线电已开启，但移动数据已关闭。请前往“路由器 > 移动网络”开启。",
                color: .orange
            ))
        }
        return configs
    }

    private var connectionIndicator: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(isAuthenticated ? .green : .red)
                .frame(width: 10, height: 10)
            Text(isAuthenticated ? "Connected" : "离线")
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(isAuthenticated ? .green : .red)
        }
        .fixedSize()
        .padding(.horizontal, 10)
        .padding(.vertical, 5)
        .background(
            Capsule()
                .fill((isAuthenticated ? Color.green : Color.red).opacity(0.12))
        )
    }
}

// MARK: - Observation scopes

// Each wrapper below reads the DashboardViewModel properties for exactly one card inside its own
// body. @Observation installs tracking per body evaluation, not per property, so
// `let battery = viewModel.battery` in DashboardView.body would still subscribe the whole
// dashboard to `battery`. Only a separate View type moves the read into its own invalidation
// unit; passing the view model itself down reads nothing and so tracks nothing.

private struct CellularCardScope: View {
    let viewModel: DashboardViewModel
    var compact = false

    var body: some View {
        CellularCardView(
            wanIPv4: viewModel.wanIPv4,
            wanIPv6: viewModel.wanIPv6,
            speed: viewModel.speed,
            trafficStats: viewModel.trafficStats,
            isTrafficAvailable: viewModel.isTrafficAvailable,
            trafficUpdatedAt: viewModel.trafficUpdatedAt,
            trafficSampleInterval: viewModel.trafficSampleInterval,
            compact: compact
        )
    }
}

private struct BatteryCardScope: View {
    let viewModel: DashboardViewModel

    var body: some View {
        BatteryCardView(battery: viewModel.battery, compact: true)
    }
}

private struct CPUCardScope: View {
    let viewModel: DashboardViewModel

    var body: some View {
        CPUCardView(systemInfo: viewModel.systemInfo, thermal: viewModel.thermal, cpuAvailable: viewModel.cpuUpdatedAt != nil)
    }
}

private struct WiFiCardScope: View {
    let viewModel: DashboardViewModel
    @Binding var showWiFiShare: Bool

    var body: some View {
        WiFiCardView(wifiStatus: viewModel.wifiStatus, showWiFiShare: $showWiFiShare)
    }
}

private struct WiFiShareCardScope: View {
    let viewModel: DashboardViewModel
    let client: AgentClient
    let authManager: AuthManager
    @Binding var isExpanded: Bool

    var body: some View {
        WiFiShareCardView(
            wifiStatus: viewModel.wifiStatus,
            client: client,
            authManager: authManager,
            isExpanded: $isExpanded
        )
    }
}

private struct DevicesCardScope: View {
    let viewModel: DashboardViewModel
    @Binding var showAllDevices: Bool

    var body: some View {
        DevicesCardView(connectedDevices: viewModel.connectedDevices, showAllDevices: $showAllDevices)
    }
}

private struct LastUpdatedScope: View {
    let viewModel: DashboardViewModel

    var body: some View {
        LastUpdatedView(date: viewModel.lastUpdated)
    }
}

private struct BatteryDetailScope: View {
    let viewModel: DashboardViewModel

    var body: some View {
        BatteryDetailSheet(battery: viewModel.battery)
    }
}

private struct CPUDetailScope: View {
    let viewModel: DashboardViewModel
    let client: AgentClient

    var body: some View {
        CPUDetailSheet(cpuAvailable: viewModel.cpuUpdatedAt != nil, systemInfo: viewModel.systemInfo, thermal: viewModel.thermal, client: client)
            // Killing bloat frees ~225 MB at once, and system info only reloads on the 60s tier.
            // Nothing inside a sheet can reach pull-to-refresh, so the sheet stack asks for one.
            .environment(\.processKillRefresh, { await viewModel.refresh() })
    }
}

// MARK: - Connection & signal detail

/// The combined sheet owns signal polling while visible. Keep the dashboard's fetch suspended
/// for the same lifetime, balancing appearances when navigating to network mode or another tab.
private struct ConnectionSignalDetail: View {
    let dashboard: DashboardViewModel
    let viewModel: SignalMonitorViewModel
    let networkModeViewModel: NetworkModeViewModel?

    @Environment(\.dismiss) private var dismiss
    @State private var isSuspended = false

    var body: some View {
        SignalMonitorView(viewModel: viewModel) {
            ConnectionAddresses(viewModel: dashboard)
            if let networkModeViewModel {
                NavigationLink("网络模式") { NetworkModeView(viewModel: networkModeViewModel) }
            }
        }
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("完成") { dismiss() }
                }
            }
            .onAppear {
                guard !isSuspended else { return }
                isSuspended = true
                dashboard.suspendSignalFetch()
            }
            .onDisappear {
                guard isSuspended else { return }
                isSuspended = false
                dashboard.resumeSignalFetch()
            }
    }
}

// MARK: - Banner

private struct SIMAlertBanner: View {
    struct Config: Identifiable, Equatable {
        let id: String
        let icon: String
        let title: String
        let message: String
        let color: Color
    }

    let config: Config

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: config.icon)
                .font(.title2)
                .foregroundStyle(config.color)
            VStack(alignment: .leading, spacing: 2) {
                Text(config.title).font(.subheadline.bold()).textSelection(.enabled)
                Text(config.message).font(.caption).foregroundStyle(.secondary).textSelection(.enabled)
            }
            Spacer()
        }
        .accessibilityElement(children: .combine)
        .padding(12)
        .background(config.color.opacity(0.1), in: RoundedRectangle(cornerRadius: 12))
        .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(config.color.opacity(0.3), lineWidth: 1))
    }
}


private struct TemperatureCardScope: View {
    let viewModel: DashboardViewModel

    var body: some View {
        TemperatureCardView(thermal: viewModel.thermal, battery: viewModel.battery,
            thermalUpdatedAt: viewModel.thermalUpdatedAt, batteryUpdatedAt: viewModel.batteryUpdatedAt)
    }
}

private enum DashboardDetail: String, Identifiable {
    case connection = "连接与信号"
    case traffic = "Live traffic"
    case battery = "Battery"
    case health = "CPU 和温度"
    case wifi = "Wi-Fi 和设备"
    var id: String { rawValue }
}

private struct OverviewHeading: View {
    let title: String
    let icon: String
    var body: some View {
        HStack {
            Label(title, systemImage: icon).font(.subheadline.weight(.semibold))
            Spacer(minLength: 4)
            Image(systemName: "chevron.right").font(.caption2.weight(.semibold)).foregroundStyle(.tertiary)
        }
    }
}

private struct ConnectionOverview: View {
    let viewModel: DashboardViewModel
    var body: some View {
        let info = viewModel.operatorInfo
        let nr = viewModel.nrSignal
        let lte = viewModel.lteSignal
        CardView {
            VStack(alignment: .leading, spacing: 10) {
                OverviewHeading(title: "连接与信号", icon: "antenna.radiowaves.left.and.right")
                HStack(alignment: .firstTextBaseline) {
                    Text(info.provider.isEmpty ? "No operator" : info.provider).font(.title3.bold())
                    Spacer()
                    Text(info.networkType.isEmpty ? "—" : info.displayNetworkType(nrConnected: nr.isConnected, lteSignal: lte))
                        .font(.subheadline.weight(.semibold)).foregroundStyle(.blue)
                }
                HStack(spacing: 6) {
                    Text(nr.isConnected ? "NR RSRP" : "LTE RSRP").font(.caption).foregroundStyle(.secondary)
                    if let rsrp = nr.isConnected ? nr.rsrp : lte.rsrp {
                        AnimatedNumber(value: Int(rsrp), font: .caption.weight(.semibold), suffix: " dBm")
                    } else { Text("—").font(.caption) }
                    Spacer()
                    if info.roaming { Text("漫游").font(.caption).foregroundStyle(.orange) }
                    Text("电台详情").font(.caption).foregroundStyle(.secondary)
                }
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityHint("打开连接和信号详情")
    }
}

private struct HealthOverview: View {
    let viewModel: DashboardViewModel
    var body: some View {
        CardView {
            VStack(alignment: .leading, spacing: 10) {
                OverviewHeading(title: "CPU与散热", icon: "cpu")
                if viewModel.cpuUpdatedAt != nil {
                    CPUUsageValue(value: viewModel.systemInfo.cpuUsagePercent)
                } else { Text("—").font(.title3.bold()) }
                TimelineView(.periodic(from: .now, by: 1)) { context in
                    VStack(spacing: 6) {
                        temperature("CPU", value: viewModel.thermal.cpuTemp, date: viewModel.thermalUpdatedAt, now: context.date)
                        temperature("电池", value: viewModel.battery.temperature, date: viewModel.batteryUpdatedAt, now: context.date)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .accessibilityElement(children: .combine)
        .accessibilityHint("打开 CPU 信息及全部温度传感器")
    }
    private func temperature(_ label: String, value: Double?, date: Date?, now: Date) -> some View {
        HStack {
            Text(label).foregroundStyle(.secondary)
            Spacer(minLength: 4)
            TemperatureValueText(value: date.map { now.timeIntervalSince($0) < 10 } == true ? value : nil)
        }.font(.caption)
    }
}

private struct WiFiOverview: View {
    let viewModel: DashboardViewModel
    var body: some View {
        CardView {
            VStack(alignment: .leading, spacing: 10) {
                OverviewHeading(title: "Wi-Fi 和设备", icon: "wifi")
                HStack(alignment: .firstTextBaseline) {
                    Text(viewModel.wifiStatus.wifiOn ? "Wi-Fi on" : "无线网络关闭")
                        .font(.subheadline).foregroundStyle(viewModel.wifiStatus.wifiOn ? .green : .secondary)
                    Spacer()
                    AnimatedNumber(value: viewModel.connectedDevices.count, font: .title3.bold())
                    Text("devices").font(.caption).foregroundStyle(.secondary)
                }
                Text("网络、共享和连接设备")
                    .font(.caption).foregroundStyle(.secondary)
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityHint("打开 Wi-Fi 网络与设备列表")
    }
}

private struct ConnectionAddresses: View {
    let viewModel: DashboardViewModel
    var body: some View {
        CardView(padding: 12) {
            VStack(alignment: .leading, spacing: 8) {
                Text("广域网地址").font(.headline)
                LabeledContent("IPv4", value: viewModel.wanIPv4.isEmpty ? "—" : viewModel.wanIPv4)
                LabeledContent("IPv6", value: viewModel.wanIPv6.isEmpty ? "—" : viewModel.wanIPv6)
            }.font(.caption).textSelection(.enabled)
        }
    }
}
