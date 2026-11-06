import SwiftUI

struct ClientsView: View {
    var viewModel: ClientsViewModel

    @State private var searchText: String = ""

    private var filteredDevices: [ConnectedDevice] {
        let query = searchText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !query.isEmpty else { return viewModel.devices }
        return viewModel.devices.filter {
            $0.displayName.localizedCaseInsensitiveContains(query)
                || $0.ipAddress.localizedCaseInsensitiveContains(query)
                || $0.macAddress.localizedCaseInsensitiveContains(query)
        }
    }

    var body: some View {
        List {
            if let error = viewModel.error {
                Section {
                    Text(error)
                        .foregroundStyle(.red)
                        .font(.caption)
                }
            }

            if !filteredDevices.isEmpty {
                Section("\(filteredDevices.count) Devices") {
                    ForEach(filteredDevices) { device in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(device.displayName)
                                .font(.body.weight(.medium))
                            HStack(spacing: 12) {
                                Label(device.ipAddress.isEmpty ? "--" : device.ipAddress, systemImage: "network")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Text(device.macAddress)
                                    .font(.caption.monospacedDigit())
                                    .foregroundStyle(.tertiary)
                            }
                        }
                        .padding(.vertical, 2)
                        .accessibilityElement(children: .combine)
                    }
                }
            }
        }
        .navigationTitle("已连接设备")
        .searchable(text: $searchText, prompt: "Search devices")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.devices.isEmpty {
                if viewModel.isLoading {
                    ProgressView()
                } else if viewModel.error == nil {
                    ContentUnavailableView {
                        Label("没有设备", systemImage: "wifi.slash")
                    } description: {
                        Text("现在没有任何东西连接到路由器。")
                    }
                }
            } else if filteredDevices.isEmpty {
                ContentUnavailableView.search(text: searchText)
            }
        }
        .task { await viewModel.refresh() }
    }
}
