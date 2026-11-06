import SwiftUI

struct ScheduleRebootView: View {
    @Bindable var viewModel: ScheduleRebootViewModel

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

            Section("计划") {
                Toggle("启用自动重启", isOn: $viewModel.editEnabled)

                if viewModel.editEnabled {
                    DatePicker("时间", selection: $viewModel.editTime, displayedComponents: .hourAndMinute)

                    VStack(alignment: .leading, spacing: 8) {
                        Text("天")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                        LazyVGrid(columns: Array(repeating: GridItem(.flexible()), count: 4), spacing: 8) {
                            ForEach(ScheduleRebootConfig.dayOptions, id: \.value) { day in
                                let isSelected = viewModel.editDays.contains(day.value)
                                Button {
                                    if isSelected {
                                        viewModel.editDays.remove(day.value)
                                    } else {
                                        viewModel.editDays.insert(day.value)
                                    }
                                } label: {
                                    Text(day.label)
                                        .font(.subheadline)
                                        .frame(maxWidth: .infinity)
                                        .padding(.vertical, 8)
                                        .background(isSelected ? Color.accentColor : Color(.systemGray5))
                                        .foregroundStyle(isSelected ? .white : .primary)
                                        .clipShape(RoundedRectangle(cornerRadius: 8))
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                    .padding(.vertical, 4)
                }
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
        .navigationTitle("定时重启")
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
