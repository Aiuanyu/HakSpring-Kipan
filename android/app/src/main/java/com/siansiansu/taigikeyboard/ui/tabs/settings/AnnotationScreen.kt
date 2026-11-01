package com.siansiansu.taigikeyboard.ui.tabs.settings

// Hakfa fork: sub-screen for the template Annotate in Brackets writes in — one choice per lead order. The
// title and the wording are Hakfa-only (no i18n key), like the other fork-only screens.

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.siansiansu.taigikeyboard.i18n.generated.L10n
import com.siansiansu.taigikeyboard.ime.settings.BracketTemplate
import com.siansiansu.taigikeyboard.ui.components.SettingsCard
import com.siansiansu.taigikeyboard.ui.components.SettingsDivider
import com.siansiansu.taigikeyboard.ui.theme.AppStyle
import com.siansiansu.taigikeyboard.ui.theme.SectionHeader

const val ANNOTATION_SETTING_LABEL = "括號標註樣板"

private const val SAMPLE_HANJI = "諸夏字"
private const val SAMPLE_ROMAN = "Lomàsīi"

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AnnotationScreen(
    hanjiFirst: BracketTemplate,
    romanFirst: BracketTemplate,
    onHanjiFirstSelected: (BracketTemplate) -> Unit,
    onRomanFirstSelected: (BracketTemplate) -> Unit,
    onBack: () -> Unit,
) {
    BackHandler(onBack = onBack)

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(ANNOTATION_SETTING_LABEL) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = L10n.commonBack,
                        )
                    }
                },
            )
        },
    ) { innerPadding ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(innerPadding)
                    .padding(horizontal = 20.dp, vertical = 16.dp)
                    .verticalScroll(rememberScrollState()),
        ) {
            SectionHeader("漢字在前")
            TemplateCard(hanjiFirst, SAMPLE_HANJI, SAMPLE_ROMAN, onHanjiFirstSelected)
            SectionHeader("羅馬字在前")
            TemplateCard(romanFirst, SAMPLE_ROMAN, SAMPLE_HANJI, onRomanFirstSelected)
            Text(
                text = "漢字抑係羅馬字在前，照「輸出文字」个設定。候選詞个第二行摎送出去个字共樣；ruby 樣板个候選詞淨顯示「ᴿᵀ 讀音」。",
                modifier = Modifier.padding(top = 16.dp),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

// One card of the four templates; each row shows the template and what it writes for the sample pair.
@Composable
private fun TemplateCard(
    selected: BracketTemplate,
    lead: String,
    annotation: String,
    onSelected: (BracketTemplate) -> Unit,
) {
    SettingsCard {
        BracketTemplate.entries.forEachIndexed { index, template ->
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = 48.dp)
                        .clickable { onSelected(template) }
                        .padding(horizontal = 20.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = template.exampleLabel,
                        color = MaterialTheme.colorScheme.onSurface,
                        style = MaterialTheme.typography.bodyLarge,
                    )
                    Text(
                        text = template.wrap(lead, annotation),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
                if (selected == template) {
                    Icon(
                        imageVector = Icons.Default.Check,
                        contentDescription = null,
                        modifier = Modifier.size(AppStyle.selectionIconSize),
                        tint = MaterialTheme.colorScheme.primary,
                    )
                }
            }
            if (index < BracketTemplate.entries.lastIndex) {
                SettingsDivider()
            }
        }
    }
}
