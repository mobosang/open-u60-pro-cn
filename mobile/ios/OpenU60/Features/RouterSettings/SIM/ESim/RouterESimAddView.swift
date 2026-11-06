import SwiftUI
import PhotosUI

struct RouterESimAddView: View {
    @Bindable var viewModel: RouterESimViewModel
    let switchAfterDismiss: (RouterESimProfile) -> Void
    @Environment(\.dismiss) private var dismiss
    @Environment(\.scenePhase) private var scenePhase
    @State private var code = ""
    @State private var confirmationCode = ""
    @State private var inputError: String?
    @State private var manual = false
    @State private var scanner = false
    @State private var photo: PhotosPickerItem?
    @State private var readingPhoto = false
    private var parsed: RouterESimActivationCode? { RouterESimActivationCode.parse(code) }
    private var showsProgress: Bool { !["idle", "cancelled"].contains(viewModel.download.state) }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    if showsProgress || viewModel.operationInFlight { downloadContent } else { entryContent }
                }.padding(20)
            }
            .background(Color(.systemGroupedBackground))
            .navigationTitle("添加 eSIM").navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .topBarTrailing) { Button(viewModel.download.isBusy ? "关闭" : "完成") { clearSecrets(); dismiss() } } }
            .sheet(isPresented: $scanner) {
                NavigationStack {
                    RouterESimScanner { value in accept(value); scanner = false }
                        .ignoresSafeArea(edges: .bottom).navigationTitle("扫描eSIM二维码").navigationBarTitleDisplayMode(.inline)
                        .toolbar { ToolbarItem(placement: .topBarTrailing) { Button("关闭") { scanner = false } } }
                }
            }
            .task { viewModel.startMonitoring() }
            .onChange(of: scenePhase) { _, phase in
                if phase == .active { viewModel.startMonitoring() }
                else { viewModel.stopMonitoring() }
            }
            .task(id: photo) {
                guard let selection = photo else { return }
                readingPhoto = true
                defer { readingPhoto = false; photo = nil }
                do {
                    guard let data = try await selection.loadTransferable(type: Data.self) else { throw CocoaError(.fileReadCorruptFile) }
                    let values = try await RouterESimScanner.codes(in: data)
                    guard !Task.isCancelled else { return }
                    let valid = Array(Set(values.filter { RouterESimActivationCode.parse($0) != nil }))
                    if valid.count == 1 { accept(valid[0]); UINotificationFeedbackGenerator().notificationOccurred(.success) }
                    else { inputError = valid.isEmpty ? RouterESimActivationCode.invalidMessage : "选择一张带有eSIM二维码的照片。" }
                } catch {
                    guard !Task.isCancelled else { return }
                    inputError = "无法阅读这张照片。选择其他图像或手动输入代码。"
                }
            }
            .onChange(of: viewModel.download.state) { _, state in
                if state == "done" { clearSecrets() }
                if state == "cancelled" { viewModel.resetDownloadResult() }
            }
            .onDisappear { clearSecrets(); photo = nil }
            .privacySensitive()
        }
    }

    private var entryContent: some View {
        VStack(alignment: .leading, spacing: 20) {
            VStack(alignment: .leading, spacing: 10) {
                Image(systemName: "qrcode.viewfinder").font(.largeTitle).foregroundStyle(.teal)
                Text("将配置文件添加到您的路由器").font(.title2.bold())
                Text("使用运营商提供的 eSIM 二维码。您将在安装之前查看配置文件。").font(.subheadline).foregroundStyle(.secondary)
            }
            if viewModel.connectionPhase == .disconnected || viewModel.connectionPhase == .waiting {
                Label("您的路由器没有移动数据连接。下载前先连接；无法使用您手机本身的数据。", systemImage: "wifi.exclamationmark")
                    .font(.subheadline).foregroundStyle(.secondary).padding(14)
                    .background(.orange.opacity(0.08), in: .rect(cornerRadius: 12))
            }
            CardView(padding: 16) {
                VStack(spacing: 12) {
                    Button { scanner = true } label: { Label("扫描二维码", systemImage: "camera.fill").frame(maxWidth: .infinity, minHeight: 30) }
                        .buttonStyle(.borderedProminent).tint(.teal)
                    PhotosPicker(selection: $photo, matching: .images) {
                        Label(readingPhoto ? "Reading photo…" : "从照片中选择二维码", systemImage: "photo").frame(maxWidth: .infinity, minHeight: 30)
                    }.buttonStyle(.bordered).disabled(readingPhoto)
                    Button { manual = true } label: { Label("手动输入代码", systemImage: "keyboard").frame(maxWidth: .infinity, minHeight: 30) }.buttonStyle(.borderless)
                }
            }
            if manual || !code.isEmpty {
                CardView(padding: 16) {
                    VStack(alignment: .leading, spacing: 12) {
                        Label("激活码", systemImage: "lock.shield").font(.subheadline.weight(.semibold))
                        SecureField("LPA:1$…", text: $code).textInputAutocapitalization(.never).autocorrectionDisabled().textFieldStyle(.roundedBorder)
                            .onChange(of: code) { _, _ in inputError = nil; confirmationCode = "" }
                        if let parsed {
                            Label("代码准备好", systemImage: "checkmark.circle.fill").font(.caption).foregroundStyle(.teal)
                            Text("来自\(parsed.server)").font(.caption).foregroundStyle(.secondary)
                        }
                        if parsed?.needsConfirmationCode == true {
                            SecureField("运营商确认码", text: $confirmationCode).textInputAutocapitalization(.never).autocorrectionDisabled().textFieldStyle(.roundedBorder)
                            Text("输入运营商提供的单独确认码（最多 64 个字符）。").font(.caption).foregroundStyle(.secondary)
                        }
                        Button("阅读配置文件详细信息") {
                            guard let parsed else { inputError = RouterESimActivationCode.invalidMessage; return }
                            guard !parsed.needsConfirmationCode || RouterESimActivationCode.validConfirmationCode(confirmationCode) else {
                                inputError = "请输入运营商提供的确认码（最多 64 个字符）。"; return
                            }
                            Task { await viewModel.startDownload(code: parsed.value, confirmationCode: confirmationCode) }
                        }.buttonStyle(.borderedProminent).tint(.teal)
                            .disabled(code.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || viewModel.blocksSwitching || readingPhoto)
                    }
                }
            }
            if let inputError { Label(inputError, systemImage: "exclamationmark.circle").font(.subheadline).foregroundStyle(.orange) }
            if let note = viewModel.downloadNote {
                CardView {
                    VStack(alignment: .leading, spacing: 8) {
                        Label("无法开始此下载", systemImage: "exclamationmark.circle").font(.subheadline.weight(.semibold))
                        Text(note).font(.subheadline).foregroundStyle(.secondary).textSelection(.enabled)
                        if note.localizedCaseInsensitiveContains("agent password required") {
                            Text("在代理设置中设置路由器代理密码，然后重试。").font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }
            }
            Label("您的路由器需要互联网来下载配置文件。保持 Wi-Fi 连接。下载通常需要 10-60 秒。", systemImage: "wifi").font(.caption).foregroundStyle(.secondary)
            Label("您的激活码是私人且一次性的。它仅发送到您的路由器，不会被 OpenU60 保存。", systemImage: "lock").font(.caption).foregroundStyle(.secondary)
        }
    }

    private var downloadContent: some View {
        CardView(padding: 20) {
            VStack(alignment: .leading, spacing: 18) {
                HStack(spacing: 14) {
                    if ["running", "cancelling"].contains(viewModel.download.state) || viewModel.operationInFlight { ProgressView().controlSize(.large) }
                    else { Image(systemName: viewModel.download.state == "confirm" ? "simcard" : (viewModel.download.state == "done" ? "checkmark.circle.fill" : "exclamationmark.circle")).font(.largeTitle).foregroundStyle(["done", "confirm"].contains(viewModel.download.state) ? .teal : .orange) }
                    Text(viewModel.download.progressText).font(.title3.weight(.semibold))
                }
                if viewModel.download.state == "confirm" {
                    Text("安装\(viewModel.download.displayName)？").font(.headline)
                    if let name = viewModel.download.name, name != viewModel.download.displayName { Text(name).font(.subheadline).foregroundStyle(.secondary) }
                    Text("该配置文件将安装在您路由器的 eSIM 上并保持非活动状态，直到您切换到它。 2分钟内确认。").font(.subheadline).foregroundStyle(.secondary)
                    Button("安装配置文件") { Task { await viewModel.confirmDownload() } }.buttonStyle(.borderedProminent).tint(.teal).disabled(viewModel.operationInFlight)
                } else if viewModel.download.state == "done" {
                    Text("添加到您的路由器。您当前的配置文件仍然有效。").font(.subheadline).foregroundStyle(.secondary)
                    if let profile = viewModel.profiles.first(where: { $0.iccid == viewModel.download.iccid }), profile.canEnable {
                        Button("立即切换") { switchAfterDismiss(profile); clearSecrets(); dismiss() }.buttonStyle(.borderedProminent).tint(.teal).disabled(viewModel.blocksSwitching)
                    } else {
                        Button("刷新配置文件") { Task { await viewModel.refresh() } }.disabled(viewModel.blocksSwitching)
                    }
                    Button("后来") { clearSecrets(); dismiss() }.buttonStyle(.bordered)
                } else if viewModel.download.state == "failed" {
                    Text(viewModel.download.error ?? "运营商无法添加此配置文件。").font(.subheadline).textSelection(.enabled)
                    Button("重试") { viewModel.resetDownloadResult(); manual = true }.buttonStyle(.borderedProminent)
                } else {
                    Text(viewModel.download.step == "es10b_load_bound_profile_package" ? "请保持路由器供电。此阶段无法取消安装。" : "请保持路由器联网。你可以关闭此页面，稍后返回查看进度。").font(.subheadline).foregroundStyle(.secondary)
                }
                if let note = viewModel.downloadNote {
                    Text(note).font(.subheadline).foregroundStyle(.orange).textSelection(.enabled)
                    Button("检查状态") { Task { await viewModel.retryDownloadStatus() } }.disabled(viewModel.operationInFlight)
                }
                if viewModel.download.canCancel {
                    Button("取消下载", role: .destructive) { Task { await viewModel.cancelDownload() } }.buttonStyle(.bordered).disabled(viewModel.operationInFlight)
                }
            }.frame(maxWidth: .infinity, alignment: .leading)
        }
    }
    private func accept(_ value: String) {
        guard let parsed = RouterESimActivationCode.parse(value) else { inputError = RouterESimActivationCode.invalidMessage; return }
        code = parsed.value; confirmationCode = ""; inputError = nil; manual = true
    }
    private func clearSecrets() { code = ""; confirmationCode = "" }
}
