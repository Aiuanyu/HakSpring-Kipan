package com.siansiansu.taigikeyboard.ui.tabs.settings

import android.content.Context
import android.os.Build
import com.siansiansu.taigikeyboard.BuildConfig
import com.siansiansu.taigikeyboard.engine.ActiveLect
import com.siansiansu.taigikeyboard.engine.HakfaLects
import com.siansiansu.taigikeyboard.engine.LexiconVersion
import com.siansiansu.taigikeyboard.ime.core.CrashLog
import com.siansiansu.taigikeyboard.ime.settings.PrefHelper

private const val UNKNOWN_VERSION = "unknown"

// Snapshot of local device and app state for user-initiated bug reports.
// No data is transmitted — the user controls when and how to share.
data class DiagnosticInfo(
    val appVersion: String,
    val buildNumber: Int,
    val lexiconVersion: String,
    val activeLectName: String,
    val osVersion: String,
    val deviceModel: String,
    /** Hakfa fork: the last uncaught exceptions and the system's exit records; empty when there are none. */
    val recentCrashes: String = "",
) {
    fun formattedDeviceInfo(): String =
        """
            |客源翠企盤 HakSpring-Kipan
            |App: v$appVersion ($buildNumber)
            |詞庫版本: $lexiconVersion
            |目前腔頭: $activeLectName
            |OS: Android $osVersion (API ${Build.VERSION.SDK_INT})
            |Device: $deviceModel
        """.trimMargin().trim()

    fun formatted(): String =
        formattedDeviceInfo() + if (recentCrashes.isEmpty()) "" else "\n\nRecent crashes:\n$recentCrashes"
}

// Gathers local diagnostic information on demand.
object DiagnosticService {
    fun gather(context: Context): DiagnosticInfo {
        val appVersion =
            try {
                context.packageManager.getPackageInfo(context.packageName, 0).versionName ?: UNKNOWN_VERSION
            } catch (_: Exception) {
                UNKNOWN_VERSION
            }

        val lexicon = LexiconVersion.get(context).displaySummary()
        val lectCode = try {
            PrefHelper(context).lect
        } catch (_: Exception) {
            ActiveLect.current
        }
        val activeLectName = HakfaLects.nameOf(lectCode)

        return DiagnosticInfo(
            appVersion = appVersion,
            buildNumber = BuildConfig.VERSION_CODE,
            lexiconVersion = lexicon,
            activeLectName = activeLectName,
            osVersion = Build.VERSION.RELEASE,
            deviceModel = "${Build.MANUFACTURER} ${Build.MODEL}",
            recentCrashes = CrashLog.read(context),
        )
    }
}
