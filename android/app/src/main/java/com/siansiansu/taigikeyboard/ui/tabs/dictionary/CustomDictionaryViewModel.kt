package com.siansiansu.taigikeyboard.ui.tabs.dictionary

import android.app.Application
import android.net.Uri
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.siansiansu.taigikeyboard.engine.RustEngineBridge
import com.siansiansu.taigikeyboard.engine.hanjiReadings
import com.siansiansu.taigikeyboard.engine.proto.CustomDictionaryRefusal
import com.siansiansu.taigikeyboard.ime.core.CompositionRoot
import com.siansiansu.taigikeyboard.ime.dictionary.CustomDictionaryImportResult
import com.siansiansu.taigikeyboard.ime.dictionary.CustomDictionaryWord
import com.siansiansu.taigikeyboard.ime.dictionary.CustomWord
import com.siansiansu.taigikeyboard.ime.dictionary.UserDataClient
import com.siansiansu.taigikeyboard.ime.dictionary.UserDataException
import com.siansiansu.taigikeyboard.ime.dictionary.WordSavePlan
import com.siansiansu.taigikeyboard.ime.settings.PrefHelper
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout

// ViewModel for CustomDictionaryScreen — entries, CRUD, CSV import/export, enable toggle.
// The words are the engine's (roadmap P8b); the client runs every request on Dispatchers.IO.
class CustomDictionaryViewModel(
    application: Application,
) : AndroidViewModel(application) {
    private val prefs = PrefHelper(application)
    private val root = CompositionRoot.shared(application)
    private val userData: UserDataClient = root.userData

    private val _entries = MutableStateFlow<List<CustomDictionaryWord>>(emptyList())
    val entries: StateFlow<List<CustomDictionaryWord>> = _entries.asStateFlow()

    private val _isLoading = MutableStateFlow(true)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    private val _isImporting = MutableStateFlow(false)
    val isImporting: StateFlow<Boolean> = _isImporting.asStateFlow()

    private val _isCustomDictEnabled = MutableStateFlow(prefs.customDictEnabled)
    val isCustomDictEnabled: StateFlow<Boolean> = _isCustomDictEnabled.asStateFlow()

    fun setCustomDictEnabled(enabled: Boolean) {
        _isCustomDictEnabled.value = enabled
        prefs.customDictEnabled = enabled
    }

    fun load() {
        viewModelScope.launch {
            _entries.value = quietly("load", emptyList()) { userData.listAll() }
            _isLoading.value = false
        }
    }

    // A refused save (a full dictionary, an unsearchable romanization) is
    // logged, not shown — the screen has never surfaced one.
    fun save(entry: CustomDictionaryWord) {
        viewModelScope.launch {
            quietly("save", Unit) { userData.save(entry) }
            load()
        }
    }

    /** Hakfa fork: writes a word's per-lect readings (see [planWordSave]) and reloads once. */
    fun saveWord(plan: WordSavePlan) {
        viewModelScope.launch {
            quietly("saveWord", Unit) {
                plan.saves.forEach { userData.save(it) }
                plan.deleteIds.forEach { userData.delete(it) }
            }
            load()
        }
    }

    /** Hakfa fork: removes every row of the word. */
    fun deleteWord(word: CustomWord) {
        viewModelScope.launch {
            quietly("deleteWord", Unit) { word.rows.forEach { userData.delete(it.id) } }
            load()
        }
    }

    /**
     * Hakfa fork: what the dictionary gives [hanji] in [lect]: one reading is filled in; none or several
     * are said, not guessed at (a guess would be wrong half the time; the person picks).
     */
    suspend fun lookupReading(
        hanji: String,
        lect: Int,
    ): ReadingLookup =
        try {
            withTimeout(LOOKUP_TIMEOUT_MS) {
                withContext(Dispatchers.IO) {
                    val readings = RustEngineBridge.hanjiReadings(hanji, lect)
                    when {
                        readings == null -> ReadingLookup.Failed
                        readings.isEmpty() -> ReadingLookup.None
                        readings.size == 1 -> ReadingLookup.Found(readings.single())
                        else -> ReadingLookup.Several(readings.size)
                    }
                }
            }
        } catch (_: TimeoutCancellationException) {
            ReadingLookup.Failed
        }

    fun delete(id: String) {
        viewModelScope.launch {
            quietly("delete", Unit) { userData.delete(id) }
            load()
        }
    }

    fun deleteAll() {
        viewModelScope.launch {
            quietly("deleteAll", Unit) { userData.deleteAll() }
            load()
        }
    }

    suspend fun exportCSV(): ByteArray = userData.exportCsv()

    /**
     * Imports the file at [uri]. A file over the size limit is refused before
     * it is read; the engine refuses the rest — throwing
     * [UserDataException.Refused], whose refusal the screen words.
     */
    suspend fun importFile(uri: Uri): CustomDictionaryImportResult {
        _isImporting.value = true
        try {
            val csv =
                withContext(Dispatchers.IO) {
                    val resolver = getApplication<Application>().contentResolver
                    val size = resolver.openFileDescriptor(uri, "r")?.use { it.statSize } ?: 0L
                    if (size > UserDataClient.MAX_IMPORT_FILE_BYTES) {
                        throw UserDataException.Refused(
                            CustomDictionaryRefusal.CUSTOM_DICTIONARY_REFUSAL_FILE_TOO_LARGE,
                            "file is larger than 5 MB",
                        )
                    }
                    resolver.openInputStream(uri)?.use { it.readBytes() }
                        ?: throw IllegalStateException("Cannot read file")
                }
            return userData.importCsv(csv)
        } finally {
            _isImporting.value = false
            load()
        }
    }

    /** Runs [request], logging a failure and answering [fallback] rather than failing the screen. */
    private suspend fun <T> quietly(
        op: String,
        fallback: T,
        request: suspend () -> T,
    ): T =
        try {
            request()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            root.logger.w(TAG, "[CUSTOM-DICT] $op failed: ${e.message}", e)
            fallback
        }

    private companion object {
        const val TAG = "CustomDictionaryViewModel"
        const val LOOKUP_TIMEOUT_MS = 8_000L
    }
}

/** What the 「各腔自動補音」 lookup found for one lect. */
sealed interface ReadingLookup {
    data class Found(
        val roman: String,
    ) : ReadingLookup

    data class Several(
        val count: Int,
    ) : ReadingLookup

    data object None : ReadingLookup

    /** The engine did not answer, or took too long. */
    data object Failed : ReadingLookup
}
