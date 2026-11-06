import SwiftUI

struct MobileNetworkView: View {
    @Bindable var viewModel: MobileNetworkViewModel

    var body: some View {
        List {
            if let msg = viewModel.message {
                Section {
                    Text(msg)
                        .font(.subheadline)
                        .foregroundStyle(viewModel.messageIsError ? .red : .green)
                        .textSelection(.enabled)
                }
            }

            Section {
                Toggle("飞行模式", isOn: Binding(
                    get: { viewModel.airplaneModeEnabled },
                    set: { val in
                        viewModel.airplaneModeEnabled = val
                        Task { await viewModel.setAirplaneMode(enabled: val) }
                    }
                ))
                .disabled(viewModel.isLoading)

                Toggle("移动数据", isOn: Binding(
                    get: { viewModel.selectedDataEnabled },
                    set: { val in
                        viewModel.selectedDataEnabled = val
                        Task { await viewModel.setMobileData(enabled: val) }
                    }
                ))
                .disabled(viewModel.isLoading || viewModel.airplaneModeEnabled)
            } header: {
                Text("网络连接")
            } footer: {
                Text(mobileDataFooter)
            }

            Section("连接方式") {
                Picker("模式", selection: $viewModel.selectedConnectMode) {
                    Text("自动").tag(1)
                    Text("手动").tag(0)
                }
                .pickerStyle(.segmented)
                .disabled(viewModel.isLoading || viewModel.airplaneModeEnabled)
            }

            Section {
                Toggle("数据漫游", isOn: $viewModel.selectedRoaming)
                    .disabled(viewModel.isLoading || viewModel.airplaneModeEnabled)
            } footer: {
                Text("启用漫游可能会向您的运营商收取额外费用。")
            }

            Section("网络选择") {
                Picker("模式", selection: $viewModel.selectedNetSelectMode) {
                    Text("自动").tag("auto_select")
                    Text("手动").tag("manual_select")
                }
                .pickerStyle(.segmented)
                .disabled(viewModel.isLoading || viewModel.airplaneModeEnabled)

                if viewModel.selectedNetSelectMode == "manual_select" {
                    Button {
                        Task { await viewModel.scanNetworks() }
                    } label: {
                        HStack {
                            Text("扫描网络")
                            Spacer()
                            if viewModel.isScanning {
                                ProgressView()
                            }
                        }
                    }
                    .disabled(viewModel.isScanning)

                    ForEach(viewModel.config.operators) { op in
                        Button {
                            Task { await viewModel.registerNetwork(mccMnc: op.mccMnc, rat: op.rat) }
                        } label: {
                            HStack {
                                VStack(alignment: .leading) {
                                    Text(op.name)
                                        .foregroundStyle(.primary)
                                    Text("\(op.mccMnc) · \(op.rat)")
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                                Spacer()
                                if op.status == "current" {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundStyle(.green)
                                } else if op.status == "forbidden" {
                                    Image(systemName: "xmark.circle")
                                        .foregroundStyle(.red)
                                }
                            }
                        }
                        .disabled(viewModel.isLoading || op.status == "forbidden")
                    }
                }
            }

            Section {
                Button {
                    Task { await viewModel.applySettings() }
                } label: {
                    Text("应用")
                        .frame(maxWidth: .infinity)
                }
                .disabled(viewModel.isLoading || !viewModel.hasChanges || viewModel.airplaneModeEnabled)
            }
        }
        .navigationTitle("移动网络")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .alert("需要重新启动", isPresented: $viewModel.showRebootAfterAirplaneOff) {
            Button("立即重新启动") {
                Task { await viewModel.reboot() }
            }
            Button("取消", role: .cancel) {
                viewModel.airplaneModeEnabled = true
            }
        } message: {
            Text("由于固件限制，不重新启动就无法恢复蜂窝无线电。路由器将重新启动（大约 60 秒）。")
        }
        .task { await viewModel.refresh() }
    }

    private var mobileDataFooter: String {
        if !viewModel.config.isDataEnabled && viewModel.config.isConnected {
            return "移动数据设置已关闭，但连接仍然有效。"
        } else if !viewModel.config.isDataEnabled {
            return "移动数据已禁用。"
        } else if viewModel.config.isConnected {
            let status = viewModel.config.connectStatus
                .replacingOccurrences(of: "_", with: " ")
                .capitalized
            return "已连接 — \(status)"
        } else if !viewModel.config.connectStatus.isEmpty {
            return "未连接"
        } else {
            return "禁用移动数据将断开蜂窝连接。"
        }
    }
}
