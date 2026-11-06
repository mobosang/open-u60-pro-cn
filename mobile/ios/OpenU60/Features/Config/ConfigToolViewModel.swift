import SwiftUI
import UniformTypeIdentifiers

@Observable
@MainActor
final class ConfigToolViewModel {
    var header: ConfigHeader?
    var decryptedXML: String?
    var usedKey: KnownKey?
    var isProcessing: Bool = false
    var error: String?
    var showDocumentPicker: Bool = false
    var showExporter: Bool = false
    var serialNumber: String = ""

    private var rawFileData: Data?
    private var decryptedData: Data?

    func importFile(data: Data) {
        rawFileData = data
        header = nil
        decryptedXML = nil
        usedKey = nil
        error = nil

        guard let parsedHeader = ZTEConfigCrypto.parseHeader(data: data) else {
            error = "不是有效的 ZXHN 配置文件（缺少魔术头）"
            return
        }
        header = parsedHeader
    }

    func decrypt() {
        guard let data = rawFileData, let header = header else {
            error = "未加载文件"
            return
        }

        isProcessing = true
        error = nil

        let payloadStart = Int(header.payloadOffset)
        guard payloadStart < data.count else {
            error = "负载偏移超出文件大小"
            isProcessing = false
            return
        }
        let payload = data.subdata(in: payloadStart..<data.count)

        if header.payloadType == .plain {
            finishDecrypt(payload, key: KnownKey(description: "未加密", keyBytes: Data()))
            return
        }

        let sig = header.signature.isEmpty ? nil : header.signature
        let ser = serialNumber.isEmpty ? nil : serialNumber

        guard let (decrypted, key) = ZTEConfigCrypto.tryDecrypt(
            payload: payload,
            payloadType: header.payloadType,
            serial: ser,
            signature: sig
        ) else {
            error = "无法使用任何已知密钥解密。尝试输入设备序列号。"
            isProcessing = false
            return
        }

        finishDecrypt(decrypted, key: key)
    }

    private func finishDecrypt(_ data: Data, key: KnownKey) {
        // Try decompression
        do {
            let decompressed = try ZTECompression.decompress(data)
            decryptedData = decompressed
            decryptedXML = String(data: decompressed, encoding: .utf8)
                ?? String(data: decompressed, encoding: .ascii)
                ?? "Binary data (\(decompressed.count) bytes)"
        } catch {
            // Maybe it's already XML
            if let xml = String(data: data, encoding: .utf8), xml.contains("<") {
                decryptedData = data
                decryptedXML = xml
            } else {
                self.error = "解密成功但解压失败：\(error.localizedDescription)"
            }
        }
        usedKey = key
        isProcessing = false
    }

    func reEncryptAndExport() -> Data? {
        guard let xmlData = decryptedData, let header = header else { return nil }

        do {
            let compressed = try ZTECompression.compress(xmlData, chunked: false)

            let keyData = usedKey?.keyBytes ?? ConfigConstants.knownKeys.first?.keyBytes ?? Data()
            let encrypted: Data?

            switch header.payloadType {
            case .ecb:
                encrypted = ZTEConfigCrypto.encryptECB(data: compressed, key: keyData)
            case .cbc, .cbcNew:
                encrypted = ZTEConfigCrypto.encryptCBC(data: compressed, key: keyData)
            case .plain:
                encrypted = compressed
            }

            guard let payload = encrypted else {
                error = "加密失败"
                return nil
            }

            let newHeader = ZTEConfigCrypto.buildHeader(
                payloadType: header.payloadType,
                signature: header.signature,
                payloadOffset: UInt32(ConfigConstants.headerSize)
            )
            return newHeader + payload
        } catch {
            self.error = "重新加密失败：\(error.localizedDescription)"
            return nil
        }
    }
}
