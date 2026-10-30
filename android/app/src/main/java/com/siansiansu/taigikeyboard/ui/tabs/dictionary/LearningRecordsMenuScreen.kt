package com.siansiansu.taigikeyboard.ui.tabs.dictionary

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import com.siansiansu.taigikeyboard.engine.proto.LearningRecordKind
import com.siansiansu.taigikeyboard.engine.proto.LectsSynced
import com.siansiansu.taigikeyboard.i18n.LocalStringResolver
import com.siansiansu.taigikeyboard.i18n.generated.L10n
import com.siansiansu.taigikeyboard.i18n.generated.StringKey
import com.siansiansu.taigikeyboard.ui.components.ActionRow
import com.siansiansu.taigikeyboard.ui.components.ResultDialog
import com.siansiansu.taigikeyboard.ui.components.SettingsCard
import com.siansiansu.taigikeyboard.ui.components.SettingsDivider
import com.siansiansu.taigikeyboard.ui.components.resultMessage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch

// Learning Records — one row per kind (word frequency, learned phrases), each opening that kind's list,
// then Delete Learning Records: every learning store at once (next-word association included), custom words kept.
// [onClearAll] throws for the screen to report.
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LearningRecordsMenuScreen(
    onNavigateBack: () -> Unit,
    onKind: (LearningRecordKind) -> Unit,
    onClearAll: suspend () -> Unit,
    // Hakfa fork (各腔互學): the preview counts what would be copied, the confirm copies. Both throw for the screen to report.
    onSyncPreview: suspend () -> LectsSynced,
    onSyncConfirm: suspend () -> LectsSynced,
) {
    val scope = rememberCoroutineScope()
    // Read after the clear returns; tracks the live display language.
    val stringResolver by rememberUpdatedState(LocalStringResolver.current)
    // While the clear runs, a list opened now could read the rows it is emptying: the kind rows wait too.
    var isClearing by remember { mutableStateOf(false) }
    // Only a failure is said; a clear that landed says nothing (USER 2026-10-16).
    var clearFailure by remember { mutableStateOf<String?>(null) }

    // Hakfa fork (各腔互學): the preview waits for the user's confirmation before anything is copied.
    var syncPreview by remember { mutableStateOf<LectsSynced?>(null) }
    var syncFailure by remember { mutableStateOf<String?>(null) }

    fun previewSync() {
        isClearing = true
        scope.launch {
            try {
                syncPreview = onSyncPreview()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                syncFailure = resultMessage("各腔互學無成功", e.message)
            } finally {
                isClearing = false
            }
        }
    }

    fun confirmSync() {
        syncPreview = null
        isClearing = true
        scope.launch {
            try {
                onSyncConfirm()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                syncFailure = resultMessage("各腔互學無成功", e.message)
            } finally {
                isClearing = false
            }
        }
    }

    fun clearAll() {
        isClearing = true
        scope.launch {
            try {
                onClearAll()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                clearFailure =
                    resultMessage(stringResolver.resolve(StringKey.DICTIONARY_CLEAR_LEARNING_RECORDS_FAILED), e.message)
            } finally {
                isClearing = false
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = L10n.dictionaryLearningRecords,
                        fontWeight = FontWeight.Bold,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = L10n.commonBack)
                    }
                },
                colors =
                    TopAppBarDefaults.topAppBarColors(
                        containerColor = MaterialTheme.colorScheme.surfaceContainer,
                    ),
            )
        },
        containerColor = MaterialTheme.colorScheme.surfaceContainer,
    ) { padding ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 20.dp),
        ) {
            Spacer(Modifier.height(8.dp))
            SettingsCard {
                ActionRow(
                    label = kindLabel(LearningRecordKind.LEARNING_RECORD_KIND_FREQUENCY),
                    onClick = { if (!isClearing) onKind(LearningRecordKind.LEARNING_RECORD_KIND_FREQUENCY) },
                    trailingIcon = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                )
                SettingsDivider()
                ActionRow(
                    label = kindLabel(LearningRecordKind.LEARNING_RECORD_KIND_LEARNED_PHRASE),
                    onClick = { if (!isClearing) onKind(LearningRecordKind.LEARNING_RECORD_KIND_LEARNED_PHRASE) },
                    trailingIcon = Icons.AutoMirrored.Filled.KeyboardArrowRight,
                )
            }
            Spacer(Modifier.height(16.dp))
            // Hakfa fork: 各腔互學, one row; the preview dialog asks before copying.
            SettingsCard {
                ActionRow(
                    label = "各腔互學",
                    onClick = { if (!isClearing) previewSync() },
                )
            }
            Spacer(Modifier.height(16.dp))
            // Runs on the tap, no confirmation (USER 2026-10-16).
            SettingsCard {
                ActionRow(
                    label = L10n.dictionaryClearLearningRecords,
                    onClick = { if (!isClearing) clearAll() },
                    textColor = MaterialTheme.colorScheme.error,
                )
            }
        }
    }

    syncPreview?.let { preview ->
        AlertDialog(
            onDismissRequest = { syncPreview = null },
            title = { Text("各腔互學") },
            text = { Text(lectsSyncSummary(preview)) },
            confirmButton = {
                TextButton(onClick = { confirmSync() }) { Text("互學") }
            },
            dismissButton = {
                TextButton(onClick = { syncPreview = null }) { Text(L10n.commonCancel) }
            },
        )
    }

    syncFailure?.let { message ->
        ResultDialog(
            message = message,
            confirmLabel = L10n.commonOk,
            onDismiss = { syncFailure = null },
        )
    }

    clearFailure?.let { message ->
        ResultDialog(
            message = message,
            confirmLabel = L10n.commonOk,
            onDismiss = { clearFailure = null },
        )
    }
}

/** What the preview says before anything is copied (the macOS Learning Records page says the same). */
internal fun lectsSyncSummary(preview: LectsSynced): String {
    val added = preview.addedWords + preview.addedPhrases
    val lines = mutableListOf("會增加 $added 筆（詞 ${preview.addedWords}、詞組 ${preview.addedPhrases}）。")
    if (preview.composed > 0) lines += "其中 ${preview.composed} 筆係用目的腔詞典个詞組合出來个。"
    if (preview.alreadyPresent > 0) lines += "本來就有：${preview.alreadyPresent} 筆。"
    if (preview.skippedMissing > 0) lines += "目的腔無共隻漢字，跳過：${preview.skippedMissing} 筆。"
    if (preview.skippedAmbiguous > 0) lines += "目的腔有多種讀音，跳過：${preview.skippedAmbiguous} 筆。"
    return lines.joinToString("\n")
}

