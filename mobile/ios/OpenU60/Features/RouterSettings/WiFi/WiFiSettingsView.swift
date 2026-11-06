import SwiftUI

struct WiFiSettingsView: View {
    @Bindable var viewModel: WiFiSettingsViewModel

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
                Toggle("WiFi", isOn: $viewModel.editWifiOnOff)
            }

            if viewModel.editWifiOnOff {
                Section {
                    Toggle("无线电", isOn: Binding(
                        get: { !viewModel.editRadio2gDisabled },
                        set: { viewModel.editRadio2gDisabled = !$0 }
                    ))

                    if !viewModel.editRadio2gDisabled {
                        TextField("SSID", text: $viewModel.editSSID2g)
                            .autocorrectionDisabled()
                        SecureField("密码", text: $viewModel.editKey2g)

                        Picker("信道", selection: $viewModel.editChannel2g) {
                            ForEach(WiFiConfig.channelOptions2g, id: \.self) { ch in
                                Text(ch == "auto" ? "自动" : "Ch \(ch)").tag(ch)
                            }
                        }

                        Picker("带宽", selection: $viewModel.editBandwidth2g) {
                            ForEach(WiFiConfig.bandwidthOptions2g, id: \.self) { bw in
                                Text(bandwidthLabel(bw)).tag(bw)
                            }
                        }

                        Picker("发射功率", selection: $viewModel.editTxpower2g) {
                            ForEach(WiFiConfig.txpowerOptions, id: \.self) { pwr in
                                Text("\(pwr)%").tag(pwr)
                            }
                        }

                        Picker("加密", selection: $viewModel.editEncryption2g) {
                            ForEach(WiFiConfig.encryptionOptions, id: \.self) { enc in
                                Text(encryptionLabel(enc)).tag(enc)
                            }
                        }

                        Toggle("隐藏SSID", isOn: $viewModel.editHidden2g)
                    }
                } header: {
                    Text("2.4 GHz")
                } footer: {
                    VStack(alignment: .leading, spacing: 4) {
                        if let pwr = Int(viewModel.editTxpower2g), pwr <= 20 {
                            Text("低 TX 功率可能会缩小范围并阻止某些客户端连接。")
                        }
                    }
                }

                Section {
                    Toggle("无线电", isOn: Binding(
                        get: { !viewModel.editRadio5gDisabled },
                        set: { viewModel.editRadio5gDisabled = !$0 }
                    ))

                    if !viewModel.editRadio5gDisabled {
                        TextField("SSID", text: $viewModel.editSSID5g)
                            .autocorrectionDisabled()
                        SecureField("密码", text: $viewModel.editKey5g)

                        Picker("信道", selection: $viewModel.editChannel5g) {
                            ForEach(WiFiConfig.channels5g(for: viewModel.editBandwidth5g), id: \.self) { ch in
                                Text(ch == "auto" ? "自动" : "Ch \(ch)").tag(ch)
                            }
                        }

                        Picker("带宽", selection: $viewModel.editBandwidth5g) {
                            ForEach(WiFiConfig.bandwidths5g(for: viewModel.editChannel5g), id: \.self) { bw in
                                Text(bandwidthLabel(bw)).tag(bw)
                            }
                        }
                        .onChange(of: viewModel.editBandwidth5g) { _, newBW in
                            let valid = WiFiConfig.channels5g(for: newBW)
                            if !valid.contains(viewModel.editChannel5g) {
                                viewModel.editChannel5g = "auto"
                            }
                        }
                        .onChange(of: viewModel.editChannel5g) { _, newCh in
                            let valid = WiFiConfig.bandwidths5g(for: newCh)
                            if !valid.contains(viewModel.editBandwidth5g) {
                                viewModel.editBandwidth5g = "auto"
                            }
                        }

                        Picker("发射功率", selection: $viewModel.editTxpower5g) {
                            ForEach(WiFiConfig.txpowerOptions, id: \.self) { pwr in
                                Text("\(pwr)%").tag(pwr)
                            }
                        }

                        Picker("加密", selection: $viewModel.editEncryption5g) {
                            ForEach(WiFiConfig.encryptionOptions, id: \.self) { enc in
                                Text(encryptionLabel(enc)).tag(enc)
                            }
                        }

                        Toggle("隐藏SSID", isOn: $viewModel.editHidden5g)
                    }
                } header: {
                    Text("5 GHz")
                } footer: {
                    VStack(alignment: .leading, spacing: 4) {
                        if viewModel.editBandwidth5g == "EHT160" {
                            Text("160 MHz: channels 36–64 or 100–128")
                        } else if viewModel.editBandwidth5g == "EHT80" {
                            Text("80 MHz: channels 36–64, 100–128, or 149–161")
                        }
                        if viewModel.editBandwidth5g == "EHT20" {
                            Text("5 GHz 下的 20 MHz 带宽较窄，部分客户端可能无法连接或性能较差。建议使用 80 MHz 或更宽带宽以获得更好的兼容性。")
                        }
                        if let pwr = Int(viewModel.editTxpower5g), pwr <= 20 {
                            Text("5 GHz 上的低 TX 功率可能会阻止客户端连接，尤其是在带宽较宽的情况下")
                        }
                    }
                }

                Section("高级") {
                    Toggle("WiFi 7 (802.11be)", isOn: $viewModel.editWifi7Enabled)
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
            } footer: {
                Text("应用更改将在重新启动设置时短暂断开 WiFi 的连接")
            }
        }
        .navigationTitle("WiFi 设置")
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

    private func bandwidthLabel(_ bw: String) -> String {
        switch bw {
        case "auto": return "自动"
        case "EHT20": return "20 MHz"
        case "EHT40": return "40 MHz"
        case "EHT80": return "80 MHz"
        case "EHT160": return "160 MHz"
        default: return bw
        }
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
}
