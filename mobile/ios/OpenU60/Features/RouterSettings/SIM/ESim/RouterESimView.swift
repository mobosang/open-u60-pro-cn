import SwiftUI
import UIKit

private extension Color {
    // Darken teal text on light surfaces while retaining the system color in dark mode.
    static let eSimAccent = Color(uiColor: UIColor { traits in
        traits.userInterfaceStyle == .dark ? .systemTeal : UIColor(red: 0, green: 0.40, blue: 0.42, alpha: 1)
    })
}

struct RouterESimView: View {
    @Bindable var viewModel: RouterESimViewModel
    @Environment(\.scenePhase) private var scenePhase
    @State private var isVisible = false
    @State private var adding = false
    @State private var addedSwitch: RouterESimProfile?

    private var otherProfiles: [RouterESimProfile] { viewModel.profiles.filter { !$0.isEnabled } }

    var body: some View {
        dialogs(screen)
    }

    private var screen: some View {
        ScrollViewReader { proxy in
            List { screenContent }
                .listStyle(.plain)
                .scrollContentBackground(.hidden)
                .environment(\.defaultMinListRowHeight, 0)
                .onChange(of: viewModel.message) { _, message in
                    if message != nil { withAnimation { proxy.scrollTo("esim-top", anchor: .top) } }
                }
                .onChange(of: viewModel.switchingTo) { _, target in
                    if target != nil { withAnimation { proxy.scrollTo("esim-top", anchor: .top) } }
                }
                .onChange(of: viewModel.operationInFlight) { _, updating in
                    if updating { withAnimation { proxy.scrollTo("esim-top", anchor: .top) } }
                }
        }
        .background(Color(.systemGroupedBackground))
        .navigationTitle("eSIM 配置文件")
        .navigationBarTitleDisplayMode(.inline)
        .refreshable { await viewModel.refresh() }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button { viewModel.resetDownloadResult(); adding = true } label: { Image(systemName: "plus") }
                    .disabled(viewModel.blocksSwitching).accessibilityLabel("添加 eSIM")
            }
            ToolbarItem(placement: .topBarTrailing) {
                Button(action: refresh) {
                    if viewModel.isLoading { ProgressView() }
                    else { Image(systemName: "arrow.clockwise") }
                }
                .disabled(viewModel.isLoading || viewModel.isSwitching || viewModel.operationInFlight)
                .accessibilityLabel(viewModel.download.isBusy ? "Refresh download status" : "刷新配置文件和连接")
            }
        }
        .sheet(isPresented: $adding, onDismiss: {
            if let profile = addedSwitch { addedSwitch = nil; viewModel.requestSwitch(to: profile) }
        }) {
            RouterESimAddView(viewModel: viewModel, switchAfterDismiss: { addedSwitch = $0 })
        }
        .task {
            isVisible = true
            await viewModel.refresh()
            if !Task.isCancelled && scenePhase == .active { viewModel.startMonitoring() }
        }
        .onDisappear { isVisible = false; viewModel.stopMonitoring() }
        .onChange(of: scenePhase) { _, phase in
            if phase == .active && isVisible { viewModel.startMonitoring() }
            else { viewModel.stopMonitoring() }
        }
        .onChange(of: viewModel.message) { _, message in
            if message?.localizedCaseInsensitiveContains("a different eSIM card") == true { retryProfiles() }
        }
        .onChange(of: viewModel.downloadNote) { _, note in
            if note?.localizedCaseInsensitiveContains("a different eSIM card") == true { retryProfiles() }
        }
        .sheet(item: $viewModel.pendingRename) { profile in
            ESimRenameSheet(profile: profile, text: $viewModel.renameText, blocked: viewModel.blocksSwitching) { name in
                Task { await viewModel.confirmRename(profile, to: name) }
            }
        }
        .confirmationDialog(
            viewModel.pendingDelete.map { "Delete \($0.displayName) permanently?" } ?? "Delete profile?",
            isPresented: Binding(get: { viewModel.pendingDelete != nil }, set: { if !$0 { viewModel.pendingDelete = nil } }),
            titleVisibility: .visible, presenting: viewModel.pendingDelete
        ) { profile in
            Button("删除\(profile.displayName)", role: .destructive) { Task { await viewModel.confirmDelete(profile) } }
            Button("取消", role: .cancel) {}
        } message: { profile in
            Text("这无法撤销。您需要 \(profile.provider.isEmpty ? profile.displayName : profile.provider) 的新二维码才能取回此 eSIM。")
        }
    }

    private var screenContent: some View {
        Group {
            Color.clear.frame(height: 0).id("esim-top")
            if !viewModel.hasLoaded && viewModel.profiles.isEmpty {
                loadingCard
            } else {
                noticeCards
                if !viewModel.profiles.isEmpty || viewModel.switchingTo != nil { connectionCard }
                inventoryHeader
                if let listError = viewModel.listError {
                    ESimListErrorCard(text: listError, canRetry: !viewModel.blocksSwitching, retry: retryProfiles)
                }
                if viewModel.profiles.isEmpty {
                    if viewModel.message == nil && viewModel.listError == nil && !viewModel.download.isBusy { emptyCard }
                } else {
                    profilesContent
                }
                if let pending = viewModel.pendingReceipts, pending > 0 {
                    ESimReceiptsCard(count: pending, sending: viewModel.receiptsSending,
                                     canSend: !viewModel.blocksSwitching) {
                        Task { await viewModel.sendReceipts() }
                    }
                }
                if viewModel.chip != .empty { ESimChipCard(chip: viewModel.chip) }
            }
        }
        .listRowInsets(EdgeInsets(top: 8, leading: 16, bottom: 8, trailing: 16))
        .listRowSeparator(.hidden)
        .listRowBackground(Color.clear)
    }

    private func dialogs<Content: View>(_ content: Content) -> some View {
        content
        .confirmationDialog(
            viewModel.pendingSwitch.map { "Switch to \($0.displayName)?" } ?? "Switch profile?",
            isPresented: Binding(get: { viewModel.pendingSwitch != nil },
                                 set: { if !$0 { viewModel.pendingSwitch = nil } }),
            titleVisibility: .visible,
            presenting: viewModel.pendingSwitch
        ) { profile in
            Button("切换至\(profile.displayName)") { Task { await viewModel.confirmSwitch(to: profile) } }
            Button("取消", role: .cancel) {}
        } message: { profile in
            Text((["路由器将断开 \(viewModel.enabledProfile?.displayName ?? "the current profile") 并连接 \(profile.displayName)。互联网将暂停约一分钟，Wi-Fi 保持开启。"]
                  + viewModel.inventory.switchWarnings(for: profile)).joined(separator: "\n\n"))
        }
    }

    @ViewBuilder
    private var noticeCards: some View {
        if let message = viewModel.message {
            ESimMessageCard(message: message, isError: viewModel.messageIsError,
                            canRetry: !viewModel.blocksSwitching, retry: retryProfiles)
        }
        if viewModel.operationInFlight {
            CardView { HStack(spacing: 12) { ProgressView(); Text("更新eSIM…").font(.subheadline); Spacer() } }
        }
        if viewModel.download.state != "idle" {
            Button { adding = true } label: {
                CardView(padding: 16) {
                    HStack(spacing: 12) {
                        Image(systemName: viewModel.download.isBusy ? "arrow.down.circle" : "simcard").font(.title2)
                        VStack(alignment: .leading, spacing: 4) {
                            Text(viewModel.download.progressText).font(.subheadline.weight(.semibold))
                            Text("点击查看进度和后续步骤").font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer(minLength: 0)
                        Image(systemName: "chevron.right").font(.caption)
                    }.foregroundStyle(.primary)
                }
            }.buttonStyle(.plain)
        }
    }

    private var connectionCard: some View {
        ESimConnectionCard(phase: viewModel.connectionPhase, target: viewModel.switchingTo,
                           operatorName: viewModel.operatorName, networkType: viewModel.networkType,
                           signalBars: viewModel.signalBars, note: viewModel.connectionNote,
                           checkConnection: refresh)
    }

    private var inventoryHeader: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                sectionHeading("在您的 eSIM 上")
                Spacer()
                if viewModel.inventory.updating || viewModel.connectionPhase.blocksSwitching {
                    HStack(spacing: 5) {
                        ProgressView().controlSize(.mini)
                        Text("更新中……").font(.caption)
                    }.foregroundStyle(.secondary).accessibilityElement(children: .combine)
                } else {
                    Text(viewModel.listError != nil && viewModel.profiles.isEmpty ? "Not read yet" : (viewModel.profiles.count == 1 ? "1 profile" : "\(viewModel.profiles.count) profiles")).font(.caption).foregroundStyle(.secondary)
                }
            }
            if viewModel.inventory.observedAt > 0 {
                Text("最后读取 \(Date(timeIntervalSince1970: TimeInterval(viewModel.inventory.observedAt)).formatted(date: .omitted, time: .shortened))")
                    .font(.caption2).foregroundStyle(.secondary)
            }
        }.padding(.horizontal, 2)
    }

    private var profilesContent: some View {
        Group {
            if let active = viewModel.enabledProfile { profileCard(active) }
            if !otherProfiles.isEmpty {
                if viewModel.enabledProfile != nil {
                    sectionHeading("切换到其他配置文件").padding(.top, 4)
                }
                ForEach(otherProfiles) { profile in profileCard(profile) }
            }
            Button { viewModel.resetDownloadResult(); adding = true } label: {
                Label("添加 eSIM", systemImage: "plus.circle.fill").frame(maxWidth: .infinity, minHeight: 32)
            }.buttonStyle(.bordered).disabled(viewModel.blocksSwitching)

            VStack(alignment: .leading, spacing: 8) {
                Label("向左滑动查看配置文件选项。向右滑动即可切换。", systemImage: "hand.draw")
                Label("切换会使互联网暂停大约一分钟。您的路由器的 Wi-Fi 保持开启状态。", systemImage: "wifi")
            }.font(.caption).foregroundStyle(.secondary).padding(.horizontal, 4)
        }
    }

    private func profileCard(_ profile: RouterESimProfile) -> some View {
        ESimProfileCard(profile: profile, canSwitch: !viewModel.blocksSwitching,
                        onSwitch: { viewModel.requestSwitch(to: profile) },
                        onRename: { viewModel.requestRename(profile) },
                        onDelete: { viewModel.requestDelete(profile) })
            .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                if !viewModel.blocksSwitching {
                    if !profile.isEnabled {
                        // The confirmation dialog owns the destructive action. Keeping this button
                        // non-destructive prevents List from removing the row before confirmation.
                        Button("删除", systemImage: "trash") { viewModel.requestDelete(profile) }.tint(.red)
                    }
                    Button("重命名", systemImage: "pencil") { viewModel.requestRename(profile) }.tint(.blue)
                }
            }
            .swipeActions(edge: .leading, allowsFullSwipe: false) {
                if profile.canEnable && !viewModel.blocksSwitching {
                    Button("开关", systemImage: "arrow.triangle.2.circlepath") { viewModel.requestSwitch(to: profile) }.tint(Color.eSimAccent)
                }
            }
            .contextMenu {
                Button("重命名", systemImage: "pencil") { viewModel.requestRename(profile) }.disabled(viewModel.blocksSwitching)
                if !profile.isEnabled {
                    Button("删除配置文件", systemImage: "trash", role: .destructive) { viewModel.requestDelete(profile) }.disabled(viewModel.blocksSwitching)
                }
            }
            .accessibilityAction(named: "重命名") { viewModel.requestRename(profile) }
    }

    private func sectionHeading(_ title: String) -> some View {
        Text(title).font(.subheadline.weight(.semibold)).foregroundStyle(.secondary)
            .accessibilityAddTraits(.isHeader)
    }

    private var loadingCard: some View {
        CardView {
            HStack(spacing: 14) {
                ProgressView()
                VStack(alignment: .leading, spacing: 4) {
                    Text("阅读简介").font(.headline)
                    Text("检查路由器中的 eSIM...").font(.subheadline).foregroundStyle(.secondary)
                }
                Spacer(minLength: 0)
            }
            .padding(.vertical, 12)
        }
    }

    private var emptyCard: some View {
        CardView {
            VStack(spacing: 14) {
                Image(systemName: "simcard.2").font(.largeTitle).foregroundStyle(.secondary)
                    .padding(16).background(.secondary.opacity(0.08), in: .rect(cornerRadius: 16))
                Text("还没有配置文件").font(.headline)
                Text("使用运营商提供的二维码添加一个。")
                    .font(.subheadline).foregroundStyle(.secondary).multilineTextAlignment(.center)
                Button("添加 eSIM") { viewModel.resetDownloadResult(); adding = true }.buttonStyle(.borderedProminent).tint(Color.eSimAccent).disabled(viewModel.blocksSwitching)
            }
            .frame(maxWidth: .infinity).padding(.vertical, 18)
        }
    }

    private func refresh() { Task { await viewModel.refresh() } }
    private func retryProfiles() { Task { await viewModel.retryProfiles() } }
}

private struct ESimProfileCard: View {
    let profile: RouterESimProfile
    let canSwitch: Bool
    let onSwitch: () -> Void
    let onRename: () -> Void
    let onDelete: () -> Void

    private var subtitle: String {
        if !profile.provider.isEmpty && profile.provider.caseInsensitiveCompare(profile.displayName) != .orderedSame { return profile.provider }
        if !profile.name.isEmpty && profile.name.caseInsensitiveCompare(profile.displayName) != .orderedSame { return profile.name }
        return profile.isEnabled ? "Selected on your router" : "准备切换"
    }

    var body: some View {
        HStack(spacing: 0) {
            if profile.canEnable {
                Button(action: onSwitch) { content }.buttonStyle(.plain).disabled(!canSwitch)
                    .accessibilityHint("切换配置文件，互联网将暂停约一分钟，Wi-Fi 保持开启。")
            } else { content }
            Menu {
                Button("重命名", systemImage: "pencil", action: onRename)
                if !profile.isEnabled { Button("删除配置文件", systemImage: "trash", role: .destructive, action: onDelete) }
            } label: {
                Image(systemName: "ellipsis").font(.body.weight(.semibold))
                    .frame(minWidth: 44, minHeight: 44).contentShape(.rect)
            }
            .disabled(!canSwitch).accessibilityLabel("\(profile.displayName) 的选项")
            .padding(.trailing, 6)
        }
        .background(profile.isEnabled ? Color.eSimAccent.opacity(0.08) : Color(.secondarySystemGroupedBackground), in: .rect(cornerRadius: 16))
        .overlay { RoundedRectangle(cornerRadius: 16).strokeBorder(profile.isEnabled ? Color.eSimAccent.opacity(0.35) : Color.primary.opacity(0.07), lineWidth: 1) }
    }

    private var content: some View {
        HStack(alignment: .center, spacing: 12) {
            Image(systemName: profile.isEnabled ? "simcard.fill" : "simcard")
                .font(.title2).foregroundStyle(profile.isEnabled ? Color.eSimAccent : .blue)
                .frame(width: 44, height: 50)
                .background((profile.isEnabled ? Color.eSimAccent : .blue).opacity(0.1), in: .rect(cornerRadius: 12))
            VStack(alignment: .leading, spacing: 5) {
                if profile.isEnabled {
                    HStack(alignment: .center, spacing: 4) {
                        ESimCheckmark(diameter: 13)
                        Text("已启用")
                    }.font(.caption2.weight(.semibold)).foregroundStyle(Color.eSimAccent)
                }
                Text(profile.displayName.isEmpty ? "Unnamed profile" : profile.displayName)
                    .font(.body.weight(.semibold)).foregroundStyle(.primary).fixedSize(horizontal: false, vertical: true)
                Text(subtitle).font(.caption).foregroundStyle(.secondary)
                Text("…\(profile.iccid.suffix(4))").font(.caption2.monospacedDigit()).foregroundStyle(.secondary)
            }.frame(maxWidth: .infinity, alignment: .leading)
            if profile.canEnable {
                Image(systemName: "arrow.triangle.2.circlepath").foregroundStyle(canSwitch ? .blue : .secondary).accessibilityHidden(true)
            } else if !profile.isEnabled {
                Image(systemName: "lock.fill").foregroundStyle(.secondary).accessibilityLabel("不可用")
            }
        }
        .padding(14).contentShape(.rect).accessibilityElement(children: .combine)
    }
}

private struct ESimCheckmark: View {
    let diameter: CGFloat

    var body: some View {
        Circle().fill(Color.eSimAccent)
            .frame(width: diameter, height: diameter)
            .overlay(alignment: .center) {
                Image(systemName: "checkmark")
                    .resizable().scaledToFit()
                    .frame(width: diameter * 0.52, height: diameter * 0.52)
                    .fontWeight(.bold)
                    .foregroundStyle(Color(.secondarySystemGroupedBackground))
            }
            .accessibilityHidden(true)
    }
}

private struct ESimRenameSheet: View {
    let profile: RouterESimProfile
    @Binding var text: String
    let blocked: Bool
    let save: (String) -> Void
    @Environment(\.dismiss) private var dismiss
    @FocusState private var focused: Bool
    private var trimmed: String { text.trimmingCharacters(in: .whitespacesAndNewlines) }
    private var byteCount: Int { trimmed.utf8.count }
    private var valid: Bool { byteCount <= 64 && text.rangeOfCharacter(from: .newlines) == nil }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Label(profile.displayName, systemImage: "simcard").font(.headline)
                    TextField(profile.name.isEmpty ? profile.provider : profile.name, text: $text)
                        .focused($focused).submitLabel(.done).onSubmit { submit() }
                } header: { Text("昵称") } footer: {
                    VStack(alignment: .leading, spacing: 6) {
                        HStack {
                            Text(valid ? "留空则使用运营商名称。" : "请使用单行文本，最多 64 字节。")
                            Spacer()
                            Text("\(byteCount) / 64 bytes").monospacedDigit().fixedSize()
                        }.foregroundStyle(valid ? Color.secondary : .red)
                        Text("名称中的表情符号或重音字母每个字符可能占用多个字节。")
                    }
                }
            }
            .navigationTitle("重命名配置文件").navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("取消") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) { Button("保存", action: submit).disabled(!valid || blocked || trimmed == profile.nickname) }
            }
            .task { focused = true }
        }
        .presentationDetents([.medium, .large]).presentationDragIndicator(.visible)
    }
    private func submit() {
        guard valid, !blocked, trimmed != profile.nickname else { return }
        let name = trimmed
        dismiss()
        save(name)
    }
}

private struct ESimConnectionCard: View {
    let phase: RouterESimConnectionPhase
    let target: String?
    let operatorName: String
    let networkType: String
    let signalBars: Int
    let note: String?
    let checkConnection: () -> Void

    private var isProgress: Bool { phase.blocksSwitching }
    private var needsAttention: Bool { phase == .waiting || phase == .unavailable }
    private var accent: Color { needsAttention ? .orange : Color.eSimAccent }
    private var title: String {
        switch phase {
        case .checking: "正在检查连接"
        case .switching: "正在切换配置文件"
        case .reloading: "正在重新加载 SIM"
        case .registering: "正在注册网络"
        case .connected: "移动数据已连接"
        case .disconnected: "移动数据未连接"
        case .waiting: "连接需要处理"
        case .unavailable: target == nil ? "连接状态不可用" : "切换需要处理"
        }
    }
    private var detail: String {
        if let note { return note }
        switch phase {
        case .switching: return "在 eSIM 上选择新的配置文件。"
        case .reloading: return "调制解调器正在加载 SIM。该卡可能在 5-25 秒内无法读取。"
        case .registering: return "新的配置文件已加载。等待移动数据连接。"
        case .waiting: return "移动数据尚未连接。检查计划或承保范围，或选择下面的其他配置文件。"
        case .disconnected: return "您的路由器的 Wi-Fi 保持连接。您仍然可以切换配置文件。"
        case .checking: return "正在读取路由器实时状态。"
        case .unavailable: return "再次查看路由器的连接状态。"
        case .connected: return "移动数据已上。交通可能还需要几秒钟。"
        }
    }
    private var currentStep: Int {
        if phase == .switching || phase == .unavailable { return 0 }
        if phase == .reloading { return 1 }
        return 2
    }
    private var announcement: String { "\(title). \(detail)" }

    var body: some View {
        CardView(padding: 16) {
            VStack(alignment: .leading, spacing: 16) {
                HStack(alignment: .top, spacing: 12) {
                    ZStack {
                        RoundedRectangle(cornerRadius: 12).fill(accent.opacity(0.1)).frame(width: 44, height: 44)
                        if isProgress || phase == .checking { ProgressView().tint(accent) }
                        else if phase == .connected { ESimCheckmark(diameter: 22) }
                        else { Image(systemName: needsAttention ? "exclamationmark.circle" : "antenna.radiowaves.left.and.right.slash").font(.title3).foregroundStyle(accent) }
                    }.accessibilityHidden(true)
                    VStack(alignment: .leading, spacing: 4) {
                        if let target { Text("切换到\(target)").font(.caption).foregroundStyle(.secondary) }
                        Text(title).font(.headline)
                        if phase == .connected {
                            Text([operatorName, networkType].filter { !$0.isEmpty }.joined(separator: " · ")).font(.caption).foregroundStyle(.secondary)
                        }
                    }.frame(maxWidth: .infinity, alignment: .leading)
                    if phase == .connected {
                        Image(systemName: "cellularbars", variableValue: Double(signalBars) / 5)
                            .foregroundStyle(Color.eSimAccent).accessibilityLabel("信号 \(signalBars) 为 5")
                    }
                }
                if target != nil && (isProgress || phase == .connected) {
                    VStack(alignment: .leading, spacing: 0) {
                        step("切换配置文件", icon: "simcard", index: 0)
                        step("重新加载SIM", icon: "arrow.clockwise", index: 1)
                        step("连接到网络", icon: "antenna.radiowaves.left.and.right", index: 2)
                    }
                }
                if phase != .connected || target != nil {
                    Text(detail).font(.subheadline).foregroundStyle(.secondary).textSelection(.enabled)
                        .fixedSize(horizontal: false, vertical: true)
                }
                if isProgress {
                    Label("Wi-Fi 保持连接 · 通常 15–45 秒", systemImage: "wifi")
                        .font(.caption).foregroundStyle(.secondary)
                }
                if needsAttention { Button("再次检查", action: checkConnection).buttonStyle(.bordered) }
            }
        }
        .accessibilityElement(children: .contain)
        .onChange(of: announcement) { _, text in UIAccessibility.post(notification: .announcement, argument: text) }
    }

    private func step(_ label: String, icon: String, index: Int) -> some View {
        let complete = phase == .connected || index < currentStep
        let current = index == currentStep && phase != .connected
        let color: Color = complete ? Color.eSimAccent : (current ? accent : .secondary)
        return HStack(spacing: 10) {
            Circle().fill(color.opacity(current || complete ? 0.12 : 0.05))
                .frame(width: 28, height: 28)
                .overlay(alignment: .center) {
                    Image(systemName: complete ? "checkmark" : (current && needsAttention ? "exclamationmark" : icon))
                        .font(.caption.weight(.semibold))
                        .frame(width: 16, height: 16, alignment: .center)
                }
            Text(label).font(.subheadline.weight(current ? .semibold : .regular))
            Spacer()
            if current && isProgress { Text("进行中").font(.caption2) }
            if complete { Text("完成").font(.caption2) }
        }
        .foregroundStyle(color).padding(.vertical, 5)
        .accessibilityElement(children: .combine)
    }
}

private struct ESimMessageCard: View {
    let message: String
    let isError: Bool
    let canRetry: Bool
    let retry: () -> Void
    private var housekeeping: Bool { message.localizedCaseInsensitiveContains("receipts") }
    /// A switch or delete that found the card still in use: a notice, not a result.
    private var busy: Bool { message == RouterESimViewModel.busyText }
    private var guidance: String? {
        let text = message.lowercased()
        if text.contains("agent password required") { return "在代理设置中设置路由器代理密码，然后重试您的操作。" }
        if text.contains("airplane") || text.contains("low-power") { return "打开路由器设置→移动网络检查调制解调器模式。" }
        if text.contains("different esim card") { return "现在在路由器中重新加载卡上的配置文件。重试之前请查看该列表。" }
        if text.contains("404") || text.contains("not found") { return "此操作可能需要更新的路由器代理。您保存的配置文件未更改。" }
        return nil
    }
    var body: some View {
        CardView(padding: 16) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: busy ? "hourglass" : housekeeping ? "paperplane" : (isError ? "exclamationmark.circle" : "checkmark.circle.fill"))
                    .font(.title3).foregroundStyle(isError && !housekeeping ? .orange : Color.eSimAccent)
                VStack(alignment: .leading, spacing: 8) {
                    Text(busy ? "eSIM in use" : housekeeping ? "运营商回执" : (isError ? "暂时无法完成此操作" : "配置文件已更新")).font(.subheadline.weight(.semibold))
                    Text(message).font(.subheadline).foregroundStyle(.secondary).textSelection(.enabled)
                    if let guidance { Text(guidance).font(.caption).foregroundStyle(.secondary) }
                    if isError && !housekeeping {
                        Button("重新加载配置文件", action: retry).buttonStyle(.bordered).disabled(!canRetry)
                        Text("重新加载仅读取列表。它不会重复您的操作。").font(.caption2).foregroundStyle(.secondary)
                    }
                }.frame(maxWidth: .infinity, alignment: .leading)
            }
        }
        .accessibilityElement(children: .contain)
        .onChange(of: message, initial: true) { _, text in UIAccessibility.post(notification: .announcement, argument: text) }
    }
}

private struct ESimListErrorCard: View {
    let text: String
    let canRetry: Bool
    let retry: () -> Void
    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label("无法刷新配置文件", systemImage: "arrow.clockwise.circle").font(.subheadline.weight(.semibold))
            Text(text).font(.caption).foregroundStyle(.secondary).textSelection(.enabled)
            Button("重试", action: retry).font(.caption.weight(.semibold)).buttonStyle(.bordered).disabled(!canRetry)
        }
        .frame(maxWidth: .infinity, alignment: .leading).padding(14)
        .background(.orange.opacity(0.07), in: .rect(cornerRadius: 12))
    }
}

private struct ESimReceiptsCard: View {
    let count: Int
    let sending: Bool
    let canSend: Bool
    let send: () -> Void
    @State private var expanded = false
    var body: some View {
        CardView(padding: 16) {
            VStack(alignment: .leading, spacing: 12) {
                HStack(spacing: 12) {
                    Image(systemName: "paperplane").foregroundStyle(.blue).font(.title3)
                        .frame(width: 40, height: 40).background(.blue.opacity(0.08), in: .rect(cornerRadius: 10))
                    VStack(alignment: .leading, spacing: 3) {
                        Text("运营商回执").font(.subheadline.weight(.semibold))
                        Text(sending ? "Sending…" : "\(count) waiting · Retried automatically").font(.caption).foregroundStyle(.secondary)
                    }.frame(maxWidth: .infinity, alignment: .leading)
                    if sending { ProgressView().accessibilityLabel("发送运营商回执") }
                }
                DisclosureGroup("为什么回执仍在等待？", isExpanded: $expanded) {
                    Text("添加、切换或删除配置文件时，您的 eSIM 会向运营商发送回执。连接移动数据时，路由器每 10 分钟重试一次。您不需要保持此屏幕打开。")
                        .font(.caption).foregroundStyle(.secondary).padding(.top, 6)
                }.font(.caption)
                Button(action: send) { Label(sending ? "Sending receipts…" : "立即发送", systemImage: "arrow.up.circle") }
                    .buttonStyle(.bordered).font(.caption.weight(.semibold)).disabled(!canSend || sending)
            }
        }
    }
}

private struct ESimChipCard: View {
    let chip: RouterESimChip
    @Environment(\.scenePhase) private var scenePhase
    @State private var expanded = false
    @State private var revealed = false
    @State private var copied = false

    var body: some View {
        CardView(padding: 14) {
            VStack(alignment: .leading, spacing: 12) {
                Button { expanded.toggle() } label: {
                    HStack(spacing: 10) {
                        Image(systemName: "simcard.2").foregroundStyle(.blue)
                        VStack(alignment: .leading, spacing: 3) {
                            Text("eSIM 卡").font(.subheadline.weight(.semibold)).foregroundStyle(.primary)
                            Text("卡片详情与标识符").font(.caption).foregroundStyle(.secondary)
                        }
                        Spacer()
                        Image(systemName: expanded ? "chevron.up" : "chevron.down").font(.caption.weight(.semibold)).foregroundStyle(.secondary)
                    }.frame(minHeight: 32).contentShape(.rect)
                }.buttonStyle(.plain).accessibilityValue(expanded ? "Expanded" : "Collapsed")
                if expanded {
                    Divider()
                    if !chip.firmware.isEmpty { LabeledContent("固件", value: chip.firmware).font(.caption) }
                    if let free = chip.freeMemoryBytes {
                        LabeledContent("可用内存", value: ByteCountFormatter.string(fromByteCount: Int64(max(0, free)), countStyle: .memory)).font(.caption)
                    }
                    if !chip.eid.isEmpty {
                        VStack(alignment: .leading, spacing: 6) {
                            HStack {
                                Label("EID", systemImage: "lock.shield").font(.caption).foregroundStyle(.secondary)
                                Spacer()
                                Button { revealed.toggle(); copied = false } label: {
                                    Image(systemName: revealed ? "eye.slash" : "eye").frame(minWidth: 44, minHeight: 44)
                                }.accessibilityLabel(revealed ? "Hide EID" : "揭示EID")
                                if revealed {
                                    Button {
                                        UIPasteboard.general.string = chip.eid
                                        copied = true
                                    } label: {
                                        Image(systemName: copied ? "checkmark" : "doc.on.doc").frame(minWidth: 44, minHeight: 44)
                                    }.accessibilityLabel(copied ? "EID copied" : "Copy EID")
                                }
                            }
                            if revealed { Text(chip.eid).font(.caption.monospaced()).textSelection(.enabled).fixedSize(horizontal: false, vertical: true) }
                            else { Text("•••• •••• •••• …\(chip.eid.suffix(4))").font(.caption.monospaced()).foregroundStyle(.secondary) }
                        }
                    }
                }
            }
        }
        .onChange(of: scenePhase) { _, phase in if phase != .active { revealed = false; copied = false } }
        .onChange(of: expanded) { _, value in if !value { revealed = false; copied = false } }
    }
}
