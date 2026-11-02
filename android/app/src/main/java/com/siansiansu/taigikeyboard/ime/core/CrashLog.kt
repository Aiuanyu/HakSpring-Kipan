package com.siansiansu.taigikeyboard.ime.core

import android.app.ActivityManager
import android.app.ApplicationExitInfo
import android.content.Context
import android.os.Build
import java.io.File
import java.io.PrintWriter
import java.io.StringWriter
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.TimeZone

/**
 * Hakfa fork: keeps the last uncaught exceptions so a crash seen on a phone (the keyboard and the app share one
 * process, so a crash in either takes both down) can be read afterwards — Settings → Diagnostic → Copy. Nothing
 * leaves the device. The file holds the newest entries only, capped in size.
 */
object CrashLog {
    private const val FILE_NAME = "crash_log.txt"
    private const val MAX_CHARS = 24_000
    private const val ENTRY_SEPARATOR = "\n=====\n"

    /** Records every uncaught exception from now on, then lets the previous handler (the system's) kill the process as usual. */
    fun install(context: Context) {
        val file = File(context.applicationContext.filesDir, FILE_NAME)
        val previous = Thread.getDefaultUncaughtExceptionHandler()
        Thread.setDefaultUncaughtExceptionHandler { thread, error ->
            runCatching { file.writeText(appendEntry(runCatching { file.readText() }.getOrDefault(""), entry(thread.name, error))) }
            previous?.uncaughtException(thread, error)
        }
    }

    /** The recorded crashes, newest last, plus the system's own record of how the process last ended (API 30+). */
    fun read(context: Context): String {
        val recorded = runCatching { File(context.filesDir, FILE_NAME).readText() }.getOrDefault("").trim()
        val exits = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) exitReasons(context) else ""
        return listOf(recorded, exits).filter { it.isNotEmpty() }.joinToString("\n\n")
    }

    internal fun entry(
        threadName: String,
        error: Throwable,
        nowMs: Long = System.currentTimeMillis(),
    ): String {
        val stack = StringWriter().also { error.printStackTrace(PrintWriter(it)) }.toString().trimEnd()
        return "${formatTime(nowMs)} thread=$threadName\n$stack"
    }

    /** [existing] plus [entry], cut from the front (oldest first) to at most [MAX_CHARS]. */
    internal fun appendEntry(
        existing: String,
        entry: String,
    ): String {
        val all = if (existing.isBlank()) entry else existing.trimEnd() + ENTRY_SEPARATOR + entry
        if (all.length <= MAX_CHARS) return all
        val tail = all.takeLast(MAX_CHARS)
        // Start at an entry boundary when one is left, so the oldest kept entry is whole.
        val boundary = tail.indexOf(ENTRY_SEPARATOR)
        return if (boundary >= 0) tail.substring(boundary + ENTRY_SEPARATOR.length) else tail
    }

    private fun formatTime(ms: Long): String = SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US).apply { timeZone = TimeZone.getTimeZone("Asia/Taipei") }.format(Date(ms)) + " (UTC+8)"

    @androidx.annotation.RequiresApi(Build.VERSION_CODES.R)
    private fun exitReasons(context: Context): String {
        val manager = context.getSystemService(Context.ACTIVITY_SERVICE) as? ActivityManager ?: return ""
        val exits = runCatching { manager.getHistoricalProcessExitReasons(context.packageName, 0, 5) }.getOrDefault(emptyList())
        if (exits.isEmpty()) return ""
        return "System exit records (newest first):\n" +
            exits.joinToString("\n") { exitLine(it) }
    }

    // `status` is the signal number for SIGNALED / CRASH_NATIVE (9 = killed from outside, 11 = segfault, 6 = abort),
    // the exit code for EXIT_SELF; `rss` / `pss` show whether the process was big when it died; `importance` says
    // whether it was in front (100) or behind (300+).
    @androidx.annotation.RequiresApi(Build.VERSION_CODES.R)
    private fun exitLine(info: ApplicationExitInfo): String =
        "${formatTime(info.timestamp)} ${reasonName(info.reason)} status=${info.status} importance=${info.importance} " +
            "rss=${info.rss / 1024}MB pss=${info.pss / 1024}MB ${info.description.orEmpty()}".trimEnd()

    private fun reasonName(reason: Int): String =
        when (reason) {
            ApplicationExitInfo.REASON_CRASH -> "CRASH"
            ApplicationExitInfo.REASON_CRASH_NATIVE -> "CRASH_NATIVE"
            ApplicationExitInfo.REASON_ANR -> "ANR"
            ApplicationExitInfo.REASON_LOW_MEMORY -> "LOW_MEMORY"
            ApplicationExitInfo.REASON_SIGNALED -> "SIGNALED"
            ApplicationExitInfo.REASON_USER_REQUESTED -> "USER_REQUESTED"
            ApplicationExitInfo.REASON_EXIT_SELF -> "EXIT_SELF"
            ApplicationExitInfo.REASON_USER_STOPPED -> "USER_STOPPED"
            ApplicationExitInfo.REASON_DEPENDENCY_DIED -> "DEPENDENCY_DIED"
            ApplicationExitInfo.REASON_EXCESSIVE_RESOURCE_USAGE -> "EXCESSIVE_RESOURCE_USAGE"
            ApplicationExitInfo.REASON_PERMISSION_CHANGE -> "PERMISSION_CHANGE"
            ApplicationExitInfo.REASON_INITIALIZATION_FAILURE -> "INITIALIZATION_FAILURE"
            ApplicationExitInfo.REASON_OTHER -> "OTHER"
            ApplicationExitInfo.REASON_FREEZER -> "FREEZER"
            // 15 / 16 need API 33: the app was disabled / updated (an install replaces the running process).
            15 -> "PACKAGE_STATE_CHANGE"
            16 -> "PACKAGE_UPDATED"
            else -> "REASON_$reason"
        }
}
