import SwiftUI

struct SpeedTestView: View {
    /// Owned as `@State` so a re-evaluated `NavigationLink` destination cannot replace the view
    /// model (and lose the running test) while the screen is on screen.
    @State private var viewModel: SpeedTestViewModel

    init(viewModel: SpeedTestViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        List {
            if let msg = viewModel.message {
                Section {
                    Text(msg)
                        .font(.subheadline)
                        .foregroundStyle(viewModel.messageIsError ? .red : .green)
                }
            }

            Section("服务器") {
                Picker("服务器", selection: $viewModel.selectedServerId) {
                    Text("选择服务器").tag(nil as Int?)
                    ForEach(viewModel.servers) { server in
                        Text("\(server.sponsor) - \(server.name), \(server.country)")
                            .tag(server.id as Int?)
                    }
                }
                .disabled(viewModel.isRunning)
            }

            Section("控制") {
                if viewModel.isRunning {
                    HStack {
                        Text(phaseLabel)
                        Spacer()
                        Text("\(viewModel.progress.progress)%")
                            .foregroundStyle(.secondary)
                            .contentTransition(.numericText())
                            .animation(.default, value: viewModel.progress.progress)
                    }
                    ProgressView(value: Double(viewModel.progress.progress), total: 100)
                        .accessibilityLabel(phaseLabel)
                        .accessibilityValue("\(viewModel.progress.progress) percent")

                    HStack {
                        Spacer()
                        if viewModel.progress.phase == "download" {
                            Image(systemName: "arrow.down")
                                .font(.title)
                                .foregroundStyle(.blue)
                        } else if viewModel.progress.phase == "upload" {
                            Image(systemName: "arrow.up")
                                .font(.title)
                                .foregroundStyle(.orange)
                        }
                        Text(String(format: "%.1f", viewModel.progress.liveSpeedMbps))
                            .font(.system(size: 48, weight: .bold, design: .rounded))
                            .contentTransition(.numericText())
                            .animation(.default, value: viewModel.progress.liveSpeedMbps)
                        Text("兆比特/秒")
                            .font(.title3)
                            .foregroundStyle(.secondary)
                        Spacer()
                    }
                    .padding(.vertical, 8)
                    .accessibilityElement(children: .ignore)
                    .accessibilityLabel("当前速度")
                    .accessibilityValue(String(format: "%.1f megabits per second", viewModel.progress.liveSpeedMbps))

                    Button("停止测试", role: .destructive) {
                        Task { await viewModel.stopTest() }
                    }
                } else {
                    Button("启动速度测试") {
                        Task { await viewModel.startTest() }
                    }
                    .disabled(viewModel.isLoading || viewModel.selectedServerId == nil)
                }
            }

            if viewModel.progress.phase == "complete" {
                Section("结果") {
                    if let ping = viewModel.progress.pingMs {
                        LabeledContent("延迟测试", value: String(format: "%.1f ms", ping))
                    }
                    if let jitter = viewModel.progress.jitterMs {
                        LabeledContent("抖动", value: String(format: "%.1f ms", jitter))
                    }
                    if let download = viewModel.progress.downloadMbps {
                        LabeledContent("下载", value: String(format: "%.2f Mbps", download))
                    }
                    if let upload = viewModel.progress.uploadMbps {
                        LabeledContent("上传", value: String(format: "%.2f Mbps", upload))
                    }
                }

                Section("转账") {
                    LabeledContent("已下载", value: formatBytes(viewModel.progress.downloadBytes))
                    LabeledContent("已上传", value: formatBytes(viewModel.progress.uploadBytes))
                    if !viewModel.progress.server.isEmpty {
                        LabeledContent("服务器", value: viewModel.progress.server)
                    }
                }
            }
        }
        .navigationTitle("速度测试")
        .task {
            await viewModel.loadServers()
            await viewModel.resumeIfRunning()
        }
        .onDisappear { viewModel.stopPolling() }
        .overlay {
            if viewModel.isLoading && !viewModel.isRunning {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
    }

    private var phaseLabel: String {
        switch viewModel.progress.phase {
        case "latency": return "测试延迟..."
        case "download": return "正在下载..."
        case "upload": return "正在上传..."
        default: return viewModel.progress.phase.capitalized
        }
    }

    private func formatBytes(_ bytes: Int) -> String {
        if bytes < 1024 { return "\(bytes) B" }
        let kb = Double(bytes) / 1024
        if kb < 1024 { return String(format: "%.1f KB", kb) }
        let mb = kb / 1024
        if mb < 1024 { return String(format: "%.1f MB", mb) }
        let gb = mb / 1024
        return String(format: "%.2f GB", gb)
    }
}
