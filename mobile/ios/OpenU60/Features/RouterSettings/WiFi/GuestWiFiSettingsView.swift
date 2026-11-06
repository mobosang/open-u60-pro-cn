import SwiftUI

struct GuestWiFiSettingsView: View {
    @Bindable var viewModel: GuestWiFiSettingsViewModel

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
                Toggle("2.4 GHz", isOn: $viewModel.editEnabled2g)
                Toggle("5 GHz", isOn: $viewModel.editEnabled5g)
            } header: {
                Text("无线电频段")
            } footer: {
                if viewModel.isTimerExpired {
                    Text("计时器到期 — 访客 WiFi 被自动禁用")
                        .foregroundStyle(.orange)
                }
            }

            if viewModel.isAnyBandEnabled {
                Section("网络") {
                    TextField("SSID", text: $viewModel.editSsid)
                        .autocorrectionDisabled()
                    SecureField("密码", text: $viewModel.editKey)

                    Picker("加密", selection: $viewModel.editEncryption) {
                        ForEach(WiFiConfig.encryptionOptions, id: \.self) { enc in
                            Text(encryptionLabel(enc)).tag(enc)
                        }
                    }

                    Toggle("隐藏SSID", isOn: $viewModel.editHidden)
                    Toggle("客户端隔离", isOn: $viewModel.editIsolate)
                }
            }

            if viewModel.isAnyBandEnabled || viewModel.isTimerExpired || viewModel.remainingSeconds > 0 {
                Section {
                    Picker("自动关闭", selection: $viewModel.editActiveTime) {
                        ForEach(GuestWiFiConfig.activeTimeOptions, id: \.minutes) { option in
                            Text(option.label).tag(option.minutes)
                        }
                    }
                } header: {
                    Text("计时器")
                } footer: {
                    VStack(alignment: .leading, spacing: 4) {
                        if viewModel.editActiveTime > 0 {
                            Text("访客 WiFi 将在 \(activeTimeLabel(viewModel.editActiveTime)) 后自动关闭")
                        }
                        if let remaining = viewModel.remainingTimeText {
                            Text(remaining)
                                .foregroundStyle(.orange)
                        }
                    }
                }
            }

            Section {
                Button {
                    Task { await viewModel.apply() }
                } label: {
                    Text("应用")
                        .frame(maxWidth: .infinity)
                }
                .disabled(viewModel.isLoading)
            }
        }
        .navigationTitle("访客 WiFi")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .task { await viewModel.refresh() }
    }

    private func encryptionLabel(_ enc: String) -> String {
        switch enc {
        case "none": return "无"
        case "psk+tkip": return "WPA-PSK (TKIP)"
        case "psk+ccmp": return "WPA-PSK (AES)"
        case "psk2+ccmp": return "WPA2-PSK (AES)"
        case "psk-mixed+ccmp": return "WPA/WPA2 混合"
        case "sae": return "WPA3-SAE"
        case "sae-mixed": return "WPA2/WPA3 混合"
        default: return enc
        }
    }

    private func activeTimeLabel(_ minutes: Int) -> String {
        GuestWiFiConfig.activeTimeOptions.first { $0.minutes == minutes }?.label ?? "\(minutes) min"
    }
}
