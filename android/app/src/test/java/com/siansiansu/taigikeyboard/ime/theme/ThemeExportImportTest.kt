package com.siansiansu.taigikeyboard.ime.theme

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.UUID

class ThemeExportImportTest {
    private fun sampleTheme(name: String = "TestTheme"): UserTheme {
        val colors = UserThemeSeed.colors.copy(background = ThemeBackground.Solid(0xFF112233.toInt()))
        val appearance = ThemeAppearance.USER_THEME_SEED.copy(colors = colors, keyShadowIntensity = 0.5f)
        return UserTheme(
            id = UUID.randomUUID().toString(),
            name = name,
            appearance = appearance,
            createdAt = 1000L,
            updatedAt = 2000L,
        )
    }

    @Test
    fun exportAndImport_roundTrip() {
        val original = sampleTheme("MyPalette")
        val exported = ThemeExportImport.exportTheme(original)
        assertTrue(exported.contains("\"type\": \"kipan_theme\""))
        assertTrue(exported.contains("\"schemaVersion\": 1"))

        val importedResult = ThemeExportImport.importTheme(exported)
        assertTrue(importedResult.isSuccess)

        val imported = importedResult.getOrThrow()
        assertEquals("MyPalette", imported.name)
        assertEquals(original.appearance.keyShadowIntensity, imported.appearance.keyShadowIntensity, 0.001f)
        // A new UUID is assigned
        assertNotEquals(original.id, imported.id)
    }

    @Test
    fun importTheme_disambiguatesDuplicateNames() {
        val original = sampleTheme("Ocean")
        val exported = ThemeExportImport.exportTheme(original)

        val existing = listOf(sampleTheme("Ocean"), sampleTheme("Ocean (1)"))
        val imported = ThemeExportImport.importTheme(exported, existing).getOrThrow()

        assertEquals("Ocean (2)", imported.name)
    }

    @Test
    fun importTheme_acceptsBareUserThemeJson() {
        val original = sampleTheme("Bare")
        val bareJson = original.toJson().toString()

        val imported = ThemeExportImport.importTheme(bareJson).getOrThrow()
        assertEquals("Bare", imported.name)
    }

    @Test
    fun importTheme_rejectsInvalidContent() {
        assertTrue(ThemeExportImport.importTheme("").isFailure)
        assertTrue(ThemeExportImport.importTheme("{ invalid json }").isFailure)
        assertTrue(ThemeExportImport.importTheme("{\"other\": 123}").isFailure)
    }

    @Test
    fun suggestFileName_cleansSpecialCharacters() {
        assertEquals("kipan-theme-My_Theme.json", ThemeExportImport.suggestFileName("My Theme"))
        assertEquals("kipan-theme-客語主題_01.json", ThemeExportImport.suggestFileName("客語主題/01"))
        assertEquals("kipan-theme-theme.json", ThemeExportImport.suggestFileName("   "))
    }

    @Test
    fun exportAllAndImport_multiThemeRoundTrip() {
        val theme1 = sampleTheme("Theme1")
        val theme2 = sampleTheme("Theme2")
        val exported = ThemeExportImport.exportAllThemes(listOf(theme1, theme2))
        assertTrue(exported.contains("\"type\": \"kipan_themes_backup\""))

        val importedList = ThemeExportImport.importThemes(exported).getOrThrow()
        assertEquals(2, importedList.size)
        assertEquals("Theme1", importedList[0].name)
        assertEquals("Theme2", importedList[1].name)
    }

    @Test
    fun suggestBackupFileName_formatsTimestamp() {
        assertTrue(ThemeExportImport.suggestBackupFileName().startsWith("kipan-themes-backup-"))
        assertTrue(ThemeExportImport.suggestBackupFileName().endsWith(".json"))
    }
}
