import SwiftUI

struct PasswordConfirmView: View {
    let title: String
    let message: String
    let confirmLabel: String
    let onConfirm: () async -> Void
    @Environment(\.dismiss) private var dismiss

    @State private var password: String = ""
    @State private var error: String?
    @State private var isProcessing: Bool = false

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Text(message)
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                Section {
                    SecureField("路由器管理密码", text: $password)
                        .textContentType(.password)
                        .autocorrectionDisabled()
                }

                if let error {
                    Section {
                        Text(error)
                            .font(.subheadline)
                            .foregroundStyle(.red)
                    }
                }

                Section {
                    Button(role: .destructive) {
                        Task { await confirm() }
                    } label: {
                        HStack {
                            Spacer()
                            if isProcessing {
                                ProgressView()
                            } else {
                                Text(confirmLabel)
                            }
                            Spacer()
                        }
                    }
                    .disabled(password.isEmpty || isProcessing)
                }
            }
            .navigationTitle(title)
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
    }

    private func confirm() async {
        guard let storedPassword = KeychainHelper.load(key: "router_password") else {
            error = "未找到存储的密码。请重新登录。"
            return
        }

        guard password == storedPassword else {
            error = "Incorrect password"
            password = ""
            return
        }

        isProcessing = true
        await onConfirm()
        isProcessing = false
        dismiss()
    }
}
