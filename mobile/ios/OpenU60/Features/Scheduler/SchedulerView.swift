import SwiftUI

struct SchedulerView: View {
    @Bindable var viewModel: SchedulerViewModel

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

            if viewModel.jobs.isEmpty && !viewModel.isLoading {
                Section {
                    ContentUnavailableView {
                        Label("无自动化", systemImage: "clock.arrow.2.circlepath")
                    } description: {
                        Text("安排飞行模式、重新启动或 WiFi 切换等操作在设备上自动运行。")
                    }
                }
            }

            if !viewModel.jobs.isEmpty {
                Section("自动化") {
                    ForEach(viewModel.jobs) { job in
                        HStack {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(job.name)
                                    .font(.headline)
                                Text(job.scheduleSummary)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Text(job.actionSummary)
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                            .contentShape(Rectangle())
                            .onTapGesture {
                                viewModel.presentedSheet = .edit(job)
                            }
                            Spacer()
                            VStack(alignment: .trailing, spacing: 4) {
                                Toggle("", isOn: Binding(
                                    get: { job.enabled },
                                    set: { val in Task { await viewModel.toggleJob(id: job.id, enabled: val) } }
                                ))
                                .labelsHidden()
                                if let status = job.lastStatus {
                                    Circle()
                                        .fill(status == 200 ? Color.green : Color.red)
                                        .frame(width: 8, height: 8)
                                }
                            }
                        }
                        .padding(.vertical, 2)
                        .swipeActions(edge: .trailing) {
                            Button(role: .destructive) {
                                Task { await viewModel.deleteJob(id: job.id) }
                            } label: {
                                Label("删除", systemImage: "trash")
                            }
                        }
                    }
                }
            }

            Section {
                Button {
                    viewModel.presentedSheet = .add
                } label: {
                    Label("新的自动化", systemImage: "plus")
                }
            }
        }
        .navigationTitle("自动化")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .task { await viewModel.refresh() }
        .sheet(item: $viewModel.presentedSheet) { sheet in
            switch sheet {
            case .add:
                SchedulerJobFormView(viewModel: viewModel)
            case .edit(let job):
                SchedulerJobFormView(viewModel: viewModel, editingJob: job)
            }
        }
    }
}
