import SwiftUI

struct SMSForwardConfigView: View {
    @Bindable var viewModel: SMSForwardViewModel

    @State private var markRead = false
    @State private var deleteAfter = false

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

            Section("设置") {
                Toggle("已启用", isOn: Binding(
                    get: { viewModel.config.enabled },
                    set: { val in Task { await viewModel.toggleEnabled(val) } }
                ))

                Toggle("转发后标记为已读", isOn: $markRead)

                Toggle("转发后删除", isOn: $deleteAfter)

                Button {
                    Task {
                        await viewModel.updateConfig(
                            enabled: viewModel.config.enabled,
                            pollIntervalSecs: viewModel.config.pollIntervalSecs,
                            markRead: markRead,
                            deleteAfter: deleteAfter
                        )
                    }
                } label: {
                    Text("保存设置")
                        .frame(maxWidth: .infinity)
                }
            }

            if viewModel.config.rules.isEmpty && !viewModel.isLoading {
                Section {
                    ContentUnavailableView {
                        Label("没有规则", systemImage: "envelope.arrow.triangle.branch")
                    } description: {
                        Text("添加转发规则，自动将短信转发到Telegram、Discord、webhooks等。")
                    }
                }
            }

            if !viewModel.config.rules.isEmpty {
                Section("规则") {
                    ForEach(viewModel.config.rules) { rule in
                        HStack {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(rule.name)
                                    .font(.headline)
                                Text(filterSummary(rule.filter))
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                Text(destinationSummary(rule.destination))
                                    .font(.caption2)
                                    .foregroundStyle(.secondary)
                            }
                            .contentShape(Rectangle())
                            .onTapGesture {
                                viewModel.presentedSheet = .edit(rule)
                            }
                            .accessibilityElement(children: .combine)
                            .accessibilityAddTraits(.isButton)
                            .accessibilityHint("Edit rule")
                            Spacer()
                            Toggle(rule.name, isOn: Binding(
                                get: { rule.enabled },
                                set: { val in Task { await viewModel.toggleRule(id: rule.id, enabled: val) } }
                            ))
                            .labelsHidden()
                        }
                        .padding(.vertical, 2)
                        .swipeActions(edge: .trailing) {
                            Button(role: .destructive) {
                                Task { await viewModel.deleteRule(id: rule.id) }
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
                    Label("新规则", systemImage: "plus")
                }
            }

            Section {
                NavigationLink {
                    SMSForwardLogView(viewModel: viewModel)
                } label: {
                    Label("转发日志", systemImage: "doc.text")
                }
            }

            if viewModel.lastForwardedId > 0 {
                Section {
                    HStack {
                        Text("最后转发的 ID")
                            .foregroundStyle(.secondary)
                        Spacer()
                        Text("\(viewModel.lastForwardedId)")
                            .font(.caption.monospacedDigit())
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .navigationTitle("短信转发")
        .refreshable {
            await viewModel.refresh()
            syncLocalState()
        }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .task {
            await viewModel.refresh()
            syncLocalState()
        }
        .sheet(item: $viewModel.presentedSheet) { sheet in
            switch sheet {
            case .add:
                SMSForwardRuleFormView(viewModel: viewModel)
            case .edit(let rule):
                SMSForwardRuleFormView(viewModel: viewModel, editingRule: rule)
            }
        }
    }

    private func syncLocalState() {
        markRead = viewModel.config.markReadAfterForward
        deleteAfter = viewModel.config.deleteAfterForward
    }

    private func filterSummary(_ filter: SmsFilter) -> String {
        switch filter {
        case .all:
            return "全部短信"
        case .sender(let patterns):
            return "Sender: \(patterns.joined(separator: ", "))"
        case .content(let keywords):
            return "Keywords: \(keywords.joined(separator: ", "))"
        case .senderAndContent(let patterns, let keywords):
            return "Sender: \(patterns.joined(separator: ", ")) + Keywords: \(keywords.joined(separator: ", "))"
        }
    }

    private func destinationSummary(_ dest: ForwardDestination) -> String {
        switch dest {
        case .telegram(_, let chatId, _):
            return "Telegram（聊天：\(chatId)）"
        case .webhook(let url, _, _):
            return "Webhook（\(url)）"
        case .sms(let number):
            return "短信（\(number)）"
        case .ntfy(_, let topic, _):
            return "ntfy (\(topic))"
        case .discord:
            return "Discord"
        case .slack:
            return "Slack"
        }
    }
}
