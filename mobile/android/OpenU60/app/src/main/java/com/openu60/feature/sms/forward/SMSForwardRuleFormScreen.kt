package com.openu60.feature.sms.forward

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.openu60.core.model.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SMSForwardRuleFormScreen(
    onBack: () -> Unit,
    viewModel: SMSForwardViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    var name by remember { mutableStateOf("") }

    // Filter state
    var filterType by remember { mutableStateOf("all") }
    var senderPatterns by remember { mutableStateOf("") }
    var contentKeywords by remember { mutableStateOf("") }

    // Destination state
    var destType by remember { mutableStateOf("telegram") }
    var destExpanded by remember { mutableStateOf(false) }

    // Telegram
    var tgBotToken by remember { mutableStateOf("") }
    var tgChatId by remember { mutableStateOf("") }
    var tgSilent by remember { mutableStateOf(false) }

    // Webhook
    var webhookUrl by remember { mutableStateOf("") }
    var webhookMethod by remember { mutableStateOf("POST") }
    var webhookMethodExpanded by remember { mutableStateOf(false) }
    var webhookHeaders by remember { mutableStateOf("") }

    // SMS
    var smsNumber by remember { mutableStateOf("") }

    // ntfy
    var ntfyUrl by remember { mutableStateOf("https://ntfy.sh") }
    var ntfyTopic by remember { mutableStateOf("") }
    var ntfyToken by remember { mutableStateOf("") }

    // Discord
    var discordUrl by remember { mutableStateOf("") }

    // Slack
    var slackUrl by remember { mutableStateOf("") }

    val destTypes = listOf("telegram", "webhook", "sms", "ntfy", "discord", "slack")
    val destLabels = listOf("Telegram", "Webhook", "短信", "ntfy", "Discord", "Slack")

    fun buildFilter(): SmsFilter = when (filterType) {
        "sender" -> SmsFilter.Sender(senderPatterns.split(",").map { it.trim() }.filter { it.isNotEmpty() })
        "content" -> SmsFilter.Content(contentKeywords.split(",").map { it.trim() }.filter { it.isNotEmpty() })
        "sender_and_content" -> SmsFilter.SenderAndContent(
            patterns = senderPatterns.split(",").map { it.trim() }.filter { it.isNotEmpty() },
            keywords = contentKeywords.split(",").map { it.trim() }.filter { it.isNotEmpty() },
        )
        else -> SmsFilter.All
    }

    fun buildDestination(): ForwardDestination? = when (destType) {
        "telegram" -> if (tgBotToken.isNotBlank() && tgChatId.isNotBlank()) {
            ForwardDestination.Telegram(tgBotToken.trim(), tgChatId.trim(), tgSilent)
        } else null
        "webhook" -> if (webhookUrl.isNotBlank()) {
            val headers = webhookHeaders.lines()
                .mapNotNull { line ->
                    val parts = line.split(":", limit = 2)
                    if (parts.size == 2) HttpHeader(parts[0].trim(), parts[1].trim()) else null
                }
            ForwardDestination.Webhook(webhookUrl.trim(), webhookMethod, headers)
        } else null
        "sms" -> if (smsNumber.isNotBlank()) ForwardDestination.Sms(smsNumber.trim()) else null
        "ntfy" -> if (ntfyUrl.isNotBlank() && ntfyTopic.isNotBlank()) {
            ForwardDestination.Ntfy(ntfyUrl.trim(), ntfyTopic.trim(), ntfyToken.ifBlank { null })
        } else null
        "discord" -> if (discordUrl.isNotBlank()) ForwardDestination.Discord(discordUrl.trim()) else null
        "slack" -> if (slackUrl.isNotBlank()) ForwardDestination.Slack(slackUrl.trim()) else null
        else -> null
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("新规则") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "返回")
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
            // Message
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

            // Name
            OutlinedTextField(
                value = name,
                onValueChange = { name = it },
                label = { Text("规则名称") },
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )

            // Filter card
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("筛选", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    Spacer(modifier = Modifier.height(8.dp))

                    listOf(
                        "all" to "全部短信",
                        "sender" to "按发件人",
                        "content" to "按内容",
                        "sender_and_content" to "By Sender & Content",
                    ).forEach { (value, label) ->
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            RadioButton(
                                selected = filterType == value,
                                onClick = { filterType = value },
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(label)
                        }
                    }

                    if (filterType == "sender" || filterType == "sender_and_content") {
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = senderPatterns,
                            onValueChange = { senderPatterns = it },
                            label = { Text("发件人模式（逗号分隔）") },
                            modifier = Modifier.fillMaxWidth(),
                            singleLine = true,
                        )
                    }

                    if (filterType == "content" || filterType == "sender_and_content") {
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedTextField(
                            value = contentKeywords,
                            onValueChange = { contentKeywords = it },
                            label = { Text("关键字（以逗号分隔）") },
                            modifier = Modifier.fillMaxWidth(),
                            singleLine = true,
                        )
                    }
                }
            }

            // Destination card
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("目的地", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    Spacer(modifier = Modifier.height(8.dp))

                    ExposedDropdownMenuBox(
                        expanded = destExpanded,
                        onExpandedChange = { destExpanded = it },
                    ) {
                        OutlinedTextField(
                            value = destLabels[destTypes.indexOf(destType)],
                            onValueChange = {},
                            readOnly = true,
                            label = { Text("类型") },
                            trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = destExpanded) },
                            modifier = Modifier.fillMaxWidth().menuAnchor(),
                        )
                        ExposedDropdownMenu(
                            expanded = destExpanded,
                            onDismissRequest = { destExpanded = false },
                        ) {
                            destTypes.forEachIndexed { index, type ->
                                DropdownMenuItem(
                                    text = { Text(destLabels[index]) },
                                    onClick = {
                                        destType = type
                                        destExpanded = false
                                    },
                                )
                            }
                        }
                    }

                    Spacer(modifier = Modifier.height(12.dp))

                    when (destType) {
                        "telegram" -> {
                            OutlinedTextField(
                                value = tgBotToken,
                                onValueChange = { tgBotToken = it },
                                label = { Text("机器人令牌") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            OutlinedTextField(
                                value = tgChatId,
                                onValueChange = { tgChatId = it },
                                label = { Text("聊天 ID") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                Text("静默通知")
                                Switch(checked = tgSilent, onCheckedChange = { tgSilent = it })
                            }
                        }
                        "webhook" -> {
                            OutlinedTextField(
                                value = webhookUrl,
                                onValueChange = { webhookUrl = it },
                                label = { Text("URL") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            ExposedDropdownMenuBox(
                                expanded = webhookMethodExpanded,
                                onExpandedChange = { webhookMethodExpanded = it },
                            ) {
                                OutlinedTextField(
                                    value = webhookMethod,
                                    onValueChange = {},
                                    readOnly = true,
                                    label = { Text("方法") },
                                    trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = webhookMethodExpanded) },
                                    modifier = Modifier.fillMaxWidth().menuAnchor(),
                                )
                                ExposedDropdownMenu(
                                    expanded = webhookMethodExpanded,
                                    onDismissRequest = { webhookMethodExpanded = false },
                                ) {
                                    listOf("POST", "PUT").forEach { method ->
                                        DropdownMenuItem(
                                            text = { Text(method) },
                                            onClick = {
                                                webhookMethod = method
                                                webhookMethodExpanded = false
                                            },
                                        )
                                    }
                                }
                            }
                            Spacer(modifier = Modifier.height(8.dp))
                            OutlinedTextField(
                                value = webhookHeaders,
                                onValueChange = { webhookHeaders = it },
                                label = { Text("标题（名称：每行值）") },
                                modifier = Modifier.fillMaxWidth(),
                                minLines = 2,
                                maxLines = 5,
                            )
                        }
                        "sms" -> {
                            OutlinedTextField(
                                value = smsNumber,
                                onValueChange = { smsNumber = it },
                                label = { Text("转接号码") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                        }
                        "ntfy" -> {
                            OutlinedTextField(
                                value = ntfyUrl,
                                onValueChange = { ntfyUrl = it },
                                label = { Text("服务器地址") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            OutlinedTextField(
                                value = ntfyTopic,
                                onValueChange = { ntfyTopic = it },
                                label = { Text("主题") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            OutlinedTextField(
                                value = ntfyToken,
                                onValueChange = { ntfyToken = it },
                                label = { Text("令牌（可选）") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                        }
                        "discord" -> {
                            OutlinedTextField(
                                value = discordUrl,
                                onValueChange = { discordUrl = it },
                                label = { Text("Webhook URL") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                        }
                        "slack" -> {
                            OutlinedTextField(
                                value = slackUrl,
                                onValueChange = { slackUrl = it },
                                label = { Text("Webhook URL") },
                                modifier = Modifier.fillMaxWidth(),
                                singleLine = true,
                            )
                        }
                    }
                }
            }

            // Action buttons
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                OutlinedButton(
                    onClick = {
                        val dest = buildDestination()
                        if (dest != null) {
                            viewModel.testDestination(dest)
                        }
                    },
                    enabled = buildDestination() != null && !state.isLoading,
                    modifier = Modifier.weight(1f),
                ) { Text("测试") }

                Button(
                    onClick = {
                        val filter = buildFilter()
                        val dest = buildDestination() ?: return@Button
                        viewModel.createRule(name.trim(), filter, dest)
                        onBack()
                    },
                    enabled = name.isNotBlank() && buildDestination() != null && !state.isLoading,
                    modifier = Modifier.weight(1f),
                ) { Text("保存") }
            }
        }
    }
}
