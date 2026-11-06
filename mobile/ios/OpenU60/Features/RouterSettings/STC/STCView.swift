import SwiftUI

struct STCView: View {
    @Bindable var viewModel: STCViewModel

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

            Section("状态") {
                LabeledContent("STC") {
                    Text(viewModel.config.enabled ? "Enabled" : "Disabled")
                        .foregroundStyle(viewModel.config.enabled ? .green : .secondary)
                }
            }

            Section("参数") {
                TextField("LTE 采集定时器", text: $viewModel.editLteTimer)
                    .keyboardType(.numberPad)
                TextField("NRSA 收集定时器", text: $viewModel.editNrsaTimer)
                    .keyboardType(.numberPad)
                TextField("LTE 白名单最大值", text: $viewModel.editLteMax)
                    .keyboardType(.numberPad)
                TextField("NRSA 白名单上限", text: $viewModel.editNrsaMax)
                    .keyboardType(.numberPad)

                Button {
                    Task { await viewModel.applyParams() }
                } label: {
                    Text("应用参数")
                        .frame(maxWidth: .infinity)
                }
                .disabled(viewModel.isLoading)
            }

            Section("控制") {
                Button("启用STC") {
                    Task { await viewModel.enable() }
                }
                .disabled(viewModel.isLoading || viewModel.config.enabled)

                Button("禁用 STC") {
                    Task { await viewModel.disable() }
                }
                .disabled(viewModel.isLoading || !viewModel.config.enabled)

                Button("重置白名单", role: .destructive) {
                    Task { await viewModel.reset() }
                }
                .disabled(viewModel.isLoading)
            }
        }
        .navigationTitle("智能基站连接")
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
}
