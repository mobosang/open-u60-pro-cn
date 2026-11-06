import SwiftUI

struct APNView: View {
    @Bindable var viewModel: APNViewModel
    @State private var showAutoDetail: APNProfile?

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

            if let activeName = viewModel.activeAPNName {
                Section("当前APN") {
                    HStack {
                        Text(activeName)
                            .font(.headline)
                        Spacer()
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    }
                }
            }

            Section("APN模式") {
                Toggle("说明书APN", isOn: Binding(
                    get: { viewModel.config.isManual },
                    set: { manual in Task { await viewModel.setMode(manual: manual) } }
                ))
                .disabled(viewModel.isLoading)
            }

            if viewModel.config.isManual {
                Section {
                    Button {
                        viewModel.startAdd()
                    } label: {
                        Label("添加APN", systemImage: "plus")
                    }
                } header: {
                    Text("手动配置文件")
                }

                if !viewModel.config.profiles.isEmpty {
                    Section {
                        ForEach(viewModel.config.profiles) { profile in
                            APNProfileRow(profile: profile)
                                .contentShape(Rectangle())
                                .onTapGesture {
                                    viewModel.startEdit(profile)
                                }
                                .swipeActions(edge: .trailing) {
                                    Button(role: .destructive) {
                                        Task { await viewModel.deleteAPN(profile) }
                                    } label: {
                                        Label("删除", systemImage: "trash")
                                    }
                                    .disabled(profile.active)
                                }
                                .swipeActions(edge: .leading) {
                                    if !profile.active {
                                        Button {
                                            Task { await viewModel.activateAPN(profile) }
                                        } label: {
                                            Label("激活", systemImage: "checkmark.circle")
                                        }
                                        .tint(.green)
                                    }
                                }
                        }
                    }
                }
            } else {
                // Auto mode: show auto profiles read-only
                if !viewModel.config.autoProfiles.isEmpty {
                    Section("自动配置文件") {
                        ForEach(viewModel.config.autoProfiles) { profile in
                            APNProfileRow(profile: profile)
                                .contentShape(Rectangle())
                                .onTapGesture {
                                    showAutoDetail = profile
                                }
                        }
                    }
                }
            }
        }
        .navigationTitle("APN 设置")
        .refreshable { await viewModel.refresh() }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .padding()
                    .background(Color(.systemBackground).opacity(0.85), in: RoundedRectangle(cornerRadius: 8))
            }
        }
        .task { await viewModel.refresh() }
        .sheet(isPresented: $viewModel.showFormSheet) {
            APNFormSheet(viewModel: viewModel)
        }
        .sheet(item: $showAutoDetail) { profile in
            APNAutoDetailSheet(profile: profile)
        }
    }
}

// MARK: - Profile Row

private struct APNProfileRow: View {
    let profile: APNProfile

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(profile.name.isEmpty ? "Unnamed" : profile.name)
                    .font(.headline)
                Spacer()
                if profile.active {
                    Text("已启用")
                        .font(.caption)
                        .foregroundStyle(.green)
                }
            }
            Text(profile.apn)
                .font(.subheadline)
                .foregroundStyle(.secondary)
            Text("\(profile.pdpTypeLabel) / \(profile.authModeLabel)")
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
        .padding(.vertical, 2)
    }
}

// MARK: - Add/Edit Form Sheet

struct APNFormSheet: View {
    @Bindable var viewModel: APNViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            Form {
                Section("配置文件") {
                    TextField("名称", text: $viewModel.formProfile.name)
                    TextField("APN", text: $viewModel.formProfile.apn)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                }

                Section("Connection") {
                    Picker("PDP 类型", selection: $viewModel.formProfile.pdpType) {
                        ForEach(APNProfile.pdpTypeOptions, id: \.value) { opt in
                            Text(opt.label).tag(opt.value)
                        }
                    }

                    Picker("认证模式", selection: $viewModel.formProfile.authMode) {
                        ForEach(APNProfile.authModeOptions, id: \.value) { opt in
                            Text(opt.label).tag(opt.value)
                        }
                    }
                }

                if viewModel.formProfile.authMode != 0 {
                    Section("身份凭据") {
                        TextField("用户名", text: $viewModel.formProfile.username)
                            .autocorrectionDisabled()
                            .textInputAutocapitalization(.never)
                        SecureField("密码", text: $viewModel.formProfile.password)
                    }
                }

                Section {
                    Toggle("设为默认", isOn: $viewModel.setAsDefault)
                }

                Section {
                    Button {
                        Task { await viewModel.saveAPN() }
                    } label: {
                        Text(viewModel.isEditing ? "Save" : "添加APN")
                            .frame(maxWidth: .infinity)
                    }
                    .disabled(
                        viewModel.formProfile.name.isEmpty
                        || viewModel.formProfile.apn.isEmpty
                        || viewModel.isLoading
                    )
                }
            }
            .navigationTitle(viewModel.isEditing ? "Edit APN" : "新APN")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
    }
}

// MARK: - Auto APN Detail (Read-Only)

struct APNAutoDetailSheet: View {
    let profile: APNProfile
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                Section("配置文件") {
                    LabeledContent("名称", value: profile.name.isEmpty ? "—" : profile.name)
                    LabeledContent("APN", value: profile.apn.isEmpty ? "—" : profile.apn)
                }
                Section("Connection") {
                    LabeledContent("PDP 类型", value: profile.pdpTypeLabel)
                    LabeledContent("认证模式", value: profile.authModeLabel)
                }
                if profile.authMode != 0 {
                    Section("身份凭据") {
                        LabeledContent("用户名", value: profile.username.isEmpty ? "—" : profile.username)
                    }
                }
                Section {
                    LabeledContent("状态", value: profile.active ? "已启用" : "未启用")
                }
            }
            .navigationTitle("汽车APN")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("完成") { dismiss() }
                }
            }
        }
    }
}
