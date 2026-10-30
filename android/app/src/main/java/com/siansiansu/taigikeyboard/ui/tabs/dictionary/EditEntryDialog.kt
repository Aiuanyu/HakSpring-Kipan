package com.siansiansu.taigikeyboard.ui.tabs.dictionary

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshots.SnapshotStateMap
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.siansiansu.taigikeyboard.engine.HakfaLects
import com.siansiansu.taigikeyboard.i18n.generated.L10n
import com.siansiansu.taigikeyboard.ime.dictionary.CustomWord
import kotlinx.coroutines.launch

// Hakfa fork: Hakfa-only wording (no i18n key), like the lect screen.
private const val HANJI_FIELD_LABEL = "諸夏字"
private const val AUTO_FILL_LABEL = "各腔自動補音"

/** The field label for a lect's reading: 「打四縣」 (the short name, not the romanization). */
private fun readingLabel(name: String): String = "打" + lectShortName(name)

/** 「四縣」 from 「四縣 (Xi'ien)」. */
private fun lectShortName(name: String): String = name.substringBefore(" (")

// Dialog for adding or editing a custom word: the Hanji and one reading per lect, as the Mac sheet.
// The fields come from [HakfaLects.all], so a lect added there gets its field here.
@Composable
internal fun EditEntryDialog(
    word: CustomWord?,
    onDismiss: () -> Unit,
    onSave: (hanji: String, romans: Map<Int, String>) -> Unit,
    lookup: suspend (hanji: String, lect: Int) -> ReadingLookup,
) {
    var hanji by remember(word) { mutableStateOf(word?.hanji ?: "") }
    val romans: SnapshotStateMap<Int, String> =
        remember(word) {
            mutableStateMapOf<Int, String>().also { map ->
                HakfaLects.all.forEach { map[it.code] = word?.roman(it.code) ?: "" }
            }
        }
    val scope = rememberCoroutineScope()
    // The 「各腔自動補音」 lookup: a line saying what it is doing, then what it did.
    var autoFillNote by remember(word) { mutableStateOf<String?>(null) }
    var isLookingUp by remember(word) { mutableStateOf(false) }
    val isEditing = word != null
    val canSave = romans.values.any { it.isNotBlank() }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                if (isEditing) L10n.dictionaryEditEntry else L10n.dictionaryAddEntry,
            )
        },
        text = {
            Column(
                modifier = Modifier.verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                OutlinedTextField(
                    value = hanji,
                    onValueChange = { hanji = it },
                    label = { Text(HANJI_FIELD_LABEL) },
                    placeholder = { Text(L10n.dictionaryHanziPlaceholder) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )
                HakfaLects.all.forEach { lect ->
                    OutlinedTextField(
                        value = romans[lect.code].orEmpty(),
                        onValueChange = { romans[lect.code] = it },
                        label = { Text(readingLabel(lect.name)) },
                        placeholder = { Text(L10n.dictionaryRomanPlaceholder) },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
                TextButton(
                    onClick = {
                        val text = hanji.trim()
                        isLookingUp = true
                        autoFillNote = "查詢中⋯"
                        scope.launch {
                            val missing = HakfaLects.all.filter { romans[it.code].isNullOrBlank() }
                            val notes = mutableListOf<String>()
                            val filled = mutableListOf<String>()
                            missing.forEach { lect ->
                                val name = lectShortName(lect.name)
                                when (val found = lookup(text, lect.code)) {
                                    is ReadingLookup.Found -> {
                                        romans[lect.code] = found.roman
                                        filled += name
                                    }
                                    is ReadingLookup.Several -> notes += "$name：有 ${found.count} 種讀音，請自家揀"
                                    ReadingLookup.None -> notes += "$name：詞典無這隻字"
                                    ReadingLookup.Failed -> notes += "$name：查詢失敗"
                                }
                            }
                            autoFillNote =
                                when {
                                    missing.isEmpty() -> "各腔都有讀音，無使補"
                                    else -> (listOfNotNull(filled.takeIf { it.isNotEmpty() }?.let { "已補：${it.joinToString("、")}" }) + notes).joinToString("\n")
                                }
                            isLookingUp = false
                        }
                    },
                    enabled = hanji.isNotBlank() && !isLookingUp,
                ) {
                    Text(if (isLookingUp) "查詢中⋯" else AUTO_FILL_LABEL)
                }
                autoFillNote?.let { note ->
                    Text(
                        text = note,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = { onSave(hanji.trim(), romans.toMap()) },
                enabled = canSave,
            ) {
                Text(L10n.commonSave)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(L10n.commonCancel)
            }
        },
    )
}
