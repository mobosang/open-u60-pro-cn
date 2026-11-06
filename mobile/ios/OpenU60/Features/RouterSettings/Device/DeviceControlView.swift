import SwiftUI

struct DeviceControlView: View {
    @Bindable var viewModel: DeviceControlViewModel

    private var controlsDisabled: Bool { viewModel.isLoading }

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
                Toggle("充电上限", isOn: Binding(
                    get: { viewModel.chargeLimitEnabled },
                    set: { val in
                        viewModel.chargeLimitEnabled = val
                        Task { await viewModel.setChargeLimit(enabled: val, limit: viewModel.chargeLimit) }
                    }
                ))
                    .disabled(controlsDisabled)

                if viewModel.chargeLimitEnabled {
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(spacing: 0) {
                            Text("停在 ")
                                .font(.subheadline)
                            AnimatedNumber(value: viewModel.chargeLimit, font: .subheadline.monospacedDigit(), textColor: .primary, suffix: "%")
                        }
                        Slider(
                            value: Binding(
                                get: { Double(viewModel.chargeLimit) },
                                set: { viewModel.chargeLimit = Int($0) }
                            ),
                            in: 50...100,
                            step: 5
                        ) {
                            Text("充电上限")
                        } onEditingChanged: { editing in
                            if !editing {
                                Task { await viewModel.setChargeLimit(enabled: true, limit: viewModel.chargeLimit) }
                            }
                        }
                        .disabled(controlsDisabled)
                    }

                    Stepper(value: Binding(
                        get: { viewModel.hysteresis },
                        set: { newVal in
                            viewModel.hysteresis = newVal
                            Task { await viewModel.setChargeLimit(enabled: true, limit: viewModel.chargeLimit, hysteresis: newVal) }
                        }
                    ), in: 1...20) {
                        HStack(spacing: 0) {
                            Text("恢复充电差值： ")
                                .font(.body)
                            AnimatedNumber(value: viewModel.hysteresis, font: .body.monospacedDigit(), textColor: .primary, suffix: "%")
                        }
                    }
                    .disabled(controlsDisabled)
                }
            } footer: {
                if viewModel.chargeLimitEnabled {
                    Text("电量达到 \(viewModel.chargeLimit)% 时停止充电，降至 \(viewModel.chargeLimit - viewModel.hysteresis)% 时恢复。\n\n恢复差值用于避免充电器频繁启停。较小的差值（如 2%）能让电量更接近目标值，但充电器启停更频繁；较大的差值（如 10%）可减少启停次数，但电量波动更大。\n\n默认值为 5%，适合大多数用户。")
                } else {
                    Text("当电池电量达到设定水平时停止充电。延长电池寿命。")
                }
            }

            Section {
                Toggle("省电模式", isOn: Binding(
                    get: { viewModel.powerSaveEnabled },
                    set: { val in
                        viewModel.powerSaveEnabled = val
                        Task { await viewModel.setPowerSave(enabled: val) }
                    }
                ))
                    .disabled(controlsDisabled)
            } footer: {
                Text("限制数据通信速度以减少功耗并延长电池寿命。")
            }

            Section {
                Toggle("快速启动", isOn: Binding(
                    get: { viewModel.fastBootEnabled },
                    set: { val in
                        viewModel.fastBootEnabled = val
                        Task { await viewModel.setFastBoot(enabled: val) }
                    }
                ))
                    .disabled(controlsDisabled)
            } footer: {
                Text("启用后，断电后将暂停至 RAM，以实现近乎即时的启动。禁用使用完全关闭（关闭时节省电池）。")
            }

            Section {
                Toggle("自动睡眠", isOn: Binding(
                    get: { viewModel.autoSleepEnabled },
                    set: { val in
                        viewModel.autoSleepEnabled = val
                        Task { await viewModel.setAutoSleep(enabled: val, timeout: val ? viewModel.autoSleepTimeout : nil) }
                    }
                ))
                    .disabled(controlsDisabled)

                if viewModel.autoSleepEnabled {
                    Picker("空闲超时", selection: Binding(
                        get: { viewModel.autoSleepTimeout },
                        set: { newVal in
                            viewModel.autoSleepTimeout = newVal
                            Task { await viewModel.setAutoSleep(enabled: true, timeout: newVal) }
                        }
                    )) {
                        Text("5 min").tag("5")
                        Text("10 min").tag("10")
                        Text("20 min").tag("20")
                        Text("30 min").tag("30")
                        Text("1 hour").tag("60")
                        Text("2 hours").tag("120")
                    }
                    .disabled(controlsDisabled)
                }
            } footer: {
                Text("当没有 WiFi 客户端连接并且空闲超时到期时，自动进入低功耗睡眠。按电源按钮唤醒。")
            }

            Section {
                Button("重启路由器") {
                    viewModel.showRebootConfirm = true
                }
                .disabled(controlsDisabled)
            } footer: {
                Text("路由器将重新启动。这大约需要 60 秒。")
            }

            Section {
                Button("恢复出厂设置", role: .destructive) {
                    viewModel.showFactoryResetConfirm = true
                }
                .disabled(controlsDisabled)
            } footer: {
                Text("这将清除所有设置并恢复出厂默认设置。此操作无法撤消。")
            }
        }
        .task { await viewModel.refresh() }
        .navigationTitle("设备控制")
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .sheet(isPresented: $viewModel.showRebootConfirm) {
            PasswordConfirmView(
                title: "重启路由器",
                message: "输入您的路由器密码以确认重启。",
                confirmLabel: "重启"
            ) {
                await viewModel.reboot()
            }
        }
        .sheet(isPresented: $viewModel.showFactoryResetConfirm) {
            PasswordConfirmView(
                title: "恢复出厂设置",
                message: "这将删除所有设置。输入您的路由器密码进行确认。",
                confirmLabel: "恢复出厂设置"
            ) {
                await viewModel.factoryReset()
            }
        }
    }
}
