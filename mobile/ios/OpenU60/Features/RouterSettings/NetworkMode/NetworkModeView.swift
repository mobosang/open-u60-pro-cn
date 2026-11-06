import SwiftUI

struct NetworkModeView: View {
    @Bindable var viewModel: NetworkModeViewModel

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

            Section("当前") {
                LabeledContent("网络模式", value: displayLabel(for: viewModel.config.netSelect, in: NetworkModeConfig.netSelectOptions))
            }

            Section("网络模式") {
                Picker("模式", selection: $viewModel.selectedNetSelect) {
                    ForEach(NetworkModeConfig.netSelectOptions, id: \.value) { option in
                        Text(option.label).tag(option.value)
                    }
                }
                .pickerStyle(.menu)
            }

            Section {
                Button {
                    Task { await viewModel.applyMode() }
                } label: {
                    Text("应用")
                        .frame(maxWidth: .infinity)
                }
                .disabled(viewModel.isLoading || viewModel.selectedNetSelect == viewModel.config.netSelect)
            }
        }
        .navigationTitle("网络模式")
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

    private func displayLabel(for value: String, in options: [(label: String, value: String)]) -> String {
        options.first(where: { $0.value == value })?.label ?? (value.isEmpty ? "—" : value)
    }
}
