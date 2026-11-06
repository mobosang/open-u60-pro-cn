import SwiftUI

struct CellLockView: View {
    @Bindable var viewModel: CellLockViewModel

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

            Section("当前小区") {
                LabeledContent("NR PCI", value: viewModel.status.nrPCI.isEmpty ? "—" : viewModel.status.nrPCI)
                LabeledContent("NR EARFCN", value: viewModel.status.nrEARFCN.isEmpty ? "—" : viewModel.status.nrEARFCN)
                LabeledContent("NR 频段", value: viewModel.status.nrBand.isEmpty ? "—" : viewModel.status.nrBand)
                LabeledContent("LTE PCI", value: viewModel.status.ltePCI.isEmpty ? "—" : viewModel.status.ltePCI)
                LabeledContent("LTE EARFCN", value: viewModel.status.lteEARFCN.isEmpty ? "—" : viewModel.status.lteEARFCN)
                LabeledContent("锁定") {
                    Text(viewModel.status.locked ? "Yes" : "No")
                        .foregroundStyle(viewModel.status.locked ? .orange : .green)
                }
            }

            Section("锁定 NR 单元") {
                TextField("PCI", text: $viewModel.nrPCI)
                    .keyboardType(.numberPad)
                TextField("EARFCN", text: $viewModel.nrEARFCN)
                    .keyboardType(.numberPad)
                TextField("频段（可选）", text: $viewModel.nrBand)
                    .keyboardType(.numberPad)
                Button("锁NR") {
                    Task { await viewModel.lockNR() }
                }
                .disabled(viewModel.isLoading || viewModel.nrPCI.isEmpty || viewModel.nrEARFCN.isEmpty)
            }

            Section("锁定 LTE 单元") {
                TextField("PCI", text: $viewModel.ltePCI)
                    .keyboardType(.numberPad)
                TextField("EARFCN", text: $viewModel.lteEARFCN)
                    .keyboardType(.numberPad)
                Button("锁LTE") {
                    Task { await viewModel.lockLTE() }
                }
                .disabled(viewModel.isLoading || viewModel.ltePCI.isEmpty || viewModel.lteEARFCN.isEmpty)
            }

            Section("邻居扫描") {
                Button {
                    Task { await viewModel.scanNeighbors() }
                } label: {
                    HStack {
                        Text("扫描邻居")
                        if viewModel.isScanning {
                            Spacer()
                            ProgressView()
                        }
                    }
                }
                .disabled(viewModel.isScanning)

                ForEach(viewModel.neighbors) { cell in
                    VStack(alignment: .leading, spacing: 4) {
                        HStack {
                            Text(cell.type)
                                .font(.headline)
                            Text("频段 \(cell.band)")
                                .font(.subheadline)
                                .foregroundStyle(.secondary)
                        }
                        Text("PCI：\(cell.pci) EARFCN：\(cell.earfcn) RSRP：\(cell.rsrp)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.vertical, 2)
                }
            }

            if viewModel.status.locked {
                Section {
                    Button("解锁单元格", role: .destructive) {
                        Task { await viewModel.unlock() }
                    }
                    .disabled(viewModel.isLoading)
                }
            }
        }
        .navigationTitle("小区锁定")
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
