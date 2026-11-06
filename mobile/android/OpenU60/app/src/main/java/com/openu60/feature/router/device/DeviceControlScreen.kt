package com.openu60.feature.router.device

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.openu60.core.components.AnimatedNumber

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeviceControlScreen(
    onBack: () -> Unit,
    viewModel: DeviceControlViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    LaunchedEffect(Unit) { viewModel.refresh() }

    // Reboot confirmation dialog
    if (state.showRebootConfirm) {
        AlertDialog(
            onDismissRequest = { viewModel.dismissRebootConfirm() },
            title = { Text("重启设备") },
            text = { Text("您确定要重新启动设备吗？") },
            confirmButton = {
                TextButton(onClick = { viewModel.reboot() }) {
                    Text("重启", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.dismissRebootConfirm() }) { Text("取消") }
            },
        )
    }

    // Factory reset confirmation dialog
    if (state.showResetConfirm) {
        AlertDialog(
            onDismissRequest = { viewModel.dismissResetConfirm() },
            title = { Text("恢复出厂设置") },
            text = { Text("这将清除所有设置并将设备恢复为出厂默认设置。此操作无法撤消。") },
            confirmButton = {
                TextButton(onClick = { viewModel.factoryReset() }) {
                    Text("重置", color = MaterialTheme.colorScheme.error)
                }
            },
            dismissButton = {
                TextButton(onClick = { viewModel.dismissResetConfirm() }) { Text("取消") }
            },
        )
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("设备控制") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回")
                    }
                },
            )
        },
    ) { padding ->
        val controlsDisabled = state.isLoading

        PullToRefreshBox(
            isRefreshing = state.isLoading,
            onRefresh = { viewModel.refresh() },
            modifier = Modifier.fillMaxSize().padding(padding),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                state.message?.let { msg ->
                    Card(
                        colors = CardDefaults.cardColors(
                            containerColor = if (state.messageIsError) MaterialTheme.colorScheme.errorContainer
                            else MaterialTheme.colorScheme.primaryContainer,
                        ),
                    ) {
                        Text(
                            msg,
                            modifier = Modifier.padding(16.dp),
                            color = if (state.messageIsError) MaterialTheme.colorScheme.onErrorContainer
                            else MaterialTheme.colorScheme.onPrimaryContainer,
                        )
                    }
                }

                // Charge Limit
                if (state.chargeControlLoaded) {
                    Card(modifier = Modifier.fillMaxWidth()) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Text("充电上限", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                            Spacer(modifier = Modifier.height(8.dp))

                            ToggleRow("充电上限", state.chargeLimitEnabled) {
                                viewModel.setChargeLimit(it, state.chargeLimit)
                            }

                            if (state.chargeLimitEnabled) {
                                Spacer(modifier = Modifier.height(8.dp))
                                var sliderValue by remember(state.chargeLimit) { mutableFloatStateOf(state.chargeLimit.toFloat()) }
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text("停在 ", style = MaterialTheme.typography.bodyLarge)
                                    AnimatedNumber(
                                        value = sliderValue.toInt(),
                                        style = MaterialTheme.typography.bodyLarge,
                                        suffix = "%",
                                    )
                                }
                                Slider(
                                    value = sliderValue,
                                    onValueChange = { sliderValue = it },
                                    onValueChangeFinished = { viewModel.setChargeLimit(true, sliderValue.toInt()) },
                                    valueRange = 50f..100f,
                                    steps = 9,
                                    modifier = Modifier.fillMaxWidth(),
                                )

                                Spacer(modifier = Modifier.height(8.dp))
                                Row(
                                    modifier = Modifier.fillMaxWidth(),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Row(verticalAlignment = Alignment.CenterVertically) {
                                        Text("恢复充电差值： ", style = MaterialTheme.typography.bodyLarge)
                                        AnimatedNumber(
                                            value = state.hysteresis,
                                            style = MaterialTheme.typography.bodyLarge,
                                            suffix = "%",
                                        )
                                    }
                                    Row(verticalAlignment = Alignment.CenterVertically) {
                                        IconButton(
                                            onClick = {
                                                val newVal = (state.hysteresis - 1).coerceAtLeast(1)
                                                viewModel.setChargeLimit(true, state.chargeLimit, newVal)
                                            },
                                            enabled = state.hysteresis > 1,
                                        ) { Text("−", style = MaterialTheme.typography.titleLarge) }
                                        IconButton(
                                            onClick = {
                                                val newVal = (state.hysteresis + 1).coerceAtMost(20)
                                                viewModel.setChargeLimit(true, state.chargeLimit, newVal)
                                            },
                                            enabled = state.hysteresis < 20,
                                        ) { Text("+", style = MaterialTheme.typography.titleLarge) }
                                    }
                                }
                            }

                            Spacer(modifier = Modifier.height(4.dp))
                            if (state.chargeLimitEnabled) {
                                Text(
                                    "电量达到 ${state.chargeLimit}% 时停止充电，降至 ${state.chargeLimit - state.hysteresis}% 时恢复。\n\n" +
                                        "恢复差值用于避免充电器频繁启停。" +
                                        "较小的差值能让电量更接近目标值，但启停更频繁。" +
                                        "较大的差值可减少启停次数，但电量波动更大。\n\n默认值：5%",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            } else {
                                Text(
                                    "在设定级别停止充电。延长电池寿命。",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                }

                // Other toggles
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("电源设置", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                        Spacer(modifier = Modifier.height(8.dp))

                        if (state.powerSaveLoaded) {
                            ToggleRow("省电模式", state.powerSave) { viewModel.togglePowerSave(it) }
                        }
                        if (state.fastBootLoaded) {
                            ToggleRow("快速启动", state.fastBoot) { viewModel.toggleFastBoot(it) }
                        }
                        if (state.autoSleepLoaded) {
                            ToggleRow("自动睡眠", state.autoSleepEnabled) {
                                viewModel.setAutoSleep(it)
                            }
                            if (state.autoSleepEnabled) {
                                Spacer(modifier = Modifier.height(4.dp))
                                val presets = listOf(5, 10, 20, 30, 60, 120)
                                val currentMinutes = state.autoSleepTimeout.toIntOrNull() ?: 5
                                val currentIndex = presets.indexOf(currentMinutes).coerceAtLeast(0)
                                var sliderIndex by remember(currentIndex) { mutableFloatStateOf(currentIndex.toFloat()) }
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text("空闲超时： ", style = MaterialTheme.typography.bodyLarge)
                                    AnimatedNumber(
                                        value = presets[sliderIndex.toInt().coerceIn(0, presets.lastIndex)],
                                        style = MaterialTheme.typography.bodyLarge,
                                        suffix = " min",
                                    )
                                }
                                Slider(
                                    value = sliderIndex,
                                    onValueChange = { sliderIndex = it },
                                    onValueChangeFinished = {
                                        val selected = presets[sliderIndex.toInt().coerceIn(0, presets.lastIndex)]
                                        viewModel.setAutoSleep(true, "$selected")
                                    },
                                    valueRange = 0f..(presets.size - 1).toFloat(),
                                    steps = presets.size - 2,
                                    enabled = !controlsDisabled,
                                    modifier = Modifier.fillMaxWidth(),
                                )
                                Spacer(modifier = Modifier.height(4.dp))
                                Text(
                                    "当没有 WiFi 客户端连接并且空闲超时到期时休眠。",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                }

                // Reboot & Reset
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("设备操作", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                        Spacer(modifier = Modifier.height(12.dp))
                        Button(
                            onClick = { viewModel.showRebootConfirm() },
                            enabled = !controlsDisabled,
                            modifier = Modifier.fillMaxWidth(),
                        ) { Text("重启设备") }
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedButton(
                            onClick = { viewModel.showResetConfirm() },
                            enabled = !controlsDisabled,
                            modifier = Modifier.fillMaxWidth(),
                            colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error),
                        ) { Text("恢复出厂设置") }
                    }
                }
            }
        }
    }
}

@Composable
private fun ToggleRow(label: String, checked: Boolean, onToggle: (Boolean) -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, style = MaterialTheme.typography.bodyLarge)
        Switch(checked = checked, onCheckedChange = onToggle)
    }
}
