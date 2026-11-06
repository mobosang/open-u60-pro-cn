package com.openu60.feature.router.stk

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun STKScreen(
    onBack: () -> Unit,
    viewModel: STKViewModel = hiltViewModel(),
) {
    val state by viewModel.state.collectAsState()

    LaunchedEffect(viewModel) { viewModel.watchSTK() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("STK / USSD") },
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

            // USSD Section
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text("USSD", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                    Spacer(modifier = Modifier.height(8.dp))

                    OutlinedTextField(
                        value = state.ussdCode,
                        onValueChange = { viewModel.updateUssdCode(it) },
                        label = { Text("USSD 代码（例如*#06#）") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Button(
                        onClick = { viewModel.sendUSSD() },
                        enabled = !state.isLoading && state.ussdCode.isNotBlank(),
                        modifier = Modifier.fillMaxWidth(),
                    ) { Text("发送USSD") }

                    if (state.showUssdResponse) {
                        Spacer(modifier = Modifier.height(12.dp))
                        Card(
                            modifier = Modifier.fillMaxWidth(),
                            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
                        ) {
                            Column(modifier = Modifier.padding(12.dp)) {
                                Text("回应", fontWeight = FontWeight.Medium)
                                Spacer(modifier = Modifier.height(4.dp))
                                Text(state.ussdResponse.response.ifBlank { state.ussdResponse.rawResponse })

                                if (state.ussdResponse.sessionActive) {
                                    Spacer(modifier = Modifier.height(8.dp))
                                    OutlinedTextField(
                                        value = state.ussdReply,
                                        onValueChange = { viewModel.updateUssdReply(it) },
                                        label = { Text("回复") },
                                        modifier = Modifier.fillMaxWidth(),
                                        singleLine = true,
                                    )
                                    Spacer(modifier = Modifier.height(8.dp))
                                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                        Button(
                                            onClick = { viewModel.respondUSSD() },
                                            enabled = !state.isLoading && state.ussdReply.isNotBlank(),
                                            modifier = Modifier.weight(1f),
                                        ) { Text("回复") }
                                        OutlinedButton(
                                            onClick = { viewModel.cancelUSSD() },
                                            enabled = !state.isLoading,
                                            modifier = Modifier.weight(1f),
                                        ) { Text("取消") }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // STK Section
            if (!state.stkNotSupported) {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                        ) {
                            Text(
                                state.stkMenu.title.ifBlank { "STK Menu" },
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.Bold,
                            )
                            if (!state.stkRootMenu && state.stkType != "status") {
                                TextButton(enabled = !state.isLoading && !state.stkBusy, onClick = { viewModel.goBackSTK() }) { Text("返回") }
                            }
                        }

                        if (state.isLoading) {
                            Spacer(modifier = Modifier.height(8.dp))
                            LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
                        }

                        Spacer(modifier = Modifier.height(8.dp))
                        if (state.stkReason.isNotEmpty()) {
                            Text(state.stkReason, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        if (state.stkText.isNotEmpty()) Text(state.stkText)
                        if (state.stkType == "input") {
                            if (state.stkYesNo) {
                                Row {
                                    TextButton(enabled = !state.isLoading && !state.stkBusy,
                                        onClick = { viewModel.respondSTK(true) }) { Text("是的") }
                                    TextButton(enabled = !state.isLoading && !state.stkBusy,
                                        onClick = { viewModel.respondSTK(false) }) { Text("No") }
                                }
                            } else {
                                OutlinedTextField(value = state.stkReply,
                                    onValueChange = viewModel::updateStkReply,
                                    label = { Text("SIM 回应") },
                                    enabled = !state.isLoading && !state.stkBusy,
                                    visualTransformation = if (state.stkHidden)
                                        androidx.compose.ui.text.input.PasswordVisualTransformation()
                                        else androidx.compose.ui.text.input.VisualTransformation.None,
                                    keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(
                                        keyboardType = if (state.stkDigitsOnly)
                                            androidx.compose.ui.text.input.KeyboardType.Phone
                                            else androidx.compose.ui.text.input.KeyboardType.Text),
                                )
                                Text("输入${state.stkMin}–${state.stkMax}字符")
                                TextButton(enabled = !state.isLoading && !state.stkBusy,
                                    onClick = { viewModel.respondSTK() }) { Text("发送响应") }
                            }
                        } else if (state.stkType == "display") {
                            TextButton(enabled = !state.isLoading && !state.stkBusy,
                                onClick = { viewModel.respondSTK() }) { Text("继续") }
                        }
                        if (!state.stkRootMenu && state.stkType != "status") {
                            TextButton(enabled = !state.isLoading && !state.stkBusy,
                                onClick = { viewModel.cancelSTK() }) { Text("结束 SIM 会话") }
                        }
                        if (state.stkType == "status") {
                            TextButton(enabled = !state.isLoading && !state.stkBusy,
                                onClick = { viewModel.loadSTKMenu() }) { Text("重新加载SIM菜单") }
                        }
                        state.stkMenu.items.forEach { item ->
                            Card(
                                modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
                                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
                                enabled = !state.isLoading && !state.stkBusy,
                                onClick = { viewModel.selectSTKItem(item) },
                            ) {
                                Text(
                                    item.label,
                                    modifier = Modifier.padding(12.dp),
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
