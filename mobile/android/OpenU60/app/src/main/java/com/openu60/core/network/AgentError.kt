package com.openu60.core.network

sealed class AgentError(override val message: String, override val cause: Throwable? = null) : Exception(message, cause) {
    class Unauthorized : AgentError("未经过验证。请登录。")
    class ServerError(msg: String) : AgentError("Server error: $msg")
    class NetworkError(msg: String, cause: Throwable? = null) : AgentError("Network error: $msg", cause)
    class DecodingError(msg: String) : AgentError("响应解码失败：$msg")
    class ServerUnreachable : AgentError("无法联系到代理")
    class Timeout : AgentError("请求超时")
}
