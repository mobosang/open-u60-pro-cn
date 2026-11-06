import SwiftUI

enum DNSMode: Int, CaseIterable, Identifiable {
    case auto, custom, doh, dot

    var id: Int { rawValue }

    var label: String {
        switch self {
        case .auto: "自动"
        case .custom: "自定义"
        case .doh: "DoH"
        case .dot: "DoT"
        }
    }
}

@Observable
@MainActor
final class DNSSettingsViewModel {
    var config: DNSConfig = .empty
    var isLoading: Bool = false
    var message: String?
    var messageIsError: Bool = false

    // Mode
    var selectedMode: DNSMode = .auto

    // Editable fields
    var editPrimary: String = ""
    var editSecondary: String = ""

    // DoH
    var doh: DoHStatus = .empty
    var editUpstream: String = ""
    var editDotEndpoint: String = ""
    var editDotServerName: String = ""
    var cacheEntries: [DoHCacheEntry] = []
    var showCacheInspector: Bool = false

    private let client: AgentClient
    private let authManager: AuthManager
    private var initialLoadDone = false

    init(client: AgentClient, authManager: AuthManager) {
        self.client = client
        self.authManager = authManager
    }

    func refresh() async {
        isLoading = true
        message = nil

        do {
            let data = try await client.getJSON("/api/dns/status")
            let mode = data["mode"] as? String ?? "automatic"
            config = DNSConfig(
                wanDnsMode: mode == "automatic" ? "auto" : "manual",
                primaryDns: data["primary"] as? String ?? "",
                secondaryDns: data["secondary"] as? String ?? "",
                ipv6PrimaryDns: "",
                ipv6SecondaryDns: "",
                ipv6DnsMode: ""
            )
            let stats = data["stats"] as? [String: Any] ?? [:]
            doh = DoHStatus(
                enabled: data["proxy_running"] as? Bool ?? false,
                upstreamUrl: data["doh_url"] as? String ?? "",
                cacheEntries: stats["cache_entries"] as? Int ?? 0,
                cacheHits: stats["cache_hits"] as? Int ?? 0,
                cacheMisses: stats["cache_misses"] as? Int ?? 0,
                queriesTotal: stats["queries_total"] as? Int ?? 0
            )
            if !initialLoadDone {
                editPrimary = config.primaryDns
                editSecondary = config.secondaryDns
                editUpstream = doh.upstreamUrl
                editDotEndpoint = data["dot_endpoint"] as? String ?? ""
                editDotServerName = data["dot_server_name"] as? String ?? ""
                switch mode {
                case "manual": selectedMode = .custom
                case "doh": selectedMode = .doh
                case "dot": selectedMode = .dot
                default: selectedMode = .auto
                }
                initialLoadDone = true
            }
            if doh.enabled {
                await refreshCache()
            }
        } catch {
            showMessage("加载DNS失败：\(error.localizedDescription)", isError: true)
        }

        isLoading = false
    }

    func apply() async {
        if selectedMode != .auto && editPrimary.isEmpty {
            showMessage("主DNS不能为空", isError: true)
            return
        }
        if selectedMode != .auto && !isValidIPv4(editPrimary) {
            showMessage("无效的主 DNS 地址", isError: true)
            return
        }
        if !editSecondary.isEmpty && !isValidIPv4(editSecondary) {
            showMessage("辅助 DNS 地址无效", isError: true)
            return
        }
        if selectedMode == .doh && editUpstream.isEmpty {
            showMessage("DoH 上游 URL 不能为空", isError: true)
            return
        }
        if selectedMode == .dot && (editDotEndpoint.isEmpty || editDotServerName.isEmpty) {
            showMessage("DoT 服务器和 TLS 名称不能为空", isError: true)
            return
        }
        isLoading = true
        do {
            let mode: String = switch selectedMode {
            case .auto: "automatic"
            case .custom: "manual"
            case .doh: "doh"
            case .dot: "dot"
            }
            let _ = try await client.putJSON("/api/dns/apply", body: [
                "mode": mode,
                "primary": selectedMode == .auto ? "" : editPrimary,
                "secondary": selectedMode == .auto ? "" : editSecondary,
                "doh_url": editUpstream,
                "dot_endpoint": editDotEndpoint,
                "dot_server_name": editDotServerName
            ])
            showMessage("DNS 设置已应用并验证", isError: false)
            await refresh()
        } catch {
            showMessage("失败：\(error.localizedDescription)", isError: true)
            isLoading = false
        }
    }

    private func isValidIPv4(_ str: String) -> Bool {
        let parts = str.split(separator: ".", omittingEmptySubsequences: false)
        guard parts.count == 4 else { return false }
        return parts.allSatisfy { part in
            guard let n = UInt16(part) else { return false }
            return n <= 255
        }
    }

    func clearCache() async {
        isLoading = true
        do {
            let _ = try await client.postJSON("/api/dns/cache/clear")
            showMessage("DNS 缓存清除", isError: false)
            await refresh()
        } catch {
            showMessage("失败：\(error.localizedDescription)", isError: true)
            isLoading = false
        }
    }

    func refreshCache() async {
        do {
            let data = try await client.getJSON("/api/dns/cache")
            let list = data["entries"] as? [[String: Any]] ?? []
            cacheEntries = DoHParser.parseCacheEntries(list)
        } catch {
            // cache fetch is best-effort
        }
    }

    func applyPreset(primary: String, secondary: String, upstream: String) {
        initialLoadDone = true
        if selectedMode == .auto {
            selectedMode = .custom
        }
        editPrimary = primary
        editSecondary = secondary
        editUpstream = upstream
    }

    private func showMessage(_ text: String, isError: Bool) {
        message = text
        messageIsError = isError
    }
}
