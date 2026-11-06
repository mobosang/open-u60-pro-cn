import SwiftUI

@Observable
@MainActor
final class STCViewModel {
    var config: STCConfig = .empty
    var isLoading: Bool = false
    var message: String?
    var messageIsError: Bool = false

    var editLteTimer: String = ""
    var editNrsaTimer: String = ""
    var editLteMax: String = ""
    var editNrsaMax: String = ""

    private let client: AgentClient
    private let authManager: AuthManager

    init(client: AgentClient, authManager: AuthManager) {
        self.client = client
        self.authManager = authManager
    }

    func refresh() async {
        isLoading = true
        message = nil

        do {
            let paramsData = try await client.getJSON("/api/cell/stc/params")
            config = STCParser.parseParams(paramsData)

            if let statusData = try? await client.getJSON("/api/cell/stc/status") {
                config = STCParser.parseStatus(statusData, into: config)
            }

            editLteTimer = config.lteCollectTimer
            editNrsaTimer = config.nrsaCollectTimer
            editLteMax = config.lteWhitelistMax
            editNrsaMax = config.nrsaWhitelistMax
        } catch {
            showMessage("加载STC失败：\(error.localizedDescription)", isError: true)
        }

        isLoading = false
    }

    func applyParams() async {
        isLoading = true

        do {
            let _ = try await client.putJSON("/api/cell/stc/params", body: [
                "lte_collect_timer": editLteTimer,
                "nrsa_collect_timer": editNrsaTimer,
                "lte_whitelist_max": editLteMax,
                "nrsa_whitelist_max": editNrsaMax
            ])
            showMessage("STC参数更新", isError: false)
            config.lteCollectTimer = editLteTimer
            config.nrsaCollectTimer = editNrsaTimer
            config.lteWhitelistMax = editLteMax
            config.nrsaWhitelistMax = editNrsaMax
        } catch {
            showMessage("失败：\(error.localizedDescription)", isError: true)
        }

        isLoading = false
    }

    func enable() async {
        isLoading = true
        do {
            let _ = try await client.postJSON("/api/cell/stc/enable")
            showMessage("STC 启用", isError: false)
            config.enabled = true
        } catch {
            showMessage("失败：\(error.localizedDescription)", isError: true)
        }
        isLoading = false
    }

    func disable() async {
        isLoading = true
        do {
            let _ = try await client.postJSON("/api/cell/stc/disable")
            showMessage("STC 禁用", isError: false)
            config.enabled = false
        } catch {
            showMessage("失败：\(error.localizedDescription)", isError: true)
        }
        isLoading = false
    }

    func reset() async {
        isLoading = true
        do {
            let _ = try await client.postJSON("/api/cell/stc/reset")
            showMessage("STC白名单重置", isError: false)
            config.enabled = false
        } catch {
            showMessage("失败：\(error.localizedDescription)", isError: true)
        }
        isLoading = false
    }

    private func showMessage(_ text: String, isError: Bool) {
        message = text
        messageIsError = isError
    }
}
