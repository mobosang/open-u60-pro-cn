import SwiftUI

struct LoginView: View {
    let authManager: AuthManager
    @State private var viewModel: LoginViewModel

    init(authManager: AuthManager) {
        self.authManager = authManager
        _viewModel = State(initialValue: LoginViewModel(authManager: authManager))
    }

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Spacer()

                VStack(spacing: 8) {
                    Image(systemName: "wifi.router.fill")
                        .font(.system(size: 64))
                        .foregroundStyle(.blue)
                    Text("ZTE U60 Pro")
                        .font(.title.bold())
                    Text("路由器配套工具")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                VStack(spacing: 16) {
                    SecureField("Agent 密码", text: $viewModel.password)
                        .textFieldStyle(.roundedBorder)
                        .textContentType(.password)
                        .submitLabel(.go)
                        .onSubmit {
                            Task { await viewModel.login() }
                        }

                    Toggle("保存至Keychain", isOn: $viewModel.saveToKeychain)
                        .font(.subheadline)

                    if let error = viewModel.errorMessage {
                        Text(error)
                            .font(.caption)
                            .foregroundStyle(.red)
                            .multilineTextAlignment(.center)
                    }

                    Button {
                        Task { await viewModel.login() }
                    } label: {
                        Group {
                            if viewModel.isLoading {
                                ProgressView()
                                    .tint(.white)
                            } else {
                                Text("登录")
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .frame(height: 44)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(viewModel.password.isEmpty || viewModel.isLoading)
                }
                .padding(.horizontal, 32)

                Spacer()
                Spacer()
            }
            .navigationBarTitleDisplayMode(.inline)
        }
    }
}
