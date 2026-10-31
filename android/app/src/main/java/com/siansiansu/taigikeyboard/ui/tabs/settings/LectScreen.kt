package com.siansiansu.taigikeyboard.ui.tabs.settings

// Hakfa fork: sub-screen for the lect (腔) the keyboard types in. The system keyboard menu offers the
// same choice (one subtype per lect); both write the one stored value.

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.provider.Settings
import android.view.inputmethod.InputMethodManager
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.siansiansu.taigikeyboard.engine.HakfaLects
import com.siansiansu.taigikeyboard.ui.components.LabeledSelectionListScreen

// The title and the row label are Hakfa-only wording (no i18n key), like the other fork-only screens.
const val LECT_SETTING_LABEL = "腔別"

@Composable
fun LectScreen(
    selected: Int,
    onSelected: (Int) -> Unit,
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    LabeledSelectionListScreen(
        title = LECT_SETTING_LABEL,
        options = HakfaLects.all.map { it.code to it.name },
        selected = selected,
        onSelected = onSelected,
        onBack = onBack,
        footer = {
            Text(
                text = "鍵盤語言選單（地球鍵）想愛一腔一項，愛先在系統開啟這六腔。",
                modifier = Modifier.padding(top = 16.dp),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            TextButton(onClick = { openSubtypeSettings(context) }) {
                Text("到系統設定開啟六腔")
            }
        },
    )
}

// Opens the system screen where an input method's subtypes (here: one per lect) are switched on.
// Falls back to the keyboard list when this device has no such screen.
private fun openSubtypeSettings(context: Context) {
    val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as? InputMethodManager
    val imeId = imm?.inputMethodList?.firstOrNull { it.packageName == context.packageName }?.id
    val subtypeScreen =
        Intent(Settings.ACTION_INPUT_METHOD_SUBTYPE_SETTINGS)
            .putExtra(Settings.EXTRA_INPUT_METHOD_ID, imeId)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
    try {
        context.startActivity(subtypeScreen)
    } catch (_: ActivityNotFoundException) {
        runCatching {
            context.startActivity(Intent(Settings.ACTION_INPUT_METHOD_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        }
    }
}
