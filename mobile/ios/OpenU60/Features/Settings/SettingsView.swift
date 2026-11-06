import SwiftUI

struct SettingsView: View {
    @State private var viewModel: SettingsViewModel
    @FocusState private var gatewayFieldFocused: Bool

    init(client: AgentClient) {
        _viewModel = State(initialValue: SettingsViewModel(client: client))
    }

    var body: some View {
        @Bindable var vm = viewModel
        NavigationStack {
            Form {
                Section {
                    HStack {
                        TextField("网关IP", text: $vm.gatewayIP)
                            .keyboardType(.decimalPad)
                            .autocorrectionDisabled()
                            .focused($gatewayFieldFocused)
                            .onSubmit { viewModel.commitGatewayIP() }
                            .onChange(of: gatewayFieldFocused) { _, isFocused in
                                if !isFocused { viewModel.commitGatewayIP() }
                            }
                            // Detection walks the client's base URL across candidates; an edit
                            // committed mid-probe would fight it for the same property.
                            .disabled(viewModel.isDetectingGateway)
                        Button("检测") {
                            gatewayFieldFocused = false
                            Task { await viewModel.autoDetectGateway() }
                        }
                        .buttonStyle(.bordered)
                        .font(.caption)
                        .disabled(viewModel.isDetectingGateway)
                        if viewModel.isDetectingGateway {
                            ProgressView()
                        }
                    }
                } header: {
                    Text("网关")
                } footer: {
                    if viewModel.detectionFailed {
                        Text("没有代理应答公共网关地址。之前的地址仍在使用。")
                    }
                }

                Section("认证") {
                    if viewModel.hasStoredPassword {
                        HStack {
                            Text("密码存储在Keychain中")
                                .foregroundStyle(.secondary)
                            Spacer()
                            Button("清除", role: .destructive) {
                                viewModel.clearPassword()
                            }
                            .font(.caption)
                        }
                    }
                    SecureField("新密码", text: $vm.passwordInput)
                    Button("保存至Keychain") {
                        withAnimation { viewModel.savePassword() }
                    }
                    .disabled(viewModel.passwordInput.isEmpty)
                }

                Section {
                    VStack(alignment: .leading) {
                        Text("Refresh interval: \(viewModel.pollInterval, specifier: "%.1f")s")
                        Slider(value: $vm.pollInterval, in: 1...10, step: 0.5)
                            .accessibilityLabel("刷新间隔")
                            .accessibilityValue(String(format: "%.1f seconds", viewModel.pollInterval))
                    }
                } header: {
                    Text("轮询")
                } footer: {
                    Text("实时上传和下载使用单独的 1 秒刷新，与调制解调器的更新速率相匹配。")
                }

                Section("外观") {
                    Picker("主题", selection: $vm.darkModeOverride) {
                        Text("系统").tag(0)
                        Text("浅色").tag(1)
                        Text("深色").tag(2)
                    }
                    .pickerStyle(.segmented)
                }

                Section("关于") {
                    LabeledContent("应用程序", value: "OpenU60")
                    LabeledContent("设备", value: "ZTE U60 Pro (MU5250)")
                    LabeledContent("API", value: "zte-agent REST")
                }

                Section("法律声明") {
                    Text("此应用程序不隶属于 ZTE Corporation，也不受其认可或赞助。 ZTE 和 U60 Pro 是 ZTE Corporation 的商标。")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    Link("隐私政策", destination: URL(string: "https://open-u60-pro.vercel.app/privacy")!)
                }
            }
            .navigationTitle("设置")
            .overlay {
                if viewModel.showSavedConfirmation {
                    savedToast
                }
            }
            .task { await viewModel.refreshStoredPasswordState() }
        }
    }

    private var savedToast: some View {
        Text("密码已保存")
            .font(.subheadline.weight(.medium))
            .padding(.horizontal, 16)
            .padding(.vertical, 10)
            .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 10))
            .transition(.move(edge: .top).combined(with: .opacity))
            .task {
                try? await Task.sleep(for: .seconds(1.5))
                withAnimation { viewModel.showSavedConfirmation = false }
            }
    }
}
