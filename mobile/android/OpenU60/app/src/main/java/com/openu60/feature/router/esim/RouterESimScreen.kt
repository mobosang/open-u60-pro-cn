package com.openu60.feature.router.esim

import android.text.format.Formatter
import java.text.DateFormat
import java.util.Date
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.openu60.core.model.RouterESimChip
import com.openu60.core.model.RouterESimConnectionPhase
import com.openu60.core.model.RouterESimProfile

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RouterESimScreen(onBack: () -> Unit, viewModel: RouterESimViewModel = hiltViewModel()) {
    val state by viewModel.state.collectAsState()
    var adding by remember { mutableStateOf(false) }
    val lifecycleOwner = LocalLifecycleOwner.current
    LaunchedEffect(viewModel) { viewModel.refresh() }
    LaunchedEffect(state.message, state.downloadNote, state.operationInFlight) {
        if (!state.operationInFlight && (state.message?.contains("a different eSIM card", ignoreCase = true) == true ||
                state.downloadNote?.contains("a different eSIM card", ignoreCase = true) == true)) viewModel.retryProfiles()
    }
    DisposableEffect(lifecycleOwner, viewModel) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_START) viewModel.startMonitoring()
            if (event == Lifecycle.Event.ON_STOP) viewModel.stopMonitoring()
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        if (lifecycleOwner.lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)) viewModel.startMonitoring()
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer); viewModel.stopMonitoring() }
    }

    Scaffold(topBar = {
        TopAppBar(
            title = { Text("eSIM 配置文件") },
            navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, "返回") } },
            actions = {
                IconButton(onClick = { viewModel.resetDownloadResult(); adding = true }, enabled = !state.blocksSwitching) { Icon(Icons.Default.Add, "添加 eSIM") }
                IconButton(onClick = viewModel::refresh, enabled = !state.isLoading && !state.isSwitching && !state.operationInFlight) {
                    if (state.isLoading) CircularProgressIndicator(Modifier.size(20.dp), strokeWidth = 2.dp)
                    else Icon(Icons.Default.Refresh, if (state.download.isBusy) "Refresh download status" else "刷新配置文件和连接")
                }
            },
        )
    }) { padding ->
        PullToRefreshBox(
            isRefreshing = state.isLoading && state.hasLoaded,
            onRefresh = viewModel::refresh,
            modifier = Modifier.padding(padding).fillMaxSize(),
        ) {
            RouterESimContent(state, viewModel::refresh, viewModel::requestSwitch, viewModel::requestDelete, { viewModel.resetDownloadResult(); adding = true }, { adding = true },
                rename = viewModel::requestRename, sendReceipts = viewModel::sendReceipts, retryList = viewModel::retryProfiles)
        }
    }

    if (adding) RouterESimAddScreen(viewModel) { adding = false }
    state.pendingDelete?.let { target ->
        AlertDialog(
            onDismissRequest = viewModel::dismissDelete,
            icon = { Icon(Icons.Default.DeleteForever, null, tint = MaterialTheme.colorScheme.error) },
            title = { Text("永久删除${target.displayName}？") },
            text = { Text("这无法撤销。您需要 ${target.provider.ifEmpty { target.displayName }} 中的新二维码才能取回此 eSIM。") },
            confirmButton = { TextButton(onClick = viewModel::confirmDelete, enabled = !state.blocksSwitching) { Text("删除配置文件", color = MaterialTheme.colorScheme.error) } },
            dismissButton = { TextButton(onClick = viewModel::dismissDelete) { Text("取消") } },
        )
    }
    state.pendingRename?.let { target ->
        RenameDialog(target, state.blocksSwitching, viewModel::dismissRename, viewModel::confirmRename)
    }
    state.pendingSwitch?.let { target ->
        AlertDialog(
            onDismissRequest = viewModel::dismissSwitch,
            icon = { Icon(Icons.Default.SwapHoriz, null) },
            title = { Text("切换到${target.displayName}？") },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("The router disconnects from ${state.enabledProfile?.displayName ?: "the current profile"} and connects with ${target.displayName}. Internet is down for about a minute. Wi-Fi stays on.")
                    // Advisory only: the switch itself is unchanged.
                    state.inventory.switchWarnings(target).forEach { Text(it, color = MaterialTheme.colorScheme.error) }
                }
            },
            confirmButton = { TextButton(onClick = viewModel::confirmSwitch, enabled = !state.blocksSwitching) { Text("切换到${target.displayName}") } },
            dismissButton = { TextButton(onClick = viewModel::dismissSwitch) { Text("取消") } },
        )
    }
}

@Composable
private fun RenameDialog(profile: RouterESimProfile, blocked: Boolean, dismiss: () -> Unit, save: (String) -> Unit) {
    var text by remember(profile.iccid) { mutableStateOf(profile.nickname) }
    val trimmed = text.trim()
    val bytes = trimmed.toByteArray(Charsets.UTF_8).size
    val valid = bytes <= 64 && text.none { it == '\n' || it == '\r' || it == '\u0085' || it == '\u2028' || it == '\u2029' }
    AlertDialog(
        onDismissRequest = dismiss,
        icon = { Icon(Icons.Default.Edit, null) },
        title = { Text("重命名配置文件") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(profile.displayName, style = MaterialTheme.typography.titleSmall)
                OutlinedTextField(text, { text = it }, singleLine = true, label = { Text("昵称") },
                    modifier = Modifier.fillMaxWidth(), placeholder = { Text(profile.name.ifEmpty { profile.provider }) },
                    isError = !valid, supportingText = {
                        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                            Text(if (valid) "One line" else "请使用单行文本，最多 64 字节。", Modifier.weight(1f))
                            Text("$bytes / 64 bytes")
                        }
                    })
                Text("留空以使用运营商名称。表情符号和重音字母每个字符可能使用多个字节。", style = MaterialTheme.typography.bodySmall)
            }
        },
        confirmButton = { TextButton(onClick = { save(trimmed) }, enabled = valid && !blocked && trimmed != profile.nickname) { Text("保存") } },
        dismissButton = { TextButton(onClick = dismiss) { Text("取消") } },
    )
}

@Composable
private fun RouterESimContent(
    state: RouterESimState, refresh: () -> Unit, switch: (RouterESimProfile) -> Unit, delete: (RouterESimProfile) -> Unit = {},
    add: () -> Unit = {}, resume: () -> Unit = {}, rename: (RouterESimProfile) -> Unit = {}, sendReceipts: () -> Unit = {},
    retryList: () -> Unit = refresh,
) {
    val others = remember(state.profiles) { state.profiles.filter { !it.isEnabled } }
    val scroll = rememberScrollState()
    LaunchedEffect(state.message, state.operationInFlight, state.switchingTo) {
        if (state.message != null || state.operationInFlight || state.switchingTo != null) scroll.animateScrollTo(0)
    }
    Column(
        modifier = Modifier.fillMaxSize().verticalScroll(scroll).padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        if (!state.hasLoaded && state.profiles.isEmpty()) {
            Card(Modifier.fillMaxWidth()) {
                Row(Modifier.padding(24.dp), horizontalArrangement = Arrangement.spacedBy(16.dp), verticalAlignment = Alignment.CenterVertically) {
                    CircularProgressIndicator(Modifier.size(24.dp), strokeWidth = 2.dp)
                    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text("阅读简介", style = MaterialTheme.typography.titleMedium)
                        Text("检查路由器中的 eSIM...", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        } else {
            state.message?.let { MessageCard(it, state.messageIsError, !state.blocksSwitching, retryList) }
            if (state.operationInFlight) {
                Card(Modifier.fillMaxWidth()) {
                    Row(Modifier.padding(16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                        CircularProgressIndicator(Modifier.size(20.dp), strokeWidth = 2.dp)
                        Text("更新eSIM…", style = MaterialTheme.typography.bodyMedium, modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
                    }
                }
            }
            if (state.download.state != "idle") {
                Card(onClick = resume, modifier = Modifier.fillMaxWidth()) {
                    Row(Modifier.padding(16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                        Icon(Icons.Default.Downloading, null, tint = MaterialTheme.colorScheme.primary)
                        Column(Modifier.weight(1f)) {
                            Text(state.download.progressText, style = MaterialTheme.typography.titleSmall)
                            Text("点击查看进度和后续步骤", style = MaterialTheme.typography.bodySmall)
                        }
                        Icon(Icons.Default.ChevronRight, null)
                    }
                }
            }
            if (state.profiles.isNotEmpty() || state.switchingTo != null) ConnectionCard(state, refresh)
            InventoryHeading(state)
            state.listError?.let { ListErrorCard(it, !state.blocksSwitching, retryList) }
            if (state.profiles.isEmpty()) {
                if (state.message == null && state.listError == null && !state.download.isBusy) {
                    Card(Modifier.fillMaxWidth()) {
                        Column(Modifier.fillMaxWidth().padding(28.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(14.dp)) {
                            Icon(Icons.Default.SimCard, null, Modifier.size(48.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text("还没有配置文件", style = MaterialTheme.typography.titleMedium)
                            Text("使用运营商提供的二维码添加一个。", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Button(onClick = add, enabled = !state.blocksSwitching) { Text("添加 eSIM") }
                        }
                    }
                }
            } else {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    state.enabledProfile?.let { active ->
                        ProfileCard(active, false, onRename = { rename(active) }, canEdit = !state.blocksSwitching) {}
                    }
                    if (others.isNotEmpty()) {
                        if (state.enabledProfile != null) SectionHeading("切换到其他配置文件")
                        others.forEach { profile ->
                            key(profile.iccid) {
                                ProfileCard(profile, !state.blocksSwitching, onDelete = { delete(profile) }, onRename = { rename(profile) },
                                    canEdit = !state.blocksSwitching) { switch(profile) }
                            }
                        }
                    }
                    OutlinedButton(onClick = add, enabled = !state.blocksSwitching, modifier = Modifier.fillMaxWidth()) {
                        Icon(Icons.Default.AddCircle, null); Spacer(Modifier.width(8.dp)); Text("添加 eSIM")
                    }
                    Row(Modifier.padding(horizontal = 4.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Icon(Icons.Default.Wifi, null, Modifier.size(18.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                        Text("切换会使互联网暂停大约一分钟。您的路由器的 Wi-Fi 保持开启状态。", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
            state.pendingReceipts?.takeIf { it > 0 }?.let { ReceiptsCard(it, state.receiptsSending, !state.blocksSwitching, sendReceipts) }
            if (state.chip != RouterESimChip.empty) ChipCard(state.chip)
        }
    }
}

@Composable
private fun InventoryHeading(state: RouterESimState) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            SectionHeading("在您的 eSIM 上")
            if (state.inventory.updating || state.connectionPhase.blocksSwitching) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    CircularProgressIndicator(Modifier.size(12.dp), strokeWidth = 1.5.dp)
                    Text("更新中……", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            } else Text(if (state.listError != null && state.profiles.isEmpty()) "尚未阅读" else if (state.profiles.size == 1) "1 profile" else "${state.profiles.size} profiles", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        if (state.inventory.observedAt > 0) {
            val time = remember(state.inventory.observedAt) { DateFormat.getTimeInstance(DateFormat.SHORT).format(Date(state.inventory.observedAt * 1000)) }
            Text("最后阅读 $time", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
private fun SectionHeading(text: String) {
    Text(text, style = MaterialTheme.typography.titleSmall, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.semantics { heading() })
}

@Composable
private fun ProfileCard(
    profile: RouterESimProfile, canSwitch: Boolean, onDelete: (() -> Unit)? = null, onRename: (() -> Unit)? = null,
    canEdit: Boolean = canSwitch, onSwitch: () -> Unit,
) {
    var menu by remember { mutableStateOf(false) }
    val accent = if (profile.isEnabled) MaterialTheme.colorScheme.tertiary else MaterialTheme.colorScheme.primary
    val subtitle = when {
        profile.provider.isNotEmpty() && !profile.provider.equals(profile.displayName, true) -> profile.provider
        profile.name.isNotEmpty() && !profile.name.equals(profile.displayName, true) -> profile.name
        profile.isEnabled -> "Active on your eSIM card"
        else -> "Stored on your eSIM card"
    }
    val shape = RoundedCornerShape(16.dp)
    val colors = CardDefaults.cardColors(containerColor = if (profile.isEnabled) accent.copy(alpha = 0.08f) else MaterialTheme.colorScheme.surfaceContainerLow)
    val border = BorderStroke(1.dp, if (profile.isEnabled) accent.copy(alpha = 0.35f) else MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))
    val content: @Composable () -> Unit = {
        Row(Modifier.fillMaxWidth().padding(start = 14.dp, end = 6.dp, top = 14.dp, bottom = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Box(Modifier.size(44.dp, 50.dp).background(accent.copy(alpha = 0.1f), RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
                Icon(Icons.Default.SimCard, null, tint = accent)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(5.dp)) {
                if (profile.isEnabled) Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    Icon(Icons.Default.CheckCircle, null, Modifier.size(12.dp), tint = accent)
                    Text("已启用", style = MaterialTheme.typography.labelSmall, color = accent)
                }
                Text(profile.displayName.ifEmpty { "未命名的配置文件" }, style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
                Text(subtitle, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Text("…${profile.iccid.takeLast(4)}", style = MaterialTheme.typography.labelSmall, fontFamily = FontFamily.Monospace, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            if (profile.canEnable) Icon(Icons.Default.SwapHoriz, null, tint = if (canSwitch) accent else MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.size(20.dp))
            else if (!profile.isEnabled) Icon(Icons.Default.Lock, "不可用", Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
            val canDelete = !profile.isEnabled && onDelete != null
            if (canDelete || onRename != null) {
                Box {
                    IconButton(onClick = { menu = true }, enabled = canEdit) { Icon(Icons.Default.MoreVert, "Profile options for ${profile.displayName}") }
                    DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                        if (onRename != null) DropdownMenuItem(text = { Text("重命名") },
                            leadingIcon = { Icon(Icons.Default.Edit, null) }, onClick = { menu = false; onRename() }, enabled = canEdit)
                        if (canDelete) DropdownMenuItem(text = { Text("删除配置文件", color = MaterialTheme.colorScheme.error) },
                            leadingIcon = { Icon(Icons.Default.DeleteOutline, null) }, onClick = { menu = false; onDelete?.invoke() }, enabled = canEdit)
                    }
                }
            }
        }
    }
    if (profile.canEnable) {
        Card(onClick = onSwitch, enabled = canSwitch, modifier = Modifier.fillMaxWidth(), shape = shape, colors = colors, border = border) { content() }
    } else {
        Card(modifier = Modifier.fillMaxWidth(), shape = shape, colors = colors, border = border) { content() }
    }
}

@Composable
private fun ConnectionCard(state: RouterESimState, refresh: () -> Unit) {
    val phase = state.connectionPhase
    val progress = phase.blocksSwitching
    val attention = phase == RouterESimConnectionPhase.WAITING || phase == RouterESimConnectionPhase.UNAVAILABLE
    val accent = if (attention) MaterialTheme.colorScheme.tertiary else MaterialTheme.colorScheme.primary
    val title = when (phase) {
        RouterESimConnectionPhase.CHECKING -> "正在检查连接"
        RouterESimConnectionPhase.SWITCHING -> "正在切换配置文件"
        RouterESimConnectionPhase.RELOADING -> "正在重新加载 SIM"
        RouterESimConnectionPhase.REGISTERING -> "正在注册网络"
        RouterESimConnectionPhase.CONNECTED -> "移动数据已连接"
        RouterESimConnectionPhase.DISCONNECTED -> "移动数据未连接"
        RouterESimConnectionPhase.WAITING -> "连接需要处理"
        RouterESimConnectionPhase.UNAVAILABLE -> if (state.switchingTo == null) "连接状态不可用" else "开关需要注意的地方"
    }
    val detail = state.connectionNote ?: when (phase) {
        RouterESimConnectionPhase.SWITCHING -> "在 eSIM 上选择新的配置文件。"
        RouterESimConnectionPhase.RELOADING -> "调制解调器正在加载 SIM。该卡可能在 5-25 秒内无法读取。"
        RouterESimConnectionPhase.REGISTERING -> "新的配置文件已加载。等待移动数据连接。"
        RouterESimConnectionPhase.WAITING -> "移动数据尚未连接。检查计划或承保范围，或选择下面的其他配置文件。"
        RouterESimConnectionPhase.DISCONNECTED -> "您的路由器的 Wi-Fi 保持连接。您仍然可以切换配置文件。"
        RouterESimConnectionPhase.CHECKING -> "正在读取路由器实时状态。"
        RouterESimConnectionPhase.UNAVAILABLE -> "再次查看路由器的连接状态。"
        RouterESimConnectionPhase.CONNECTED -> "移动数据已上。交通可能还需要几秒钟。"
    }
    val current = when {
        phase == RouterESimConnectionPhase.SWITCHING || phase == RouterESimConnectionPhase.UNAVAILABLE -> 0
        phase == RouterESimConnectionPhase.RELOADING -> 1
        else -> 2
    }
    Card(Modifier.fillMaxWidth(), colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow)) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.Top) {
                Box(Modifier.size(44.dp).background(accent.copy(alpha = 0.1f), RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
                    if (progress || phase == RouterESimConnectionPhase.CHECKING) CircularProgressIndicator(Modifier.size(22.dp), strokeWidth = 2.dp, color = accent)
                    else Icon(if (phase == RouterESimConnectionPhase.CONNECTED) Icons.Default.CheckCircle else if (attention) Icons.Default.Info else Icons.Default.SignalCellularConnectedNoInternet0Bar, null, tint = accent)
                }
                Column(Modifier.weight(1f).semantics(mergeDescendants = true) { liveRegion = LiveRegionMode.Polite }, verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    state.switchingTo?.let { Text("切换到$it", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    Text(title, style = MaterialTheme.typography.titleMedium)
                    // Outcome text participates in the live region even if the phase stays WAITING.
                    if (attention) Text(detail, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    if (phase == RouterESimConnectionPhase.CONNECTED) {
                        Text(listOf(state.operatorName, state.networkType).filter { it.isNotEmpty() }.joinToString(" · "), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                if (phase == RouterESimConnectionPhase.CONNECTED) Row(horizontalArrangement = Arrangement.spacedBy(2.dp), verticalAlignment = Alignment.Bottom) {
                    repeat(5) { index ->
                        Box(Modifier.width(3.dp).height((6 + index * 2).dp).background(
                            if (index < state.signalBars) accent else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.2f), RoundedCornerShape(1.dp)))
                    }
                }
            }
            if (state.switchingTo != null && (progress || phase == RouterESimConnectionPhase.CONNECTED)) {
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    listOf("切换配置文件" to Icons.Default.SimCard, "重新加载SIM" to Icons.Default.Refresh, "连接到网络" to Icons.Default.CellTower).forEachIndexed { index, (label, icon) ->
                        val complete = phase == RouterESimConnectionPhase.CONNECTED || index < current
                        val active = index == current && phase != RouterESimConnectionPhase.CONNECTED
                        val color = if (complete || active) accent else MaterialTheme.colorScheme.onSurfaceVariant
                        Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                            Box(Modifier.size(28.dp).background(color.copy(alpha = if (active || complete) 0.12f else 0.05f), CircleShape), contentAlignment = Alignment.Center) {
                                Icon(if (complete) Icons.Default.Check else if (active && attention) Icons.Default.PriorityHigh else icon, null, Modifier.size(16.dp), tint = color)
                            }
                            Text(label, Modifier.weight(1f), style = MaterialTheme.typography.bodyMedium, fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal, color = color)
                            if (active && progress) Text("进行中", style = MaterialTheme.typography.labelSmall, color = color)
                            if (complete) Text("完成", style = MaterialTheme.typography.labelSmall, color = color)
                        }
                    }
                }
            }
            if (!attention && (phase != RouterESimConnectionPhase.CONNECTED || state.switchingTo != null)) Text(detail, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            if (progress) Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Icon(Icons.Default.Wifi, null, Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                Text("Wi-Fi 保持连接 · 通常 15–45 秒", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            if (attention) OutlinedButton(onClick = refresh, enabled = !state.isLoading && !state.operationInFlight) { Text("再次检查") }
        }
    }
}

@Composable
private fun ListErrorCard(text: String, canRetry: Boolean, retry: () -> Unit) {
    Surface(Modifier.fillMaxWidth(), shape = RoundedCornerShape(12.dp), color = MaterialTheme.colorScheme.tertiaryContainer.copy(alpha = 0.35f)) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Icon(Icons.Default.Update, null, Modifier.size(20.dp))
                Text("无法刷新配置文件", style = MaterialTheme.typography.titleSmall)
            }
            Text(text, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            OutlinedButton(onClick = retry, enabled = canRetry) { Text("重试") }
        }
    }
}

@Composable
private fun ReceiptsCard(count: Int, sending: Boolean, canSend: Boolean, send: () -> Unit) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    Card(Modifier.fillMaxWidth(), colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow)) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(Modifier.size(40.dp).background(MaterialTheme.colorScheme.primary.copy(alpha = 0.08f), RoundedCornerShape(10.dp)), contentAlignment = Alignment.Center) {
                    Icon(Icons.AutoMirrored.Filled.Send, null, tint = MaterialTheme.colorScheme.primary)
                }
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                    Text("运营商回执", style = MaterialTheme.typography.titleSmall)
                    Text(if (sending) "Sending…" else "$count waiting · Retried automatically", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
                }
                if (sending) CircularProgressIndicator(Modifier.size(20.dp), strokeWidth = 2.dp)
            }
            TextButton(onClick = { expanded = !expanded }, contentPadding = PaddingValues(0.dp)) {
                Text("为什么这些人在等待？"); Spacer(Modifier.width(8.dp)); Icon(if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore, null, Modifier.size(16.dp))
            }
            if (expanded) Text("添加、切换或删除配置文件时，您的 eSIM 会向运营商发送回执。连接移动数据时，路由器每 10 分钟重试一次。您不需要保持此屏幕打开。",
                style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
            OutlinedButton(onClick = send, enabled = canSend && !sending) {
                Icon(Icons.Default.Upload, null, Modifier.size(16.dp)); Spacer(Modifier.width(8.dp)); Text(if (sending) "Sending receipts…" else "立即发送")
            }
        }
    }
}

@Composable
private fun MessageCard(message: String, isError: Boolean, canRetry: Boolean, retry: () -> Unit) {
    val housekeeping = message.contains("receipts", true)
    val guidance = when {
        message.contains("agent password required", true) -> "在代理设置中设置路由器代理密码，然后重试您的操作。"
        message.contains("airplane", true) || message.contains("low-power", true) -> "打开路由器设置→移动网络检查调制解调器模式。"
        message.contains("different eSIM card", true) -> "现在在路由器中重新加载卡上的配置文件。重试之前请查看该列表。"
        message.contains("404") || message.contains("not found", true) -> "此操作可能需要更新的路由器代理。您保存的配置文件未更改。"
        else -> null
    }
    Card(Modifier.fillMaxWidth()) {
        Row(Modifier.padding(16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Icon(if (housekeeping) Icons.AutoMirrored.Filled.Send else if (isError) Icons.Default.Info else Icons.Default.CheckCircle, null,
                tint = if (isError && !housekeeping) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary)
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(if (housekeeping) "运营商回执" else if (isError) "现在无法完成此操作" else "配置文件已更新", style = MaterialTheme.typography.titleSmall)
                Text(message, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
                guidance?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                if (isError && !housekeeping) {
                    OutlinedButton(onClick = retry, enabled = canRetry) { Text("重新加载配置文件") }
                    Text("重新加载仅读取列表。它不会重复您的操作。", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
private fun ChipCard(chip: RouterESimChip) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    // Revealed identifiers intentionally do not survive screen recreation or backgrounding.
    var revealed by remember { mutableStateOf(false) }
    var copied by remember { mutableStateOf(false) }
    val clipboard = LocalClipboardManager.current
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event -> if (event == Lifecycle.Event.ON_STOP) { revealed = false; copied = false } }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Surface(onClick = { expanded = !expanded; if (!expanded) { revealed = false; copied = false } }, color = Color.Transparent) {
                Row(Modifier.fillMaxWidth().heightIn(min = 48.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Icon(Icons.Default.SimCard, null, tint = MaterialTheme.colorScheme.primary)
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                        Text("eSIM 卡", style = MaterialTheme.typography.titleSmall)
                        Text("卡片详情与标识符", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    Icon(if (expanded) Icons.Default.ExpandLess else Icons.Default.ExpandMore, if (expanded) "Collapse card details" else "Expand card details", modifier = Modifier.size(20.dp))
                }
            }
            if (expanded) {
                HorizontalDivider()
                if (chip.firmware.isNotEmpty()) InfoRow("固件", chip.firmware)
                chip.freeMemoryBytes?.let { InfoRow("可用内存", Formatter.formatShortFileSize(context, it.coerceAtLeast(0))) }
                if (chip.eid.isNotEmpty()) {
                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                            Icon(Icons.Default.PrivacyTip, null, Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text("EID", Modifier.weight(1f).padding(start = 6.dp), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            IconButton(onClick = { revealed = !revealed; copied = false }) { Icon(if (revealed) Icons.Default.VisibilityOff else Icons.Default.Visibility, if (revealed) "隐藏EID" else "揭示EID") }
                            if (revealed) IconButton(onClick = { clipboard.setText(AnnotatedString(chip.eid)); copied = true }) { Icon(if (copied) Icons.Default.Check else Icons.Default.ContentCopy, if (copied) "EID copied" else "Copy EID") }
                        }
                        Text(if (revealed) chip.eid else "•••• •••• •••• …${chip.eid.takeLast(4)}", fontFamily = FontFamily.Monospace, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
    }
}

@Composable
private fun InfoRow(label: String, value: String) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(label, Modifier.weight(1f), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(value, style = MaterialTheme.typography.bodySmall)
    }
}

@Preview(showBackground = true)
@Composable
private fun ProfilesPreview() {
    MaterialTheme {
        RouterESimContent(RouterESimState(
            hasLoaded = true, connectionPhase = RouterESimConnectionPhase.CONNECTED, operatorName = "Smart", networkType = "5G NSA", signalBars = 4,
            profiles = listOf(
                RouterESimProfile("8963000000000000001", provider = "SMART", nickname = "Smart", isEnabled = true, profileClass = "operational"),
                RouterESimProfile("8963000000000000002", provider = "GOMO", name = "Globe Philippines ODC", profileClass = "operational"),
                RouterESimProfile("8963000000000000003", provider = "DITO", name = "DITO", profileClass = "operational"),
            ), chip = RouterESimChip("89000000000000000000000000000001", "4.2.0", 972020),
        ), {}, {})
    }
}
