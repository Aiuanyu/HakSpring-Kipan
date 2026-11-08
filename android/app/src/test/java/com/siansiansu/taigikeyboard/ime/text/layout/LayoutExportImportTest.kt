package com.siansiansu.taigikeyboard.ime.text.layout

import com.siansiansu.taigikeyboard.ime.text.layout.LayoutTestAssets.tables
import com.siansiansu.taigikeyboard.ime.text.layout.LayoutTestAssets.unchanged
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class LayoutExportImportTest {
    @Test
    fun exportAndImport_roundTrip() {
        val original = unchanged("qwerty", LayoutScript.TL, id = "orig-id").copy(name = "客話排列")
        val exported = LayoutExportImport.exportLayout(original)
        assertTrue(exported.contains("\"type\": \"kipan_layout\""))
        assertTrue(exported.contains("\"schemaVersion\": 1"))

        val importedResult = LayoutExportImport.importLayout(exported, tables)
        assertTrue(importedResult.isSuccess)

        val imported = importedResult.getOrThrow()
        assertEquals("客話排列", imported.name)
        assertEquals(original.base, imported.base)
        assertEquals(original.script, imported.script)
        assertEquals(original.keys, imported.keys)
        assertNotEquals(original.id, imported.id)
    }

    @Test
    fun importLayout_disambiguatesDuplicateNames() {
        val original = unchanged("qwerty", LayoutScript.TL).copy(name = "MyLayout")
        val exported = LayoutExportImport.exportLayout(original)

        val existing = listOf(original, original.copy(name = "MyLayout (1)"))
        val imported = LayoutExportImport.importLayout(exported, tables, existing).getOrThrow()

        assertEquals("MyLayout (2)", imported.name)
    }

    @Test
    fun importLayout_acceptsBareCustomLayoutJson() {
        val original = unchanged("phahTaigi", LayoutScript.TL).copy(name = "BareLayout")
        val bareJson = original.toJson().toString()

        val imported = LayoutExportImport.importLayout(bareJson, tables).getOrThrow()
        assertEquals("BareLayout", imported.name)
    }

    @Test
    fun importLayout_rejectsInvalidContent() {
        assertTrue(LayoutExportImport.importLayout("", tables).isFailure)
        assertTrue(LayoutExportImport.importLayout("{ corrupt json }", tables).isFailure)
        assertTrue(LayoutExportImport.importLayout("{\"base\": \"unknownBase\", \"script\": \"tl\"}", tables).isFailure)
    }

    @Test
    fun suggestFileName_cleansSpecialCharacters() {
        assertEquals("kipan-layout-Hakka_Standard.json", LayoutExportImport.suggestFileName("Hakka Standard"))
        assertEquals("kipan-layout-客語鍵盤_V1.json", LayoutExportImport.suggestFileName("客語鍵盤:V1"))
        assertEquals("kipan-layout-layout.json", LayoutExportImport.suggestFileName("   "))
    }

    @Test
    fun exportAllAndImport_multiLayoutRoundTrip() {
        val layout1 = unchanged("qwerty", LayoutScript.TL).copy(name = "Layout1")
        val layout2 = unchanged("phahTaigi", LayoutScript.TL).copy(name = "Layout2")
        val exported = LayoutExportImport.exportAllLayouts(listOf(layout1, layout2))
        assertTrue(exported.contains("\"type\": \"kipan_layouts_backup\""))

        val importedList = LayoutExportImport.importLayouts(exported, tables).getOrThrow()
        assertEquals(2, importedList.size)
        assertEquals("Layout1", importedList[0].name)
        assertEquals("Layout2", importedList[1].name)
    }

    @Test
    fun suggestBackupFileName_formatsTimestamp() {
        assertTrue(LayoutExportImport.suggestBackupFileName().startsWith("kipan-layouts-backup-"))
        assertTrue(LayoutExportImport.suggestBackupFileName().endsWith(".json"))
    }
}
