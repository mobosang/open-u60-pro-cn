import Foundation

enum AgentError: LocalizedError {
    case unauthorized
    case serverError(String)
    case networkError(Error)
    case decodingError(String)
    case serverUnreachable
    case timeout
    /// The request was cancelled (task cancellation, tab switch, view dismissal).
    /// Callers should treat this as "no result" and leave existing state untouched —
    /// never as a failure worth surfacing, retrying, or re-authenticating against.
    case cancelled

    var errorDescription: String? {
        switch self {
        case .unauthorized:
            return "未经过验证。请登录。"
        case .serverError(let message):
            return "服务器错误：\(message)"
        case .networkError(let error):
            return "网络错误：\(error.localizedDescription)"
        case .decodingError(let detail):
            return "解码响应失败：\(detail)"
        case .serverUnreachable:
            return "无法联系到代理"
        case .timeout:
            return "请求超时"
        case .cancelled:
            return "请求已取消"
        }
    }
}

extension Error {
    /// True when this error represents cancellation rather than a real failure.
    ///
    /// Recognises `CancellationError`, `AgentError.cancelled`, `URLError.cancelled`,
    /// and an `AgentError.networkError` that wraps either of the latter two — so call
    /// sites never have to unwrap the chain themselves.
    var isCancellation: Bool {
        if self is CancellationError { return true }
        if let urlError = self as? URLError, urlError.code == .cancelled { return true }
        guard let agentError = self as? AgentError else { return false }
        switch agentError {
        case .cancelled:
            return true
        case .networkError(let inner):
            return inner.isCancellation
        default:
            return false
        }
    }

    /// True when the agent rejected the request for lack of a valid session.
    var isUnauthorized: Bool {
        guard let agentError = self as? AgentError else { return false }
        if case .unauthorized = agentError { return true }
        return false
    }
}
