package com.openu60.feature.config

import android.app.Activity
import android.content.Intent
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.hilt.navigation.compose.hiltViewModel
import com.openu60.core.model.ConfigHeader
import com.openu60.core.model.KnownKey

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConfigToolScreen(
    onBack: () -> Unit,
    viewModel: ConfigToolViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()
    val serial by viewModel.serial.collectAsState()
    val customKey by viewModel.customKey.collectAsState()

    val importLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenDocument()
    ) { uri ->
        uri?.let { viewModel.importFile(it) }
    }

    val exportLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.CreateDocument("application/octet-stream")
    ) { uri ->
        uri?.let { viewModel.exportEncrypted(it) }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("配置工具") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回")
                    }
                },
                actions = {
                    if (state.fileName != null) {
                        IconButton(onClick = { viewModel.clear() }) {
                            Icon(Icons.Default.Clear, contentDescription = "清除")
                        }
                    }
                },
            )
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (state.error != null) {
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer,
                    ),
                ) {
                    Text(
                        state.error!!,
                        modifier = Modifier.padding(16.dp),
                        color = MaterialTheme.colorScheme.onErrorContainer,
                    )
                }
            }

            if (state.fileName == null) {
                // Import section
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(24.dp),
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        Icon(
                            Icons.Default.FileOpen,
                            contentDescription = null,
                            modifier = Modifier.size(48.dp),
                            tint = MaterialTheme.colorScheme.primary,
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        Text(
                            "导入配置文件",
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            "选择要解密的ZXHN config.bin文件",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Button(
                            onClick = { importLauncher.launch(arrayOf("*/*")) },
                        ) {
                            Icon(Icons.Default.Upload, contentDescription = null)
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("选择文件")
                        }
                    }
                }
            } else {
                // Header info
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(
                            "文件信息",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        InfoRow("名称", state.fileName!!)
                        InfoRow("Size", "${state.fileSize} bytes")
                        state.header?.let { header ->
                            InfoRow("Magic", header.magic)
                            InfoRow("类型", header.payloadTypeName)
                            InfoRow("Signature", header.signature.ifBlank { "(none)" })
                            InfoRow("Payload offset", "${header.payloadOffset}")
                        }
                    }
                }

                // Key input
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text(
                            "解密密钥",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = serial,
                            onValueChange = viewModel::updateSerial,
                            label = { Text("设备序列号（可选）") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = customKey,
                            onValueChange = viewModel::updateCustomKey,
                            label = { Text("自定义密钥（可选）") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            "留空以尝试所有 ${KnownKey.KNOWN_KEYS.size} 已知密钥 + 派生密钥",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }

                // Decrypt button
                Button(
                    onClick = { viewModel.decrypt() },
                    enabled = !state.isLoading,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    if (state.isLoading) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(20.dp),
                            strokeWidth = 2.dp,
                            color = MaterialTheme.colorScheme.onPrimary,
                        )
                        Spacer(modifier = Modifier.width(8.dp))
                    }
                    Icon(Icons.Default.LockOpen, contentDescription = null)
                    Spacer(modifier = Modifier.width(8.dp))
                    Text("解密")
                }

                // Decrypted XML viewer
                if (state.decryptedXml != null) {
                    if (state.usedKey != null) {
                        Text(
                            "密钥：${state.usedKey}",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.primary,
                        )
                    }

                    Card(modifier = Modifier.fillMaxWidth()) {
                        Column(modifier = Modifier.padding(16.dp)) {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text(
                                    "解密的 XML",
                                    style = MaterialTheme.typography.titleMedium,
                                    fontWeight = FontWeight.Bold,
                                )
                                Text(
                                    "${state.decryptedXml!!.length} chars",
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            val preview = state.decryptedXml!!.take(5000)
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .heightIn(max = 400.dp)
                                    .horizontalScroll(rememberScrollState()),
                            ) {
                                Text(
                                    preview + if (state.decryptedXml!!.length > 5000) "\n... (truncated)" else "",
                                    fontFamily = FontFamily.Monospace,
                                    fontSize = 10.sp,
                                    lineHeight = 14.sp,
                                )
                            }
                        }
                    }

                    // Re-encrypt section
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        OutlinedButton(
                            onClick = { viewModel.encrypt(ConfigHeader.PAYLOAD_TYPE_ECB) },
                            enabled = !state.isLoading,
                            modifier = Modifier.weight(1f),
                        ) {
                            Text("重新加密ECB")
                        }
                        OutlinedButton(
                            onClick = { viewModel.encrypt(ConfigHeader.PAYLOAD_TYPE_CBC) },
                            enabled = !state.isLoading,
                            modifier = Modifier.weight(1f),
                        ) {
                            Text("重新加密CBC")
                        }
                    }

                    if (state.encryptedOutput != null) {
                        Button(
                            onClick = {
                                exportLauncher.launch(state.encryptedFileName ?: "config_encrypted.bin")
                            },
                            modifier = Modifier.fillMaxWidth(),
                        ) {
                            Icon(Icons.Default.Download, contentDescription = null)
                            Spacer(modifier = Modifier.width(8.dp))
                            Text("导出加密文件")
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun InfoRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(
            label,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Text(
            value,
            style = MaterialTheme.typography.bodyMedium,
            fontFamily = FontFamily.Monospace,
        )
    }
}
