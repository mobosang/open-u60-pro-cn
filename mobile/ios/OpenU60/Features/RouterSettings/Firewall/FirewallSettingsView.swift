import SwiftUI

struct FirewallSettingsView: View {
    @Bindable var viewModel: FirewallSettingsViewModel

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

            Section("Firewall") {
                LabeledContent("Status") {
                    Text(viewModel.config.enabled ? "Enabled" : "Disabled")
                        .foregroundStyle(viewModel.config.enabled ? .green : .secondary)
                }

                Button(viewModel.config.enabled ? "Disable Firewall" : "Enable Firewall") {
                    Task { await viewModel.toggleFirewall(enabled: !viewModel.config.enabled) }
                }
                .disabled(viewModel.isLoading)

                if viewModel.config.enabled {
                    Picker("Level", selection: Binding(
                        get: { viewModel.config.level },
                        set: { level in Task { await viewModel.setLevel(level) } }
                    )) {
                        Text("低").tag("low")
                        Text("中等").tag("medium")
                        Text("高").tag("high")
                    }
                    .disabled(viewModel.isLoading)
                }

                LabeledContent("WAN Ping", value: viewModel.config.wanPingFilter ? "Blocked" : "Allowed")
            }

            Section("网络地址转换/UPnP") {
                Toggle("NAT", isOn: Binding(
                    get: { viewModel.config.nat },
                    set: { enabled in Task { await viewModel.toggleNAT(enabled: enabled) } }
                ))
                .disabled(viewModel.isLoading)

                Toggle("UPnP", isOn: Binding(
                    get: { viewModel.upnpEnabled },
                    set: { enabled in Task { await viewModel.toggleUPnP(enabled: enabled) } }
                ))
                .disabled(viewModel.isLoading)
            }

            Section("DMZ") {
                Toggle("Enable DMZ", isOn: $viewModel.editDmzEnabled)
                if viewModel.editDmzEnabled {
                    TextField("DMZ Host IP", text: $viewModel.editDmzIP)
                        .keyboardType(.decimalPad)
                        .autocorrectionDisabled()
                }
                Button {
                    Task { await viewModel.applyDMZ() }
                } label: {
                    Text("Apply DMZ")
                        .frame(maxWidth: .infinity)
                }
                .disabled(viewModel.isLoading)
            }

            Section("端口转发") {
                Toggle("启用", isOn: Binding(
                    get: { viewModel.config.portForwardEnabled },
                    set: { enabled in Task { await viewModel.togglePortForward(enabled: enabled) } }
                ))
                .disabled(viewModel.isLoading)

                if viewModel.config.portForwardEnabled {
                    Button {
                        viewModel.showAddPortForward = true
                    } label: {
                        Label("Add Rule", systemImage: "plus")
                    }

                    ForEach(viewModel.portForwardRules) { rule in
                        VStack(alignment: .leading, spacing: 4) {
                            HStack {
                                Text(rule.name.isEmpty ? "Rule" : rule.name)
                                    .font(.headline)
                                Spacer()
                                Text(rule.enabled ? "Active" : "Inactive")
                                    .font(.caption)
                                    .foregroundStyle(rule.enabled ? .green : .secondary)
                            }
                            Text("\(rule.protocol_.uppercased()) WAN:\(rule.wanPort) \u{2192} \(rule.lanIP):\(rule.lanPort)")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                        .padding(.vertical, 2)
                        .accessibilityElement(children: .combine)
                        .swipeActions(edge: .trailing) {
                            Button(role: .destructive) {
                                Task { await viewModel.deletePortForward(rule) }
                            } label: {
                                Label("删除", systemImage: "trash")
                            }
                        }
                    }
                }
            }

            if !viewModel.filterRules.isEmpty {
                Section("MAC/IP/端口过滤器") {
                    ForEach(viewModel.filterRules) { rule in
                        VStack(alignment: .leading, spacing: 4) {
                            HStack {
                                Text(rule.protocol_.uppercased())
                                    .font(.headline)
                                Spacer()
                                Text(rule.enabled ? "Active" : "Inactive")
                                    .font(.caption)
                                    .foregroundStyle(rule.enabled ? .green : .secondary)
                            }
                            if !rule.srcMac.isEmpty {
                                Text("MAC：\(rule.srcMac)")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                            if !rule.srcIP.isEmpty || !rule.destIP.isEmpty {
                                Text("\(rule.srcIP):\(rule.srcPort) \u{2192} \(rule.destIP):\(rule.destPort)")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                        }
                        .padding(.vertical, 2)
                        .accessibilityElement(children: .combine)
                    }
                }
            }
        }
        .navigationTitle("Firewall")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .task { await viewModel.refresh() }
        .sheet(isPresented: $viewModel.showAddPortForward) {
            PortForwardFormView(viewModel: viewModel)
        }
    }
}
