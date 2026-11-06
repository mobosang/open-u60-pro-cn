package com.openu60.core.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** One profile on the eSIM in the router (GET /api/esim/profiles). */
data class RouterESimProfile(
    val iccid: String,
    val provider: String = "",
    val name: String = "",
    val nickname: String = "",
    val isEnabled: Boolean = false,
    val profileClass: String = "",
    /** The agent's read-only pre-switch check: `manual_apn`, `roaming_off`. */
    val switchWarnings: List<String> = emptyList(),
) {
    /** The nickname when set, otherwise the operator's profile name. */
    val displayName: String
        get() = nickname.ifEmpty { name.ifEmpty { provider } }

    /** Only ordinary operator profiles can be switched to (the agent refuses test profiles). */
    val canEnable: Boolean
        get() = !isEnabled && profileClass == "operational"
}

/** Identity and capacity of the eSIM chip (GET /api/esim/chip). */
data class RouterESimChip(
    val eid: String = "",
    val firmware: String = "",
    val freeMemoryBytes: Long? = null,
) {
    companion object {
        val empty = RouterESimChip()
    }
}

// Wire types, decoded with AgentClient.get<T>(): the untyped getJSON() map turns digit-only
// strings into numbers, which would corrupt 19–20 digit ICCIDs and the 32-digit EID.

@Serializable
data class RouterESimProfileDto(
    val iccid: String? = null,
    val state: String? = null,
    val nickname: String? = null,
    val provider: String? = null,
    val name: String? = null,
    @SerialName("class") val profileClass: String? = null,
    @SerialName("switch_warnings") val switchWarnings: List<String>? = null,
)

@Serializable
data class RouterESimReadError(
    val kind: String? = null,
    val message: String? = null,
    val at: Long? = null,
)

/**
 * GET /api/esim/profiles. While a switch reloads the SIM, or another eSIM operation holds the
 * card, the agent answers with its last good list marked `updating`; a failed read returns
 * that list as `stale` with `error`.
 */
@Serializable
data class RouterESimProfilesResponse(
    val profiles: List<RouterESimProfileDto> = emptyList(),
    @SerialName("observed_at") val observedAt: Long = 0,
    val generation: Long = 0,
    @SerialName("agent_instance") val agentInstance: Long = 0,
    val updating: Boolean = false,
    val stale: Boolean = false,
    val error: RouterESimReadError? = null,
    @SerialName("pending_receipts") val pendingReceipts: Int? = null,
    /** Counts card reads in this agent run; unlike observed_at it ignores the router's clock jumps. */
    @SerialName("observed_seq") val observedSeq: Long = 0,
    @SerialName("manual_apn") val manualApn: RouterESimManualApn? = null,
)

/** The router's enabled manual APN entry (null while the APN mode is automatic). */
@Serializable
data class RouterESimManualApn(
    val name: String = "",
    val apn: String = "",
)

/** The list as accepted on screen, with its freshness markers. */
data class RouterESimInventory(
    val profiles: List<RouterESimProfile> = emptyList(),
    val observedAt: Long = 0,
    val generation: Long = 0,
    val agentInstance: Long = 0,
    val updating: Boolean = false,
    val stale: Boolean = false,
    val errorKind: String? = null,
    val errorMessage: String? = null,
    val pendingReceipts: Int? = null,
    val observedSeq: Long = 0,
    val manualApn: RouterESimManualApn? = null,
) {
    /** True when this answer predates the one shown (same agent run); older agents carry no counters. */
    fun isOlderThan(shown: RouterESimInventory): Boolean {
        if (agentInstance == 0L || agentInstance != shown.agentInstance) return false
        if (generation != shown.generation) return generation < shown.generation
        if (observedSeq != 0L && shown.observedSeq != 0L) return observedSeq < shown.observedSeq
        return observedAt < shown.observedAt
    }

    /** Advisory lines for the switch dialog; unknown codes are skipped. */
    fun switchWarnings(target: RouterESimProfile): List<String> {
        val codes = profiles.firstOrNull { it.iccid == target.iccid }?.switchWarnings ?: target.switchWarnings
        return codes.mapNotNull { code ->
            when (code) {
                "manual_apn" -> manualApn?.let {
                    "路由器使用手动 APN“${it.apn}”。若 ${target.displayName} 需要其他 APN，请在切换后前往“移动网络”修改，否则可能无法联网。"
                }
                "roaming_off" -> "${target.displayName} 可能是旅行（漫游）配置文件；数据漫游已关闭，开启后才能使用移动数据。"
                else -> null
            }
        }
    }

    /** Text for a failed read; null when the list is current. */
    val errorText: String?
        get() {
            val message = errorMessage ?: return null
            return when (errorKind) {
                "sim_reloading" -> "切换后 eSIM 仍在准备中。"
                "management_unavailable" -> "eSIM 管理应用无响应（$message）。"
                else -> message
            }
        }
}

/** GET /api/esim/switch: the agent's verified progress of the last switch. */
@Serializable
data class RouterESimSwitchStatus(
    val state: String = "idle",
    @SerialName("target_iccid") val targetICCID: String? = null,
    val generation: Long? = null,
    @SerialName("started_at") val startedAt: Long? = null,
    @SerialName("finished_at") val finishedAt: Long? = null,
    val note: String? = null,
    /** Why data didn't connect, when the router knows: `roaming_off` or `data_off`. */
    val reason: String? = null,
) {
    val isRunning: Boolean get() = state in setOf("card_busy", "reloading", "card_confirmed", "modem_confirmed")

    /** The card is confirmed: a list read now shows the new active profile. */
    val cardConfirmed: Boolean get() = state !in setOf("idle", "card_busy", "reloading", "card_unconfirmed")

    val phase: RouterESimConnectionPhase
        get() = when (state) {
            "card_busy" -> RouterESimConnectionPhase.SWITCHING
            "reloading" -> RouterESimConnectionPhase.RELOADING
            "card_confirmed", "modem_confirmed" -> RouterESimConnectionPhase.REGISTERING
            "connected" -> RouterESimConnectionPhase.CONNECTED
            "no_data", "modem_unconfirmed", "card_unconfirmed" -> RouterESimConnectionPhase.WAITING
            "not_applied" -> RouterESimConnectionPhase.UNAVAILABLE
            else -> RouterESimConnectionPhase.CHECKING
        }

    /** Shown while the switch runs: the eSIM refused for now and the router keeps trying. */
    val progressNote: String?
        get() = if (state == "card_busy") "eSIM 正忙，暂时拒绝切换。路由器会持续尝试长达 4 分钟。" else null

    /** Why a finished switch did not reach `connected`. */
    val outcomeNote: String?
        get() = when (state) {
            "no_data" -> when (reason) {
                "roaming_off" -> "此配置文件在本地网络上漫游，并且数据漫游已关闭。在移动网络中打开数据漫游。"
                "data_off" -> "配置文件处于活动状态，但移动数据已关闭。在移动网络中将其打开。"
                else -> "配置文件处于活动状态，但移动数据尚未连接。检查计划或承保范围。"
            }
            "modem_unconfirmed" -> "eSIM已更换，但路由器仍使用之前的SIM。重新启动路由器即可完成。"
            "card_unconfirmed" -> "切换后无法读取eSIM。一会儿刷新一下。"
            "not_applied" ->
                if (reason == "card_busy") "eSIM 保持忙碌状态，不切换；之前的配置文件仍然有效。几分钟后重试。"
                else "切换没有发生；之前的配置文件仍然有效。"
            else -> null
        }
}

/** GET /api/esim/notifications (only the count is used on screen). */
@Serializable
data class RouterESimNotifications(val count: Int? = null)

@Serializable
data class RouterESimChipResponse(
    val eid: String? = null,
    val firmware: String? = null,
    @SerialName("free_nvm_bytes") val freeNvmBytes: Long? = null,
)

object RouterESimParser {
    fun parseInventory(response: RouterESimProfilesResponse): RouterESimInventory = RouterESimInventory(
        profiles = parseProfiles(response),
        observedAt = response.observedAt,
        generation = response.generation,
        agentInstance = response.agentInstance,
        updating = response.updating,
        stale = response.stale,
        errorKind = response.error?.kind,
        errorMessage = response.error?.message,
        pendingReceipts = response.pendingReceipts,
        observedSeq = response.observedSeq,
        manualApn = response.manualApn,
    )

    fun parseProfiles(response: RouterESimProfilesResponse): List<RouterESimProfile> =
        response.profiles.mapNotNull { p ->
            val iccid = p.iccid
            if (iccid.isNullOrEmpty()) return@mapNotNull null
            RouterESimProfile(
                iccid = iccid,
                provider = p.provider.orEmpty(),
                name = p.name.orEmpty(),
                nickname = p.nickname.orEmpty(),
                isEnabled = p.state == "enabled",
                profileClass = p.profileClass.orEmpty(),
                switchWarnings = p.switchWarnings.orEmpty(),
            )
        }

    fun parseChip(response: RouterESimChipResponse): RouterESimChip = RouterESimChip(
        eid = response.eid.orEmpty(),
        firmware = response.firmware.orEmpty(),
        freeMemoryBytes = response.freeNvmBytes,
    )
}

/** An active card profile and a connected mobile data session are different states. */
enum class RouterESimConnectionPhase {
    CHECKING, DISCONNECTED, SWITCHING, RELOADING, REGISTERING, CONNECTED, WAITING, UNAVAILABLE;

    val blocksSwitching: Boolean
        get() = this == SWITCHING || this == RELOADING || this == REGISTERING
}

/** Do not accept the previous profile's cached data connection as switch recovery. */
class RouterESimReconnectTracker(private val targetICCID: String) {
    private var sawDisconnect = false
    private var connectedObservations = 0

    fun phase(simICCID: String, status: String, elapsedSeconds: Double): RouterESimConnectionPhase {
        val connected = isConnected(status)
        if (status.isNotEmpty() && !connected) sawDisconnect = true
        val matches = simICCID.isNotEmpty() && normalize(simICCID) == normalize(targetICCID)
        if (matches && connected && (sawDisconnect || elapsedSeconds >= 35)) {
            connectedObservations++
            if (connectedObservations >= 2) return RouterESimConnectionPhase.CONNECTED
        } else connectedObservations = 0
        if (elapsedSeconds >= 120) return RouterESimConnectionPhase.WAITING
        return if (matches) RouterESimConnectionPhase.REGISTERING else RouterESimConnectionPhase.RELOADING
    }

    private fun normalize(value: String) = value.trim().uppercase().removeSuffix("F")

    companion object {
        fun isConnected(status: String): Boolean = status.trim().lowercase() in setOf(
            "connected", "ipv4_connected", "ipv6_connected", "ipv4_ipv6_connected",
        )
    }
}

/**
 * A moved SIM identity re-reads the eSIM list. A change that arrives while a read, a switch or an
 * operation runs waits for it to end instead of being dropped: the last change of a transition
 * (`online`) has no later one to carry it.
 */
class RouterESimIdentityRead {
    private var pending = false

    fun changed() { pending = true }

    /** True once per waiting change, as soon as the screen is not [busy]. */
    fun take(busy: Boolean): Boolean {
        if (!pending || busy) return false
        pending = false
        return true
    }
}

/** Typed decoding preserves the SIM identifier's exact digits. */
@Serializable
data class RouterESimSIMStatus(
    @SerialName("sim_iccid") val iccid: String? = null,
    @SerialName("sim_states") val status: String? = null,
)

@Serializable
data class RouterESimEnableResponse(
    @SerialName("enabled_iccid") val enabledICCID: String? = null,
)

/** A secret held only in the add screen. Deliberately no generated toString(). */
class RouterESimActivationCode private constructor(
    val value: String, val server: String, val needsConfirmationCode: Boolean,
) {
    companion object {
        const val INVALID_MESSAGE = "该二维码不是 eSIM 激活码。"
        fun parse(input: String): RouterESimActivationCode? {
            val code = input.trim()
            if (code.toByteArray().size > 512 || !code.startsWith("LPA:", ignoreCase = true) ||
                code.any { it.isWhitespace() || it.isISOControl() }) return null
            val parts = code.drop(4).split('$')
            fun ascii(c: Char) = c in '0'..'9' || c in 'a'..'z' || c in 'A'..'Z'
            if (parts.size !in 3..5 || parts[0] != "1" || parts[1].isEmpty() ||
                !parts[1].all { ascii(it) || it in ".-:" } || !parts[2].all { ascii(it) || it == '-' }) return null
            return RouterESimActivationCode("LPA:" + code.drop(4), parts[1], parts.getOrNull(4) == "1")
        }
        fun validConfirmationCode(code: String) = code.isNotEmpty() && code.toByteArray().size <= 64 && code.none { it.isISOControl() }
    }
}

@Serializable
data class RouterESimDownloadStatus(
    val state: String = "idle", val step: String? = null, val provider: String? = null,
    val name: String? = null, val iccid: String? = null, val error: String? = null,
    @SerialName("started_at") val startedAt: Long? = null,
    @SerialName("finished_at") val finishedAt: Long? = null,
) {
    val isBusy: Boolean get() = state in setOf("running", "confirm", "cancelling")
    val canCancel: Boolean get() = state in setOf("running", "confirm") && step != "es10b_load_bound_profile_package"
    val displayName: String get() = listOfNotNull(provider, name).firstOrNull { it.isNotEmpty() } ?: "eSIM profile"
    val progressText: String get() = when {
        state == "confirm" -> "准备安装"
        state == "cancelling" || step?.endsWith("_cancel_session") == true -> "取消..."
        state == "done" -> "$displayName added."
        state == "failed" -> "无法添加配置文件"
        state == "cancelled" -> "下载已取消"
        step in setOf("es9p_initiate_authentication", "es10b_authenticate_server", "es9p_authenticate_client") -> "连接到运营商..."
        step == "es8p_metadata_parse" -> "阅读配置文件详细信息..."
        step in setOf("es10b_prepare_download", "es9p_get_bound_profile_package") -> "正在下载配置文件..."
        step == "es10b_load_bound_profile_package" -> "在 eSIM 上安装..."
        else -> "准备eSIM……"
    }
}

@Serializable
data class RouterESimDeleteResponse(
    @SerialName("deleted_iccid") val deletedICCID: String? = null,
    @SerialName("operator_notified") val operatorNotified: Boolean = true,
)
