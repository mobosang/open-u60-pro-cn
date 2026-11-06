import SwiftUI

struct EnableADBView: View {
    let client: AgentClient
    let authManager: AuthManager

    @State private var isLoading = false
    @State private var resultMessage: String?
    @State private var isError = false

    var body: some View {
        VStack(spacing: 24) {
            Spacer()

            Image(systemName: "cable.connector.horizontal")
                .font(.system(size: 64))
                .foregroundStyle(.blue)

            Text("启用ADB调试")
                .font(.title2.bold())

            Text("设置 USB 模式进行调试，允许通过 USB-C 进行 ADB 访问。")
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 40)

            if let msg = resultMessage {
                Text(msg)
                    .font(.subheadline.weight(.medium))
                    .foregroundStyle(isError ? .red : .green)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 32)
            }

            Button {
                Task { await enableADB() }
            } label: {
                Group {
                    if isLoading {
                        ProgressView()
                            .tint(.white)
                    } else {
                        Label("启用亚行", systemImage: "power")
                    }
                }
                .frame(maxWidth: .infinity)
                .frame(height: 44)
            }
            .buttonStyle(.borderedProminent)
            .padding(.horizontal, 40)
            .disabled(isLoading)

            Spacer()
            Spacer()
        }
        .navigationTitle("启用亚行")
    }

    private func enableADB() async {
        isLoading = true
        resultMessage = nil
        do {
            let _ = try await client.putJSON("/api/usb/mode", body: ["mode": "debug"])
            resultMessage = "ADB debug mode enabled. Connect USB-C cable to access the device."
            isError = false
        } catch {
            resultMessage = "Failed: \(error.localizedDescription)"
            isError = true
        }
        isLoading = false
    }
}
