import SwiftUI

struct VPNPassthroughView: View {
    @Bindable var viewModel: VPNPassthroughViewModel

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

            Section("VPN 穿透") {
                Toggle("L2TP", isOn: $viewModel.editL2tp)
                Toggle("PPTP", isOn: $viewModel.editPptp)
                Toggle("IP安全", isOn: $viewModel.editIpsec)
            }

            Section {
                Button {
                    Task { await viewModel.apply() }
                } label: {
                    Text("应用")
                        .frame(maxWidth: .infinity)
                }
                .disabled(viewModel.isLoading)
            }
        }
        .navigationTitle("VPN 穿透")
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
