package com.openu60.feature.router

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RouterSettingsListScreen(
    onNavigateToMobileNetwork: () -> Unit,
    onNavigateToNetworkMode: () -> Unit,
    onNavigateToCellLock: () -> Unit,
    onNavigateToSTC: () -> Unit,
    onNavigateToSignalDetect: () -> Unit,
    onNavigateToSIM: () -> Unit,
    onNavigateToRouterESim: () -> Unit,
    onNavigateToSTK: () -> Unit,
    onNavigateToWiFi: () -> Unit,
    onNavigateToGuestWiFi: () -> Unit,
    onNavigateToAPN: () -> Unit,
    onNavigateToLAN: () -> Unit,
    onNavigateToDNS: () -> Unit,
    onNavigateToFirewall: () -> Unit,
    onNavigateToTelemetryBlocker: () -> Unit,
    onNavigateToVPNPassthrough: () -> Unit,
    onNavigateToQoS: () -> Unit,
    onNavigateToDeviceControl: () -> Unit,
    onNavigateToScheduleReboot: () -> Unit,
) {
    Scaffold(
        topBar = {
            TopAppBar(title = { Text("路由器设置") })
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            // Cellular
            SectionHeader("蜂窝网络")
            SettingsItem(Icons.Default.CellTower, "移动网络", onClick = onNavigateToMobileNetwork)
            SettingsItem(Icons.Default.SettingsInputAntenna, "网络模式", onClick = onNavigateToNetworkMode)
            SettingsItem(Icons.Default.Lock, "小区锁定", onClick = onNavigateToCellLock)
            SettingsItem(Icons.Default.Hub, "STC", onClick = onNavigateToSTC)
            SettingsItem(Icons.Default.Radar, "信号检测", onClick = onNavigateToSignalDetect)
            SettingsItem(Icons.Default.SimCard, "SIM 卡", onClick = onNavigateToSIM)
            SettingsItem(Icons.Default.SimCardDownload, "eSIM 配置文件", onClick = onNavigateToRouterESim)
            SettingsItem(Icons.Default.Dialpad, "SIM Services (STK)", onClick = onNavigateToSTK)

            Spacer(modifier = Modifier.height(8.dp))

            // Connectivity
            SectionHeader("网络连接")
            SettingsItem(Icons.Default.Wifi, "WiFi", onClick = onNavigateToWiFi)
            SettingsItem(Icons.Default.WifiTethering, "访客 WiFi", onClick = onNavigateToGuestWiFi)
            SettingsItem(Icons.Default.Language, "APN", onClick = onNavigateToAPN)
            SettingsItem(Icons.Default.Router, "LAN / DHCP", onClick = onNavigateToLAN)
            SettingsItem(Icons.Default.Dns, "DNS", onClick = onNavigateToDNS)

            Spacer(modifier = Modifier.height(8.dp))

            // Security
            SectionHeader("安全")
            SettingsItem(Icons.Default.Shield, "防火墙", onClick = onNavigateToFirewall)
            SettingsItem(Icons.Default.VisibilityOff, "遥测拦截", onClick = onNavigateToTelemetryBlocker)
            SettingsItem(Icons.Default.VpnKey, "VPN 穿透", onClick = onNavigateToVPNPassthrough)

            Spacer(modifier = Modifier.height(8.dp))

            // Quality
            SectionHeader("服务质量")
            SettingsItem(Icons.Default.Speed, "服务质量", onClick = onNavigateToQoS)

            Spacer(modifier = Modifier.height(8.dp))

            // System
            SectionHeader("系统")
            SettingsItem(Icons.Default.SettingsPower, "设备控制", onClick = onNavigateToDeviceControl)
            SettingsItem(Icons.Default.Schedule, "定时重启", onClick = onNavigateToScheduleReboot)
        }
    }
}

@Composable
private fun SectionHeader(title: String) {
    Text(
        title,
        style = MaterialTheme.typography.titleSmall,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(bottom = 4.dp),
    )
}

@Composable
private fun SettingsItem(
    icon: ImageVector,
    title: String,
    onClick: () -> Unit,
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
    ) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                icon,
                contentDescription = null,
                modifier = Modifier.size(24.dp),
                tint = MaterialTheme.colorScheme.primary,
            )
            Spacer(modifier = Modifier.width(16.dp))
            Text(
                title,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.weight(1f),
            )
            Icon(
                Icons.Default.ChevronRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}
