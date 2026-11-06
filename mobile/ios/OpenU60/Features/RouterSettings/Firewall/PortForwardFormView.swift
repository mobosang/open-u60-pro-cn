import SwiftUI

struct PortForwardFormView: View {
    var viewModel: FirewallSettingsViewModel
    @Environment(\.dismiss) private var dismiss

    @State private var name: String = ""
    @State private var protocol_: String = "tcp"
    @State private var wanPort: String = ""
    @State private var lanIP: String = ""
    @State private var lanPort: String = ""

    var body: some View {
        NavigationStack {
            Form {
                Section("规则") {
                    TextField("名称", text: $name)
                    Picker("协议", selection: $protocol_) {
                        Text("TCP").tag("tcp")
                        Text("UDP").tag("udp")
                        Text("TCP + UDP").tag("tcp+udp")
                    }
                }

                Section("WAN") {
                    TextField("广域网端口", text: $wanPort)
                        .keyboardType(.numberPad)
                }

                Section("LAN") {
                    TextField("LAN IP", text: $lanIP)
                        .keyboardType(.decimalPad)
                        .autocorrectionDisabled()
                    TextField("局域网接口", text: $lanPort)
                        .keyboardType(.numberPad)
                }

                Section {
                    Button {
                        Task {
                            await viewModel.addPortForward(
                                name: name,
                                protocol_: protocol_,
                                wanPort: wanPort,
                                lanIP: lanIP,
                                lanPort: lanPort
                            )
                        }
                    } label: {
                        Text("添加规则")
                            .frame(maxWidth: .infinity)
                    }
                    .disabled(wanPort.isEmpty || lanIP.isEmpty || lanPort.isEmpty || viewModel.isLoading)
                }
            }
            .navigationTitle("新建端口转发")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { dismiss() }
                }
            }
        }
    }
}
