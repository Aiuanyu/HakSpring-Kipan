package com.siansiansu.taigikeyboard.settings

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.siansiansu.taigikeyboard.ime.dictionary.ExternalLookupURLBuilder
import com.siansiansu.taigikeyboard.ime.dictionary.GlossEntry
import com.siansiansu.taigikeyboard.ime.dictionary.GlossRepository
import com.siansiansu.taigikeyboard.ime.settings.PrefHelper
import com.siansiansu.taigikeyboard.ui.setTaigiContent

/**
 * Hakfa fork: Action Mode "客源翠" (ACTION_PROCESS_TEXT).
 * Allows user to highlight text in any app, lookup Hakfa headwords / synonyms,
 * and replace or copy the selected text in-place.
 */
class ProcessTextLookupActivity : ComponentActivity() {

    private val glossRepo by lazy { GlossRepository(this) }
    private val prefs by lazy { PrefHelper(this) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val rawText = intent.getCharSequenceExtra(Intent.EXTRA_PROCESS_TEXT)?.toString()?.trim() ?: ""
        val isReadOnly = intent.getBooleanExtra(Intent.EXTRA_PROCESS_TEXT_READONLY, false)
        val query = rawText.take(64) // Security boundary: max 64 chars

        setTaigiContent(prefs) {
            ProcessTextLookupScreen(
                query = query,
                isReadOnly = isReadOnly,
                onDismiss = { finish() },
                onSelectReplacement = { textToCommit ->
                    if (!isReadOnly) {
                        val resultIntent = Intent().apply {
                            putExtra(Intent.EXTRA_PROCESS_TEXT, textToCommit)
                        }
                        setResult(Activity.RESULT_OK, resultIntent)
                        finish()
                    } else {
                        copyToClipboard(textToCommit)
                        finish()
                    }
                },
                onOpenWeb = { word, lectId ->
                    val url = ExternalLookupURLBuilder.hakspringURL(word, lectId)
                    try {
                        startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                    } catch (e: Exception) {
                        Toast.makeText(this, "無法開啟瀏覽器", Toast.LENGTH_SHORT).show()
                    }
                },
                glossRepo = glossRepo,
                prefs = prefs,
            )
        }
    }

    private fun copyToClipboard(text: String) {
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        clipboard.setPrimaryClip(ClipData.newPlainText("客源翠", text))
        Toast.makeText(this, "已複製：$text", Toast.LENGTH_SHORT).show()
    }
}

private val LECT_OPTIONS = listOf(
    Pair(null, "全部"),
    Pair("xiien", "四縣"),
    Pair("namxiian", "南四縣"),
    Pair("hoiliug", "海陸"),
    Pair("taibu", "大埔"),
    Pair("ngiaupin", "饒平"),
    Pair("zhioon", "詔安"),
)

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun ProcessTextLookupScreen(
    query: String,
    isReadOnly: Boolean,
    onDismiss: () -> Unit,
    onSelectReplacement: (String) -> Unit,
    onOpenWeb: (String, String) -> Unit,
    glossRepo: GlossRepository,
    prefs: PrefHelper,
) {
    var searchQuery by remember(query) { mutableStateOf(query) }
    var selectedLect by remember { mutableStateOf<String?>(null) }
    val results = remember(searchQuery, selectedLect) {
        glossRepo.search(searchQuery, selectedLect)
    }

    // 半透明滿版背景遮罩，點擊背景關閉
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.Black.copy(alpha = 0.5f))
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                onClick = onDismiss,
            ),
        contentAlignment = Alignment.BottomCenter,
    ) {
        // 底部彈出面板
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClick = { /* prevent click-through */ },
                ),
            shape = RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp),
            color = MaterialTheme.colorScheme.surface,
            tonalElevation = 8.dp,
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 12.dp),
            ) {
                // 拖動指示條
                Box(
                    modifier = Modifier
                        .align(Alignment.CenterHorizontally)
                        .size(width = 36.dp, height = 4.dp)
                        .clip(RoundedCornerShape(2.dp))
                        .background(MaterialTheme.colorScheme.outlineVariant),
                )

                Spacer(modifier = Modifier.height(8.dp))

                // 標題欄
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = "客源翠",
                            fontSize = 18.sp,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.primary,
                        )
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(
                            text = "隨身查詞",
                            fontSize = 14.sp,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        if (!isReadOnly) {
                            Spacer(modifier = Modifier.width(6.dp))
                            Surface(
                                shape = RoundedCornerShape(4.dp),
                                color = MaterialTheme.colorScheme.primaryContainer,
                            ) {
                                Text(
                                    text = "點擊替換",
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.Medium,
                                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                                )
                            }
                        }
                    }
                    IconButton(onClick = onDismiss, modifier = Modifier.size(32.dp)) {
                        Icon(
                            imageVector = Icons.Default.Close,
                            contentDescription = "關閉",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }

                // 查詢搜尋框
                OutlinedTextField(
                    value = searchQuery,
                    onValueChange = { searchQuery = it },
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 4.dp),
                    placeholder = { Text("輸入客語字、詞或華語反查...", fontSize = 14.sp) },
                    singleLine = true,
                    leadingIcon = {
                        Icon(
                            imageVector = Icons.Default.Search,
                            contentDescription = "搜尋",
                            tint = MaterialTheme.colorScheme.primary,
                        )
                    },
                    trailingIcon = {
                        if (searchQuery.isNotEmpty()) {
                            IconButton(onClick = { searchQuery = "" }, modifier = Modifier.size(24.dp)) {
                                Icon(
                                    imageVector = Icons.Default.Close,
                                    contentDescription = "清除",
                                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    },
                )

                // 六腔篩選 Chips（照「四南海大平安」排）
                LazyRow(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    items(LECT_OPTIONS) { (id, name) ->
                        val isSelected = selectedLect == id
                        FilterChip(
                            selected = isSelected,
                            onClick = { selectedLect = id },
                            label = { Text(name, fontSize = 12.sp) },
                            colors = FilterChipDefaults.filterChipColors(
                                selectedContainerColor = MaterialTheme.colorScheme.secondaryContainer,
                                selectedLabelColor = MaterialTheme.colorScheme.onSecondaryContainer,
                            ),
                        )
                    }
                }

                Spacer(modifier = Modifier.height(8.dp))

                // 結果清單（最高限制約 55vh，避免遮蓋整螢幕）
                if (results.isEmpty()) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(160.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Icon(
                                imageVector = Icons.Default.Search,
                                contentDescription = null,
                                modifier = Modifier.size(36.dp),
                                tint = MaterialTheme.colorScheme.outline,
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = "客源翠詞庫中未收錄「$query」",
                                fontSize = 13.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                            Spacer(modifier = Modifier.height(8.dp))
                            OutlinedButton(
                                onClick = { onOpenWeb(query, "si") },
                                colors = ButtonDefaults.outlinedButtonColors(
                                    contentColor = MaterialTheme.colorScheme.primary,
                                ),
                            ) {
                                Icon(Icons.Default.Search, contentDescription = null, modifier = Modifier.size(16.dp))
                                Spacer(modifier = Modifier.width(4.dp))
                                Text("前往客源翠網站搜尋", fontSize = 12.sp)
                            }
                        }
                    }
                } else {
                    LazyColumn(
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(max = 420.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        items(results, key = { "${it.id}|${it.word}|${it.lectId}" }) { entry ->
                            GlossResultCard(
                                entry = entry,
                                prefs = prefs,
                                isReadOnly = isReadOnly,
                                onSelectReplacement = onSelectReplacement,
                                onOpenWeb = onOpenWeb,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun GlossResultCard(
    entry: GlossEntry,
    prefs: PrefHelper,
    isReadOnly: Boolean,
    onSelectReplacement: (String) -> Unit,
    onOpenWeb: (String, String) -> Unit,
) {
    // 依目前輸出設定格式化全字 commit
    val defaultReplacement = remember(entry, prefs) {
        val template = prefs.bracketAnnotation?.templateFor(prefs.storedIsHanjiFirst)
        if (prefs.isOutputBothScripts && template != null) {
            if (prefs.storedIsHanjiFirst) {
                template.wrap(entry.word, entry.reading)
            } else {
                template.wrap(entry.reading, entry.word)
            }
        } else {
            entry.word
        }
    }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onSelectReplacement(defaultReplacement) },
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f),
        ),
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            // 頂行：客語詞目、腔調、讀音、外部連結
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.weight(1f)) {
                    Text(
                        text = entry.word,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    // 腔別小 Badge
                    Surface(
                        shape = RoundedCornerShape(4.dp),
                        color = MaterialTheme.colorScheme.primaryContainer,
                    ) {
                        Text(
                            text = entry.lectName + if (entry.dialectTags.isNotEmpty()) " (${entry.dialectTags})" else "",
                            fontSize = 11.sp,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                            modifier = Modifier.padding(horizontal = 4.dp, vertical = 2.dp),
                        )
                    }
                    if (entry.isSynonymHit && entry.matchedKeyword != null) {
                        Spacer(modifier = Modifier.width(4.dp))
                        Surface(
                            shape = RoundedCornerShape(4.dp),
                            color = MaterialTheme.colorScheme.tertiaryContainer,
                        ) {
                            Text(
                                text = "同義",
                                fontSize = 10.sp,
                                color = MaterialTheme.colorScheme.onTertiaryContainer,
                                modifier = Modifier.padding(horizontal = 4.dp, vertical = 2.dp),
                            )
                        }
                    }
                }

                // 客源翠網頁按鈕
                OutlinedButton(
                    onClick = { onOpenWeb(entry.word, entry.lectId) },
                    modifier = Modifier.height(28.dp),
                    contentPadding = ButtonDefaults.TextButtonContentPadding,
                    colors = ButtonDefaults.outlinedButtonColors(
                        contentColor = MaterialTheme.colorScheme.primary,
                    ),
                ) {
                    Text("客源翠 🌐", fontSize = 11.sp)
                }
            }

            // 讀音
            Text(
                text = entry.reading,
                fontSize = 14.sp,
                color = MaterialTheme.colorScheme.secondary,
                fontWeight = FontWeight.Medium,
            )

            // 簡短釋義
            if (entry.gloss.isNotEmpty()) {
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = entry.gloss,
                    fontSize = 13.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            // 底部操作膠囊列
            Spacer(modifier = Modifier.height(8.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.End,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                // 淨字
                OutlinedButton(
                    onClick = { onSelectReplacement(entry.word) },
                    modifier = Modifier.height(28.dp),
                    contentPadding = ButtonDefaults.TextButtonContentPadding,
                ) {
                    Text(if (!isReadOnly) "填字" else "複製字", fontSize = 11.sp)
                }
                Spacer(modifier = Modifier.width(6.dp))
                // 淨音
                OutlinedButton(
                    onClick = { onSelectReplacement(entry.reading) },
                    modifier = Modifier.height(28.dp),
                    contentPadding = ButtonDefaults.TextButtonContentPadding,
                ) {
                    Text(if (!isReadOnly) "填音" else "複製音", fontSize = 11.sp)
                }
            }
        }
    }
}
