import SwiftUI

struct LANSpeedTestView: View {
    /// Owned as `@State` so a re-evaluated `NavigationLink` destination cannot replace the view
    /// model (and lose the running test) while the screen is on screen.
    @State private var viewModel: LANSpeedTestViewModel

    init(viewModel: LANSpeedTestViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        List {
            if let error = viewModel.error {
                Section {
                    Text(error)
                        .font(.subheadline)
                        .foregroundStyle(.red)
                }
            }

            Section("控制") {
                if viewModel.isRunning {
                    HStack {
                        Text(phaseLabel)
                        Spacer()
                        Text("\(Int(viewModel.progress * 100))%")
                            .foregroundStyle(.secondary)
                            .contentTransition(.numericText())
                            .animation(.default, value: viewModel.progress)
                    }
                    ProgressView(value: viewModel.progress)
                        .accessibilityLabel(phaseLabel)
                        .accessibilityValue("\(Int(viewModel.progress * 100)) percent")

                    if viewModel.phase == "download" || viewModel.phase == "upload" {
                        HStack {
                            Spacer()
                            if viewModel.phase == "download" {
                                Image(systemName: "arrow.down")
                                    .font(.title)
                                    .foregroundStyle(.blue)
                            } else {
                                Image(systemName: "arrow.up")
                                    .font(.title)
                                    .foregroundStyle(.orange)
                            }
                            Text(String(format: "%.1f", viewModel.liveSpeedMbps))
                                .font(.system(size: 48, weight: .bold, design: .rounded))
                                .contentTransition(.numericText())
                                .animation(.default, value: viewModel.liveSpeedMbps)
                            Text("兆比特/秒")
                                .font(.title3)
                                .foregroundStyle(.secondary)
                            Spacer()
                        }
                        .padding(.vertical, 8)
                        .accessibilityElement(children: .ignore)
                        .accessibilityLabel("当前速度")
                        .accessibilityValue(String(format: "%.1f megabits per second", viewModel.liveSpeedMbps))
                    }

                    Button("停止测试", role: .destructive) {
                        viewModel.stopTest()
                    }
                } else {
                    Button("开始 LAN 速度测试") {
                        viewModel.startTest()
                    }
                }
            }

            if viewModel.phase == "complete" {
                Section("结果") {
                    if let ping = viewModel.pingMs {
                        LabeledContent("延迟测试", value: String(format: "%.1f ms", ping))
                    }
                    if let download = viewModel.downloadMbps {
                        LabeledContent("下载", value: String(format: "%.1f Mbps", download))
                    }
                    if let upload = viewModel.uploadMbps {
                        LabeledContent("上传", value: String(format: "%.1f Mbps", upload))
                    }
                }
            }

            Section {
                Text("测量该设备和路由器之间的 WiFi 链路速度。不使用互联网数据。")
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
        }
        // No `onDisappear` teardown: a tab switch must not abort a transfer that is already
        // paid for. A real pop releases the model and its `deinit` cancels the run.
        .navigationTitle("局域网速度测试")
    }

    private var phaseLabel: String {
        switch viewModel.phase {
        case "ping": return "测试延迟..."
        case "download": return "正在下载..."
        case "upload": return "正在上传..."
        default: return viewModel.phase.capitalized
        }
    }
}
