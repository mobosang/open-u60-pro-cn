import SwiftUI

struct DeviceInfoView: View {
    var viewModel: DeviceInfoViewModel

    var body: some View {
        List {
            Section("设备") {
                infoRow("IMEI", viewModel.identity.imei)
            }

            Section("网络") {
                roamingRow
                signalBarsRow
            }

            Section("WAN") {
                infoRow("IPv4", viewModel.identity.wanIPv4)
                ForEach(viewModel.identity.wanIPv6, id: \.self) { addr in
                    infoRow("IPv6", addr)
                }
                if viewModel.identity.wanIPv6.isEmpty {
                    infoRow("IPv6", "")
                }
            }

            Section("LAN") {
                infoRow("网关", viewModel.identity.lanIP)
            }
        }
        .navigationTitle("设备信息")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
            }
        }
        .task(id: viewModel.identityRevision) { await viewModel.refresh() }
        .task { await viewModel.watchIdentity() }
    }

    // MARK: - Rows

    private func infoRow(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label)
                .foregroundStyle(.secondary)
            Spacer()
            Text(value.isEmpty ? "--" : value)
                .font(.body.monospacedDigit())
                .textSelection(.enabled)
        }
    }

    private var roamingRow: some View {
        let roaming = viewModel.operatorInfo.roaming
        return HStack {
            Text("漫游")
                .foregroundStyle(.secondary)
            Spacer()
            Text(roaming ? "漫游" : "本地网络")
                .font(.body.monospacedDigit())
                .foregroundStyle(roaming ? .orange : .green)
        }
    }

    private var signalBarsRow: some View {
        let bars = viewModel.operatorInfo.signalBar
        let maxBars = 5
        return HStack {
            Text("信号")
                .foregroundStyle(.secondary)
            Spacer()
            Image(systemName: "cellularbars")
                .foregroundStyle(bars > 0 ? signalColor(bars: bars, max: maxBars) : .secondary)
            Text("\(bars)/\(maxBars)")
                .font(.body.monospacedDigit())
                .foregroundStyle(.secondary)
                .padding(.leading, 4)
        }
    }

    // MARK: - Helpers

    private func signalColor(bars: Int, max: Int) -> Color {
        let ratio = Double(bars) / Double(max)
        if ratio >= 0.6 { return .green }
        if ratio >= 0.4 { return .yellow }
        return .red
    }
}
