package com.siansiansu.taigikeyboard.ui.tabs.dictionary

import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ExperimentalMaterial3Api
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
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.siansiansu.taigikeyboard.i18n.generated.L10n
import com.siansiansu.taigikeyboard.ime.settings.PrefHelper
import com.siansiansu.taigikeyboard.ui.components.FilterSearchBar
import com.siansiansu.taigikeyboard.ui.components.SettingsCard
import com.siansiansu.taigikeyboard.ui.components.SettingsDivider
import com.siansiansu.taigikeyboard.ui.theme.AppStyle
import com.siansiansu.taigikeyboard.ui.theme.SectionHeader

private const val MAX_VISIBLE_SEARCH_RESULTS = 5
private val SEARCH_RESULTS_MAX_HEIGHT = 200.dp

// Manage Dictionaries — the dictionary-source toggles (MOE characters with nested kautian
// subcollections, other dictionaries, supplementary data) plus dictionary search pinned at
// the bottom. Desktop twin: the Manage Dictionaries settings pane. Each toggle reads its
// pref when the page opens.
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DictionarySourcesScreen(
    prefs: PrefHelper,
    searchViewModel: DictionarySearchViewModel,
    onNavigateBack: () -> Unit,
) {
    val focusManager = LocalFocusManager.current

    var moeEnabled by remember { mutableStateOf(prefs.moeDictEnabled) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = L10n.desktopDictionarySourcesLink,
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
        Column(modifier = Modifier.fillMaxSize().padding(padding)) {
            Column(
                modifier =
                    Modifier
                        .weight(1f)
                        .verticalScroll(rememberScrollState())
                        .pointerInput(Unit) { detectTapGestures { focusManager.clearFocus() } }
                        .padding(horizontal = 20.dp)
                        .padding(bottom = AppStyle.scrollContentBottomPadding),
            ) {
                Spacer(Modifier.height(8.dp))
                // Hakfa fork (Obs IME data-ui D1): only the sources this keyboard has data for.
                // The upstream Taigi dictionaries (iTaigi, plant names, Taiwanese-Japanese, Tai-Hua,
                // STTI, the accent and appendix sub-options, Khiin / variant / LKK characters, the
                // supplement word list) are not listed; their stored values stay as they were.
                // The per-lect sub-options of the MOE dictionary and the certification list are D2.
                SectionHeader(L10n.dictionaryMoeSectionTitle)

                SettingsCard {
                    DictionaryRowWithDescription(
                        label = L10n.commonMoeDict,
                        checked = moeEnabled,
                        description = L10n.dictionaryMoeDescription,
                        url = "https://hakspring.pages.dev/",
                        onCheckedChange = {
                            moeEnabled = it
                            prefs.moeDictEnabled = it
                        },
                    )
                    SettingsDivider()
                    // hakfa: bit 1 (taigitv) is the certification word lists, locked on;
                    // PrefHelper.isNewwordDictEnabled forces it on for the engine too.
                    DictionaryRowWithDescription(
                        label = L10n.commonNewwordDict,
                        checked = true,
                        description = L10n.dictionaryNewwordDescription,
                        url = "https://hakspring.pages.dev/",
                        enabled = false,
                        onCheckedChange = {},
                    )
                    SettingsDivider()
                    // hakfa: bit 6 (kungge) is the HakSpring character supplement, locked on;
                    // PrefHelper.isKunggeDictEnabled forces it on for the engine too.
                    DictionaryRowWithDescription(
                        label = L10n.commonKunggeDict,
                        checked = true,
                        description = L10n.dictionaryKunggeDescription,
                        url = "https://hakspring.pages.dev/",
                        enabled = false,
                        onCheckedChange = {},
                    )
                }
            }

            DictionarySearchPanel(searchViewModel)
        }
    }
}

// Dictionary search: results pop up above the search bar, at most MAX_VISIBLE_SEARCH_RESULTS rows.
@Composable
private fun DictionarySearchPanel(searchViewModel: DictionarySearchViewModel) {
    val focusManager = LocalFocusManager.current
    val searchText by searchViewModel.searchText.collectAsStateWithLifecycle()
    val searchResults by searchViewModel.results.collectAsStateWithLifecycle()
    val isSearching by searchViewModel.isSearching.collectAsStateWithLifecycle()
    var selectedResultIndex by remember { mutableStateOf<Int?>(null) }
    LaunchedEffect(searchText) { selectedResultIndex = null }

    Column {
        if (searchText.isNotEmpty()) {
            if (searchResults.isEmpty() && !isSearching) {
                Text(
                    text = L10n.dictionaryNoResults,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.bodyLarge,
                    modifier =
                        Modifier
                            .padding(horizontal = 20.dp)
                            .padding(bottom = 8.dp),
                )
            } else if (searchResults.isNotEmpty()) {
                SettingsCard(
                    modifier =
                        Modifier
                            .padding(horizontal = 20.dp)
                            .padding(bottom = 8.dp)
                            .heightIn(max = SEARCH_RESULTS_MAX_HEIGHT),
                ) {
                    val visible = searchResults.take(MAX_VISIBLE_SEARCH_RESULTS)
                    LazyColumn {
                        itemsIndexed(visible) { index, result ->
                            SearchResultRow(
                                result = result,
                                isExpanded = selectedResultIndex == index,
                                onToggle = {
                                    selectedResultIndex = if (selectedResultIndex == index) null else index
                                },
                            )
                            if (index < visible.size - 1) {
                                SettingsDivider()
                            }
                        }
                    }
                }
            }
        }

        FilterSearchBar(
            value = searchText,
            onValueChange = { searchViewModel.updateSearchText(it) },
            placeholder = L10n.dictionarySearchPlaceholder,
            onClear = { focusManager.clearFocus() },
        )
    }
}
