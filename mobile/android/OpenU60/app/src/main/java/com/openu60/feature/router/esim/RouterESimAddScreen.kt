package com.openu60.feature.router.esim

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.SecureFlagPolicy
import com.google.mlkit.vision.barcode.BarcodeScanning
import com.google.mlkit.vision.barcode.BarcodeScannerOptions
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning
import com.google.mlkit.vision.common.InputImage
import com.openu60.core.model.RouterESimConnectionPhase
import com.openu60.core.model.RouterESimActivationCode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RouterESimAddScreen(viewModel: RouterESimViewModel, onClose: () -> Unit) {
    val state by viewModel.state.collectAsState()
    val context = LocalContext.current
    val haptic = LocalHapticFeedback.current
    val scope = rememberCoroutineScope()
    // Secrets intentionally use remember, never rememberSaveable or SavedStateHandle.
    var code by remember { mutableStateOf("") }
    var confirmationCode by remember { mutableStateOf("") }
    var manual by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var reading by remember { mutableStateOf(false) }
    var alive by remember { mutableStateOf(true) }
    val parsed = remember(code) { RouterESimActivationCode.parse(code) }
    val accept: (String) -> Unit = { value ->
        val result = RouterESimActivationCode.parse(value)
        if (result == null) error = RouterESimActivationCode.INVALID_MESSAGE
        else {
            code = result.value; confirmationCode = ""; manual = true; error = null
            haptic.performHapticFeedback(HapticFeedbackType.LongPress)
        }
    }
    val close = { code = ""; confirmationCode = ""; onClose() }
    DisposableEffect(Unit) {
        onDispose { alive = false; code = ""; confirmationCode = "" }
    }
    LaunchedEffect(state.download.state) {
        if (state.download.state == "done") { code = ""; confirmationCode = "" }
        if (state.download.state == "cancelled") viewModel.resetDownloadResult()
    }
    val photoPicker = rememberLauncherForActivityResult(ActivityResultContracts.PickVisualMedia()) { uri ->
        if (uri != null) {
            reading = true
            scope.launch {
                try {
                    val image = withContext(Dispatchers.IO) { InputImage.fromFilePath(context, uri) }
                    val scanner = BarcodeScanning.getClient(BarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE).build())
                    scanner.process(image).addOnSuccessListener { barcodes ->
                        if (alive) {
                            val valid = barcodes.mapNotNull { it.rawValue }.filter { RouterESimActivationCode.parse(it) != null }.distinct()
                            if (valid.size == 1) accept(valid.first())
                            else error = if (valid.isEmpty()) RouterESimActivationCode.INVALID_MESSAGE else "选择一张带有eSIM二维码的照片。"
                        }
                    }.addOnFailureListener { if (alive) error = "无法阅读这张照片。选择其他图像或手动输入代码。" }
                        .addOnCompleteListener { scanner.close(); if (alive) reading = false }
                } catch (_: Exception) { if (alive) { error = "无法阅读这张照片。选择其他图像或手动输入代码。"; reading = false } }
            }
        }
    }
    val scan = {
        reading = true
        val scanner = GmsBarcodeScanning.getClient(context, GmsBarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE).enableAutoZoom().build())
        scanner.startScan().addOnSuccessListener { if (alive) accept(it.rawValue.orEmpty()) }
            .addOnFailureListener { if (alive) error = "无法打开扫描仪。再试一次，选择照片，或手动输入代码。" }
            .addOnCompleteListener { if (alive) reading = false }
        Unit
    }

    Dialog(onDismissRequest = close, properties = DialogProperties(usePlatformDefaultWidth = false, securePolicy = SecureFlagPolicy.SecureOn)) {
        Scaffold(topBar = {
            TopAppBar(title = { Text("添加 eSIM") }, actions = { TextButton(onClick = close) { Text(if (state.download.isBusy) "关闭" else "完成") } })
        }) { padding ->
            Column(Modifier.fillMaxSize().padding(padding).verticalScroll(rememberScrollState()).padding(20.dp), verticalArrangement = Arrangement.spacedBy(20.dp)) {
                if (state.download.state !in setOf("idle", "cancelled") || state.operationInFlight) {
                    DownloadContent(state, viewModel, { viewModel.resetDownloadResult(); manual = true }, close)
                } else {
                    Icon(Icons.Default.QrCodeScanner, null, Modifier.size(40.dp), tint = MaterialTheme.colorScheme.primary)
                    Text("将配置文件添加到您的路由器", style = MaterialTheme.typography.headlineSmall)
                    Text("使用运营商提供的 eSIM 二维码。您将在安装之前查看配置文件。", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    if (state.connectionPhase == RouterESimConnectionPhase.DISCONNECTED || state.connectionPhase == RouterESimConnectionPhase.WAITING) {
                        Surface(color = MaterialTheme.colorScheme.tertiaryContainer.copy(alpha = 0.35f), shape = RoundedCornerShape(12.dp)) {
                            Text("您的路由器没有移动数据连接。下载前先连接；无法使用您手机本身的数据。", Modifier.padding(14.dp), style = MaterialTheme.typography.bodyMedium)
                        }
                    }
                    Card(Modifier.fillMaxWidth()) {
                        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                            Button(onClick = scan, enabled = !reading, modifier = Modifier.fillMaxWidth()) { Icon(Icons.Default.CameraAlt, null); Spacer(Modifier.width(8.dp)); Text("扫描二维码") }
                            OutlinedButton(onClick = { photoPicker.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly)) }, enabled = !reading, modifier = Modifier.fillMaxWidth()) {
                                Icon(Icons.Default.Photo, null); Spacer(Modifier.width(8.dp)); Text(if (reading) "Reading…" else "从照片中选择二维码")
                            }
                            TextButton(onClick = { manual = true }, modifier = Modifier.fillMaxWidth()) { Icon(Icons.Default.Keyboard, null); Spacer(Modifier.width(8.dp)); Text("手动输入代码") }
                        }
                    }
                    if (manual || code.isNotEmpty()) {
                        Card(Modifier.fillMaxWidth()) {
                            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                                Text("激活码", style = MaterialTheme.typography.titleSmall)
                                OutlinedTextField(code, { code = it; confirmationCode = ""; error = null }, label = { Text("LPA:1$…") }, modifier = Modifier.fillMaxWidth(),
                                    visualTransformation = PasswordVisualTransformation(), keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.None, autoCorrectEnabled = false, keyboardType = KeyboardType.Password))
                                parsed?.let { Text("代码已准备好·来自${it.server}", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary) }
                                if (parsed?.needsConfirmationCode == true) {
                                    OutlinedTextField(confirmationCode, { confirmationCode = it }, label = { Text("运营商确认码") }, modifier = Modifier.fillMaxWidth(),
                                        visualTransformation = PasswordVisualTransformation(), keyboardOptions = KeyboardOptions(autoCorrectEnabled = false, keyboardType = KeyboardType.Password),
                                        supportingText = { Text("运营商提供的单独代码（最多 64 个字符）。") })
                                }
                                Button(onClick = {
                                    if (parsed == null) error = RouterESimActivationCode.INVALID_MESSAGE
                                    else if (parsed.needsConfirmationCode && !RouterESimActivationCode.validConfirmationCode(confirmationCode)) error = "请输入运营商提供的确认码（最多 64 个字符）。"
                                    else viewModel.startDownload(parsed.value, confirmationCode)
                                }, enabled = code.isNotBlank() && !state.blocksSwitching && !reading) { Text("阅读配置文件详细信息") }
                            }
                        }
                    }
                    error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
                    state.downloadNote?.let { note ->
                        Card(Modifier.fillMaxWidth()) {
                            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                                Text("无法开始此下载", style = MaterialTheme.typography.titleSmall)
                                Text(note, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                if (note.contains("agent password required", true)) Text("在代理设置中设置路由器代理密码，然后重试。", style = MaterialTheme.typography.bodySmall)
                            }
                        }
                    }
                    Text("您的路由器需要互联网来下载配置文件。保持 Wi-Fi 连接。下载通常需要 10-60 秒。", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text("您的激活码是私人且一次性的。它仅发送到您的路由器，不会被 OpenU60 保存。", style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
private fun DownloadContent(state: RouterESimState, viewModel: RouterESimViewModel, retry: () -> Unit, close: () -> Unit) {
    val download = state.download
    Card(Modifier.fillMaxWidth()) {
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(18.dp)) {
            if (download.state in setOf("running", "cancelling") || state.operationInFlight) CircularProgressIndicator(Modifier.size(32.dp))
            else Icon(if (download.state == "confirm") Icons.Default.SimCard else if (download.state == "done") Icons.Default.CheckCircle else Icons.Default.ErrorOutline, null, Modifier.size(40.dp), tint = MaterialTheme.colorScheme.primary)
            Text(download.progressText, style = MaterialTheme.typography.titleLarge, modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
            when (download.state) {
                "confirm" -> {
                    Text("安装${download.displayName}？", style = MaterialTheme.typography.titleMedium)
                    download.name?.takeIf { it != download.displayName }?.let { Text(it) }
                    Text("该配置文件将安装在您路由器的 eSIM 上并保持非活动状态，直到您切换到它。 2分钟内确认。")
                    Button(onClick = viewModel::confirmDownload, enabled = !state.operationInFlight) { Text("安装配置文件") }
                }
                "done" -> {
                    Text("添加到您的路由器。您当前的配置文件仍然有效。")
                    val profile = state.profiles.firstOrNull { it.iccid == download.iccid }
                    if (profile?.canEnable == true) {
                        Button(onClick = { viewModel.requestSwitch(profile); close() }, enabled = !state.blocksSwitching) { Text("立即切换") }
                    } else TextButton(onClick = viewModel::refresh, enabled = !state.blocksSwitching) { Text("刷新配置文件") }
                    OutlinedButton(onClick = close) { Text("后来") }
                }
                "failed" -> {
                    Text(download.error ?: "The operator couldn’t add this profile.", color = MaterialTheme.colorScheme.error)
                    Button(onClick = retry) { Text("重试") }
                }
                else -> Text(if (download.step == "es10b_load_bound_profile_package") "保持路由器电源开启。在此阶段无法取消安装。" else "请保持路由器联网。你可以关闭此页面，稍后返回查看进度。")
            }
            state.downloadNote?.let {
                Text(it, color = MaterialTheme.colorScheme.error)
                TextButton(onClick = viewModel::retryDownloadStatus, enabled = !state.operationInFlight) { Text("检查状态") }
            }
            if (download.canCancel) OutlinedButton(onClick = viewModel::cancelDownload, enabled = !state.operationInFlight) { Text("取消下载", color = MaterialTheme.colorScheme.error) }
        }
    }
}

