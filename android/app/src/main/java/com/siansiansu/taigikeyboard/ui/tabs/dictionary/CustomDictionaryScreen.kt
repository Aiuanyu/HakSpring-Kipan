package com.siansiansu.taigikeyboard.ui.tabs.dictionary

import android.net.Uri
import android.widget.Toast
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.siansiansu.taigikeyboard.engine.HakfaLects
import com.siansiansu.taigikeyboard.engine.proto.CustomDictionaryRefusal
import com.siansiansu.taigikeyboard.i18n.LocalStringResolver
import com.siansiansu.taigikeyboard.i18n.generated.L10n
import com.siansiansu.taigikeyboard.i18n.generated.StringKey
import com.siansiansu.taigikeyboard.i18n.generated.dictionaryImportResult
import com.siansiansu.taigikeyboard.ime.dictionary.CustomWord
import com.siansiansu.taigikeyboard.ime.dictionary.UserDataException
import com.siansiansu.taigikeyboard.ime.dictionary.groupCustomWords
import com.siansiansu.taigikeyboard.ime.dictionary.planWordSave
import com.siansiansu.taigikeyboard.ui.components.ActionRow
import com.siansiansu.taigikeyboard.ui.components.EmptyListCard
import com.siansiansu.taigikeyboard.ui.components.FileDownload
import com.siansiansu.taigikeyboard.ui.components.FileUpload
import com.siansiansu.taigikeyboard.ui.components.FilterSearchBar
import com.siansiansu.taigikeyboard.ui.components.LoadingRow
import com.siansiansu.taigikeyboard.ui.components.NoticeCard
import com.siansiansu.taigikeyboard.ui.components.ResultDialog
import com.siansiansu.taigikeyboard.ui.components.SettingInfoButton
import com.siansiansu.taigikeyboard.ui.components.SettingsCard
import com.siansiansu.taigikeyboard.ui.components.SettingsDivider
import com.siansiansu.taigikeyboard.ui.components.SwitchRow
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

private const val DISPLAY_LIMIT = 100

private const val IMPORTING_HINT = "匯入中⋯"
private const val EXPORTING_HINT = "匯出中⋯"

// The CSV header is the engine's (`CustomDictionaryCSV`): one column per lect.
private const val CSV_FORMAT_NOTE =
    "hanzi,xiien,namxiian,hoiliug\n" +
        "𠊎,ngai,ngai,ngoi\n" +
        "舊式兩欄 roman,hanji 也讀得（入四縣）"

// Custom dictionary management — add, edit, import/export, and delete entries
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomDictionaryScreen(
    viewModel: CustomDictionaryViewModel,
    onNavigateBack: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    // Captured for the non-Composable export/import callbacks; tracks the live display language.
    val stringResolver by rememberUpdatedState(LocalStringResolver.current)

    val entries by viewModel.entries.collectAsStateWithLifecycle()
    val isLoading by viewModel.isLoading.collectAsStateWithLifecycle()
    val isImporting by viewModel.isImporting.collectAsStateWithLifecycle()
    val isCustomDictEnabled by viewModel.isCustomDictEnabled.collectAsStateWithLifecycle()

    var showEditDialog by remember { mutableStateOf(false) }
    var editingEntry by remember { mutableStateOf<CustomWord?>(null) }
    // Hakfa fork: null = every lect; otherwise only words that have a reading in that lect.
    var lectFilter by remember { mutableStateOf<Int?>(null) }
    var showResultDialog by remember { mutableStateOf(false) }
    var resultMessage by remember { mutableStateOf("") }
    var filterText by remember { mutableStateOf("") }

    // Hakfa fork: the engine stores one row per (word, lect); the list shows one row per Hanji.
    val lectCodes = remember { HakfaLects.all.map { it.code } }
    val words = remember(entries) { groupCustomWords(entries, lectCodes) }
    val byLect = lectFilter?.let { lect -> words.filter { it.readings.containsKey(lect) } } ?: words
    val filteredEntries =
        if (filterText.isEmpty()) {
            byLect.take(DISPLAY_LIMIT)
        } else {
            val query = filterText.lowercase()
            byLect.filter { word ->
                word.hanji.lowercase().contains(query) ||
                    word.readings.values.any { it.roman.lowercase().contains(query) }
            }
        }

    LaunchedEffect(Unit) { viewModel.load() }

    // File import launcher
    val importLauncher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.OpenDocument(),
        ) { uri: Uri? ->
            uri ?: return@rememberLauncherForActivityResult
            scope.launch {
                // Hakfa fork: a small hint when it starts and when it is done (failures keep the dialog).
                Toast.makeText(context, IMPORTING_HINT, Toast.LENGTH_SHORT).show()
                try {
                    val result = viewModel.importFile(uri)
                    Toast
                        .makeText(
                            context,
                            stringResolver.dictionaryImportResult(imported = result.imported, skipped = result.skipped),
                            Toast.LENGTH_LONG,
                        ).show()
                } catch (e: Exception) {
                    resultMessage =
                        when ((e as? UserDataException.Refused)?.refusal) {
                            CustomDictionaryRefusal.CUSTOM_DICTIONARY_REFUSAL_FILE_TOO_LARGE ->
                                stringResolver.resolve(StringKey.DICTIONARY_FILE_TOO_LARGE)
                            CustomDictionaryRefusal.CUSTOM_DICTIONARY_REFUSAL_FULL ->
                                stringResolver.resolve(StringKey.DICTIONARY_TOO_MANY_ENTRIES)
                            CustomDictionaryRefusal.CUSTOM_DICTIONARY_REFUSAL_NO_USABLE_ROWS,
                            CustomDictionaryRefusal.CUSTOM_DICTIONARY_REFUSAL_NOT_UTF8,
                            -> stringResolver.resolve(StringKey.DICTIONARY_INVALID_C_S_V_FORMAT)
                            else -> stringResolver.resolve(StringKey.COMMON_IMPORT_FAILED)
                        }
                    showResultDialog = true
                }
            }
        }

    // File export launcher
    val exportLauncher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.CreateDocument("text/csv"),
        ) { uri: Uri? ->
            uri ?: return@rememberLauncherForActivityResult
            scope.launch {
                Toast.makeText(context, EXPORTING_HINT, Toast.LENGTH_SHORT).show()
                try {
                    val csv = viewModel.exportCSV()
                    withContext(Dispatchers.IO) {
                        context.contentResolver.openOutputStream(uri)?.use { outputStream ->
                            outputStream.write(csv)
                        }
                    }
                    // Hakfa fork: the first line is the header; one line per Hanji after it.
                    val words = csv.decodeToString().lines().count { it.isNotBlank() } - 1
                    Toast.makeText(context, "匯出完成：$words 個詞", Toast.LENGTH_LONG).show()
                } catch (_: Exception) {
                    resultMessage = stringResolver.resolve(StringKey.COMMON_EXPORT_FAILED)
                    showResultDialog = true
                }
            }
        }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = L10n.dictionaryCustomDictionary,
                        fontWeight = FontWeight.Bold,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = L10n.commonBack)
                    }
                },
                actions = {
                    IconButton(onClick = {
                        editingEntry = null
                        showEditDialog = true
                    }) {
                        Icon(Icons.Default.Add, contentDescription = L10n.dictionaryAddEntry)
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
                    .padding(padding),
        ) {
            LazyColumn(
                modifier =
                    Modifier
                        .weight(1f)
                        .padding(horizontal = 20.dp),
            ) {
                // Enable/Disable toggle
                item {
                    Spacer(Modifier.height(8.dp))
                    SettingsCard {
                        SwitchRow(
                            label = L10n.dictionaryCustomDictEnabled,
                            checked = isCustomDictEnabled,
                            infoText = L10n.dictionaryCustomDictEnabledInfo,
                            onCheckedChange = { viewModel.setCustomDictEnabled(it) },
                        )
                    }
                }

                // Import/Export + Delete
                item {
                    Spacer(Modifier.height(16.dp))
                    Text(
                        text = L10n.dictionaryImportExportTitle,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(start = 16.dp, bottom = 8.dp),
                        style = MaterialTheme.typography.titleMedium,
                    )

                    SettingsCard {
                        // Hakfa fork: the CSV carries one column per lect (Hanji + the reading in each lect),
                        // the same file the Mac exports; the old two-column roman,hanji file still imports (into 四縣).
                        Text(
                            text = CSV_FORMAT_NOTE,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(horizontal = 20.dp).padding(top = 16.dp),
                            style = MaterialTheme.typography.bodyMedium,
                            fontFamily = FontFamily.Monospace,
                        )
                        Text(
                            text = L10n.dictionaryCustomDictDescription,
                            color = MaterialTheme.colorScheme.onSurface,
                            modifier = Modifier.padding(horizontal = 20.dp, vertical = 16.dp),
                            style = MaterialTheme.typography.bodyLarge,
                        )
                        SettingsDivider()
                        ActionRow(
                            label = L10n.dictionaryExportCSV,
                            onClick = {
                                if (!isImporting) {
                                    val dateStr = SimpleDateFormat("yyyy-MM-dd", Locale.US).format(Date())
                                    exportLauncher.launch("taigi_custom_dictionary_$dateStr.csv")
                                }
                            },
                            icon = Icons.Outlined.FileUpload,
                            textColor = MaterialTheme.colorScheme.primary,
                        )
                        SettingsDivider()
                        if (isImporting) {
                            LoadingRow()
                        } else {
                            ActionRow(
                                label = L10n.dictionaryImportCSV,
                                onClick = { importLauncher.launch(arrayOf("text/*")) },
                                icon = Icons.Outlined.FileDownload,
                                textColor = MaterialTheme.colorScheme.primary,
                            )
                        }
                    }
                }

                // Delete all — runs on the tap, no confirmation (USER 2026-10-16)
                item {
                    Spacer(Modifier.height(16.dp))
                    SettingsCard {
                        ActionRow(
                            label = L10n.dictionaryDeleteAll,
                            onClick = { if (!isImporting) viewModel.deleteAll() },
                            textColor = MaterialTheme.colorScheme.error,
                        )
                    }
                }

                // Privacy warning
                item {
                    Spacer(Modifier.height(16.dp))
                    SettingsCard {
                        Text(
                            text = L10n.dictionaryCustomDictPrivacyWarning,
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurface,
                            modifier = Modifier.padding(horizontal = 20.dp, vertical = 16.dp),
                        )
                    }
                }

                // Entry list header
                item {
                    Spacer(Modifier.height(24.dp))
                    Row(
                        modifier = Modifier.padding(start = 16.dp, bottom = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = L10n.dictionaryCustomDictionary,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Spacer(Modifier.width(6.dp))
                        SettingInfoButton(description = L10n.dictionaryFilterHint)
                    }
                }

                if (isLoading) {
                    item { SettingsCard { LoadingRow() } }
                } else if (entries.isEmpty()) {
                    item { EmptyListCard(L10n.dictionaryCustomDictEmpty) }
                } else if ((filterText.isNotEmpty() || lectFilter != null) && filteredEntries.isEmpty()) {
                    item { NoticeCard(L10n.dictionaryNoResults) }
                } else {
                    itemsIndexed(
                        items = filteredEntries,
                        key = { _, word -> word.key },
                    ) { index, word ->
                        Row(
                            modifier =
                                Modifier
                                    .fillMaxWidth()
                                    .background(MaterialTheme.colorScheme.surface)
                                    .heightIn(min = 48.dp)
                                    .padding(start = 20.dp, end = 4.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Column(
                                modifier =
                                    Modifier
                                        .weight(1f)
                                        .clickable {
                                            editingEntry = word
                                            showEditDialog = true
                                        }.padding(vertical = 12.dp),
                            ) {
                                // Hakfa fork: the Hanji, then one line per lect that has a reading.
                                Text(
                                    text = word.hanji.ifBlank { "—" },
                                    color = MaterialTheme.colorScheme.onSurface,
                                    style = MaterialTheme.typography.bodyLarge,
                                )
                                HakfaLects.all.filter { word.readings.containsKey(it.code) }.forEach { lect ->
                                    Text(
                                        text = "${lectTag(lect.code)}　${word.roman(lect.code)}",
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        style = MaterialTheme.typography.bodyMedium,
                                    )
                                }
                            }
                            IconButton(
                                onClick = { viewModel.deleteWord(word) },
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = L10n.commonDelete,
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                        if (index < filteredEntries.lastIndex) {
                            HorizontalDivider(
                                modifier = Modifier.padding(horizontal = 20.dp),
                                color = MaterialTheme.colorScheme.outlineVariant,
                            )
                        }
                    }
                }

                item { Spacer(Modifier.height(40.dp)) }
            }

            // Hakfa fork: show only the words that have a reading in one lect.
            LectFilterRow(selected = lectFilter, onSelected = { lectFilter = it })

            // Filter (anchored at bottom)
            FilterSearchBar(
                value = filterText,
                onValueChange = { filterText = it },
                placeholder = L10n.dictionarySearchPlaceholder,
            )
        }
    }

    // Edit/Add dialog
    if (showEditDialog) {
        EditEntryDialog(
            word = editingEntry,
            onDismiss = { showEditDialog = false },
            onSave = { hanji, romans ->
                viewModel.saveWord(planWordSave(editingEntry, hanji, romans, lectCodes))
                showEditDialog = false
            },
            lookup = viewModel::lookupReading,
        )
    }

    // Result dialog
    if (showResultDialog) {
        ResultDialog(
            message = resultMessage,
            confirmLabel = L10n.commonOk,
            onDismiss = { showResultDialog = false },
        )
    }
}

// Hakfa fork: the short lect name a row wears ("四縣", "海陸", "南四縣"); the part before the romanization.
private fun lectTag(code: Int): String = HakfaLects.nameOf(code).substringBefore(" (")

// Hakfa fork: 「全部」 plus one chip per lect; the chosen one limits the list to words read in that lect.
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun LectFilterRow(
    selected: Int?,
    onSelected: (Int?) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        FilterChip(selected = selected == null, onClick = { onSelected(null) }, label = { Text("全部") })
        HakfaLects.all.forEach { lect ->
            FilterChip(
                selected = selected == lect.code,
                onClick = { onSelected(lect.code) },
                label = { Text(lectTag(lect.code)) },
            )
        }
    }
}
