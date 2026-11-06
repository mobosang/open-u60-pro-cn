import SwiftUI

struct SIMView: View {
    @Bindable var viewModel: SIMViewModel

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

            if viewModel.isPinLocked {
                Section {
                    Label {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("需要 SIM PIN")
                                .font(.headline)
                            Text("输入你的PIN来解锁SIM卡")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                    } icon: {
                        Image(systemName: "lock.fill")
                            .foregroundStyle(.orange)
                    }

                    Button {
                        viewModel.pinSheetAction = .verify
                        viewModel.pinInput = ""
                        viewModel.sheetError = nil
                        viewModel.showEnterPinSheet = true
                    } label: {
                        Text("输入PIN")
                            .frame(maxWidth: .infinity)
                    }
                }
            }

            if viewModel.isPukLocked {
                Section {
                    Label {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("需要 SIM PUK")
                                .font(.headline)
                            Text("PIN 尝试错误太多。输入PUK进行解锁。")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                    } icon: {
                        Image(systemName: "exclamationmark.lock.fill")
                            .foregroundStyle(.red)
                    }

                    Button {
                        viewModel.pukInput = ""
                        viewModel.newPinInput = ""
                        viewModel.sheetError = nil
                        viewModel.showEnterPukSheet = true
                    } label: {
                        Text("输入PUK")
                            .frame(maxWidth: .infinity)
                    }
                }
            }

            Section("SIM 卡") {
                statusRow
                infoRow("ICCID", viewModel.simInfo.iccid)
                infoRow("IMSI", viewModel.simInfo.imsi)
                infoRow("MSISDN", viewModel.simInfo.msisdn)
                infoRow("SPN", viewModel.simInfo.spn)
                mccMncRow
                if !viewModel.simInfo.operatorName.isEmpty {
                    infoRow("运营商", viewModel.simInfo.operatorName)
                }
                infoRow("SIM 插槽", viewModel.simInfo.currentSlot)
            }

            Section("PIN 管理") {
                pinStatusRow

                HStack {
                    Text("PIN 尝试")
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text("\(viewModel.simInfo.pinAttempts)")
                        .font(.body.monospacedDigit())
                        .foregroundStyle(viewModel.simInfo.pinAttempts > 0 && viewModel.simInfo.pinAttempts <= 1 ? .red : .primary)
                }

                HStack {
                    Text("PUK 尝试")
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text("\(viewModel.simInfo.pukAttempts)")
                        .font(.body.monospacedDigit())
                        .foregroundStyle(viewModel.simInfo.pukAttempts > 0 && viewModel.simInfo.pukAttempts <= 3 ? .orange : .primary)
                }

                if !viewModel.isPinLocked && !viewModel.isPukLocked {
                    if viewModel.isPinEnabled {
                        Button {
                            viewModel.pinSheetAction = .disableLock
                            viewModel.pinInput = ""
                            viewModel.sheetError = nil
                            viewModel.showEnterPinSheet = true
                        } label: {
                            Label("禁用 PIN 锁定", systemImage: "lock.open")
                        }
                    } else {
                        Button {
                            viewModel.pinSheetAction = .enableLock
                            viewModel.pinInput = ""
                            viewModel.sheetError = nil
                            viewModel.showEnterPinSheet = true
                        } label: {
                            Label("启用 PIN 锁定", systemImage: "lock")
                        }
                    }

                    Button {
                        viewModel.oldPinInput = ""
                        viewModel.newPinInput = ""
                        viewModel.sheetError = nil
                        viewModel.showChangePinSheet = true
                    } label: {
                        Label("更改PIN", systemImage: "pencil")
                    }
                }
            }

            Section("SIM锁") {
                HStack {
                    Text("解锁尝试")
                        .foregroundStyle(.secondary)
                    Spacer()
                    Text("\(viewModel.lockInfo.availableTrials)")
                        .font(.body.monospacedDigit())
                }

                if viewModel.lockInfo.availableTrials > 0 {
                    Button {
                        viewModel.nckInput = ""
                        viewModel.sheetError = nil
                        viewModel.showUnlockSheet = true
                    } label: {
                        Label("输入解锁码", systemImage: "lock.open")
                    }
                }
            }
        }
        .navigationTitle("SIM 卡")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .task(id: viewModel.identityRevision) { await viewModel.refresh() }
        .task { await viewModel.watchIdentity() }
        .sheet(isPresented: $viewModel.showChangePinSheet) {
            ChangePinSheet(viewModel: viewModel)
        }
        .sheet(isPresented: $viewModel.showEnterPinSheet) {
            EnterPinSheet(viewModel: viewModel)
        }
        .sheet(isPresented: $viewModel.showEnterPukSheet) {
            EnterPukSheet(viewModel: viewModel)
        }
        .sheet(isPresented: $viewModel.showUnlockSheet) {
            UnlockSIMSheet(viewModel: viewModel)
        }
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

    private var mccMncRow: some View {
        let mcc = viewModel.simInfo.mcc
        let mnc = viewModel.simInfo.mnc
        let value = (mcc.isEmpty && mnc.isEmpty) ? "" : "\(mcc)/\(mnc)"
        return infoRow("MCC/MNC", value)
    }

    private var statusRow: some View {
        let raw = viewModel.simInfo.simStatus
        let modem = viewModel.simInfo.modemMainState.lowercased()
        let effective: String
        if raw.isEmpty && !modem.isEmpty {
            effective = modem == "modem_waitpin" ? "wait pin"
                      : modem == "modem_waitpuk" ? "wait puk"
                      : modem == "modem_init_complete" ? "sim ready"
                      : raw
        } else {
            effective = raw
        }
        let label = simStatusLabel(effective)
        let color = simStatusColor(effective)
        return HStack {
            Text("状态")
                .foregroundStyle(.secondary)
            Spacer()
            Text(label)
                .font(.body.monospacedDigit())
                .foregroundStyle(color)
        }
    }

    private var pinStatusRow: some View {
        let enabled = viewModel.isPinEnabled
        return HStack {
            Text("PIN 锁")
                .foregroundStyle(.secondary)
            Spacer()
            Text(enabled ? "Enabled" : "已禁用")
                .font(.body.monospacedDigit())
                .foregroundStyle(enabled ? .green : .secondary)
        }
    }

    // MARK: - Helpers

    private func simStatusLabel(_ raw: String) -> String {
        switch raw.lowercased() {
        case "", "unknown": return "--"
        case "sim ready": return "已就绪"
        case "sim undetected": return "未检测到 SIM"
        case "wait pin": return "需要 PIN"
        case "wait puk": return "需要 PUK"
        case "sim destroy": return "SIM 已损坏"
        case "error", "sim_error": return "错误"
        default: return raw
        }
    }

    private func simStatusColor(_ raw: String) -> Color {
        switch raw.lowercased() {
        case "sim ready": return .green
        case "sim undetected": return .red
        case "wait pin", "wait puk": return .orange
        case "sim destroy": return .red
        case "error", "sim_error": return .red
        default: return .secondary
        }
    }
}

// MARK: - Sheets

/// Renders a failed attempt inside the presented sheet; `SIMView`'s own message row is behind it.
private struct SheetErrorFooter: View {
    let message: String?

    var body: some View {
        if let message {
            Text(message)
                .foregroundStyle(.red)
        }
    }
}

struct ChangePinSheet: View {
    @Bindable var viewModel: SIMViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section("当前PIN") {
                    SecureField("老PIN", text: $viewModel.oldPinInput)
                        .keyboardType(.numberPad)
                }

                Section("新PIN") {
                    SecureField("新PIN", text: $viewModel.newPinInput)
                        .keyboardType(.numberPad)
                }

                Section {
                    Button {
                        Task { await viewModel.changePin() }
                    } label: {
                        Text("更改PIN")
                            .frame(maxWidth: .infinity)
                    }
                    .disabled(viewModel.oldPinInput.count < 4 || viewModel.newPinInput.count < 4 || viewModel.isLoading)
                } footer: {
                    SheetErrorFooter(message: viewModel.sheetError)
                }
            }
            .navigationTitle("更改PIN")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
    }
}

struct EnterPinSheet: View {
    @Bindable var viewModel: SIMViewModel
    @Environment(\.dismiss) private var dismiss

    private var title: String {
        switch viewModel.pinSheetAction {
        case .verify: return "输入PIN"
        case .enableLock: return "启用 PIN 锁定"
        case .disableLock: return "禁用 PIN 锁定"
        }
    }

    private var buttonLabel: String {
        switch viewModel.pinSheetAction {
        case .verify: return "解锁"
        case .enableLock: return "启用"
        case .disableLock: return "禁用"
        }
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    SecureField("PIN", text: $viewModel.pinInput)
                        .keyboardType(.numberPad)
                } footer: {
                    VStack(alignment: .leading, spacing: 4) {
                        if viewModel.pinSheetAction == .verify {
                            Text("\(viewModel.simInfo.pinAttempts) attempts remaining")
                        }
                        SheetErrorFooter(message: viewModel.sheetError)
                    }
                }

                Section {
                    Button {
                        Task { await viewModel.submitPin() }
                    } label: {
                        Text(buttonLabel)
                            .frame(maxWidth: .infinity)
                    }
                    .disabled(viewModel.pinInput.count < 4 || viewModel.isLoading)
                }
            }
            .navigationTitle(title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
    }
}

struct EnterPukSheet: View {
    @Bindable var viewModel: SIMViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    SecureField("PUK代码", text: $viewModel.pukInput)
                        .keyboardType(.numberPad)
                } footer: {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("\(viewModel.simInfo.pukAttempts) attempts remaining")
                        SheetErrorFooter(message: viewModel.sheetError)
                    }
                }

                Section("新PIN") {
                    SecureField("新PIN", text: $viewModel.newPinInput)
                        .keyboardType(.numberPad)
                }

                Section {
                    Button {
                        Task { await viewModel.verifyPuk() }
                    } label: {
                        Text("解锁")
                            .frame(maxWidth: .infinity)
                    }
                    .disabled(viewModel.pukInput.count < 8 || viewModel.newPinInput.count < 4 || viewModel.isLoading)
                }
            }
            .navigationTitle("输入PUK")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
    }
}

struct UnlockSIMSheet: View {
    @Bindable var viewModel: SIMViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("解锁代码（NCK）", text: $viewModel.nckInput)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                } footer: {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("\(viewModel.lockInfo.availableTrials) attempts remaining")
                        SheetErrorFooter(message: viewModel.sheetError)
                    }
                }

                Section {
                    Button {
                        Task { await viewModel.unlockSIM() }
                    } label: {
                        Text("解锁")
                            .frame(maxWidth: .infinity)
                    }
                    .disabled(viewModel.nckInput.isEmpty || viewModel.isLoading)
                }
            }
            .navigationTitle("SIM解锁")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
    }
}
