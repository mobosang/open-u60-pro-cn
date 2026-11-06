import SwiftUI
import UniformTypeIdentifiers

struct ConfigToolView: View {
    @State private var viewModel = ConfigToolViewModel()

    var body: some View {
        List {
            Section("导入") {
                Button {
                    viewModel.showDocumentPicker = true
                } label: {
                    Label("打开配置文件", systemImage: "doc.badge.plus")
                }
            }

            if let header = viewModel.header {
                Section("标题") {
                    LabeledContent("Magic", value: header.magic)
                    LabeledContent("Encryption", value: header.payloadType.displayName)
                    LabeledContent("Signature", value: header.signature.isEmpty ? "(none)" : header.signature)
                    LabeledContent("Payload Offset", value: "\(header.payloadOffset)")
                }

                Section("解密") {
                    TextField("序列号（可选）", text: $viewModel.serialNumber)
                        .autocorrectionDisabled()

                    Button {
                        viewModel.decrypt()
                    } label: {
                        Label("解密", systemImage: "lock.open.fill")
                    }
                    .disabled(viewModel.isProcessing)
                }
            }

            if let key = viewModel.usedKey {
                Section("结果") {
                    LabeledContent("Key Used", value: key.description)
                }
            }

            if let xml = viewModel.decryptedXML {
                Section("配置 XML") {
                    ScrollView(.horizontal) {
                        Text(xml.prefix(10000))
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                    }
                    .frame(maxHeight: 400)
                }

                Section("导出") {
                    Button {
                        viewModel.showExporter = true
                    } label: {
                        Label("重新加密并导出", systemImage: "square.and.arrow.up")
                    }
                }
            }

            if let error = viewModel.error {
                Section {
                    Text(error)
                        .foregroundStyle(.red)
                        .font(.caption)
                }
            }

            if viewModel.isProcessing {
                Section {
                    HStack {
                        ProgressView()
                        Text("处理中...")
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .navigationTitle("配置工具")
        .sheet(isPresented: $viewModel.showDocumentPicker) {
            DocumentPickerView { data in
                viewModel.importFile(data: data)
            }
        }
        .sheet(isPresented: $viewModel.showExporter) {
            if let data = viewModel.reEncryptAndExport() {
                ExportDocumentView(data: data, filename: "config_encrypted.bin")
            }
        }
    }
}

// MARK: - Document Picker

struct DocumentPickerView: UIViewControllerRepresentable {
    let onPick: (Data) -> Void

    func makeUIViewController(context: Context) -> UIDocumentPickerViewController {
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: [.data, .item])
        picker.delegate = context.coordinator
        picker.allowsMultipleSelection = false
        return picker
    }

    func updateUIViewController(_ uiViewController: UIDocumentPickerViewController, context: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator(onPick: onPick)
    }

    class Coordinator: NSObject, UIDocumentPickerDelegate {
        let onPick: (Data) -> Void

        init(onPick: @escaping (Data) -> Void) {
            self.onPick = onPick
        }

        func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
            guard let url = urls.first else { return }
            let accessing = url.startAccessingSecurityScopedResource()
            defer { if accessing { url.stopAccessingSecurityScopedResource() } }
            if let data = try? Data(contentsOf: url) {
                onPick(data)
            }
        }
    }
}

// MARK: - Export Document

struct ExportDocumentView: UIViewControllerRepresentable {
    let data: Data
    let filename: String

    func makeUIViewController(context: Context) -> UIActivityViewController {
        let tempURL = FileManager.default.temporaryDirectory.appendingPathComponent(filename)
        try? data.write(to: tempURL)
        return UIActivityViewController(activityItems: [tempURL], applicationActivities: nil)
    }

    func updateUIViewController(_ uiViewController: UIActivityViewController, context: Context) {}
}
