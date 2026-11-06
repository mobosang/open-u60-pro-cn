import SwiftUI

struct USBModeSheetView: View {
    @Bindable var viewModel: USBConnectionViewModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            VStack(spacing: 24) {
                Spacer()

                Image(systemName: "cable.connector")
                    .font(.system(size: 64))
                    .foregroundStyle(.blue)
                    .accessibilityHidden(true)

                Text("USB-C 已连接")
                    .font(.title2.bold())

                Text("U60 Pro 上连接有一条 USB-C 电缆。")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 40)

                if viewModel.usbStatus.powerbankActive {
                    Label("快速充电已启动", systemImage: "bolt.fill")
                        .font(.subheadline.weight(.medium))
                        .foregroundStyle(.orange)
                        .padding(.horizontal, 32)
                } else {
                    if let msg = viewModel.message {
                        Text(msg)
                            .font(.subheadline.weight(.medium))
                            .foregroundStyle(viewModel.messageIsError ? .red : .green)
                            .textSelection(.enabled)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal, 32)
                    }

                    Button {
                        Task {
                            await viewModel.enablePowerbank()
                            dismiss()
                        }
                    } label: {
                        Group {
                            if viewModel.isLoading {
                                ProgressView()
                                    .tint(.white)
                            } else {
                                Label("快速充电", systemImage: "bolt.fill")
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .frame(minHeight: 44)
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(.orange)
                    .padding(.horizontal, 40)
                    .disabled(viewModel.isLoading)
                    .accessibilityLabel("快速充电")

                    Text("使用U60 Pro电池给手机充电")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Button("驳回") {
                    dismiss()
                }
                .padding(.top, 8)

                Spacer()
                Spacer()
            }
            .navigationTitle("USB 模式")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("关闭") { dismiss() }
                }
            }
        }
        .presentationDetents([.medium])
    }
}
