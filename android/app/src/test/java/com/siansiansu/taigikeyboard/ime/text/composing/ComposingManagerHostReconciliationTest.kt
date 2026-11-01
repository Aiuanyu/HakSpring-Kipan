// Pins host-callback reconciliation: a "no composing region" report is honoured only when the host
// really no longer holds the preedit the IME last wrote (USER report 2026-09-12, first key lost).

package com.siansiansu.taigikeyboard.ime.text.composing

import android.view.inputmethod.ExtractedText
import android.view.inputmethod.InputConnection
import com.siansiansu.taigikeyboard.engine.RustEngineBridge.ComposingTransition
import com.siansiansu.taigikeyboard.engine.RustEngineBridge.ComposingTransition.Effect
import com.siansiansu.taigikeyboard.engine.proto.CaretDirection
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.lang.reflect.Proxy

class ComposingManagerHostReconciliationTest {
    /**
     * `InputConnection` stand-in over a document snapshot: `getExtractedText` returns [text] with a
     * cursor at [cursor] (`null` text = dead connection; [selectionStart] for a non-collapsed
     * selection; [startOffset] for a partial extraction). Records writes + composing regions, and
     * throws on a read when [readsForbidden] so an idle reconcile is proven not to round-trip.
     */
    private class FakeHost(
        private val text: String?,
        private val cursor: Int = text?.length ?: 0,
        private val selectionStart: Int = cursor,
        private val startOffset: Int = 0,
        private val readsForbidden: Boolean = false,
    ) {
        val writes = mutableListOf<String>()
        val regions = mutableListOf<Pair<Int, Int>>()
        val selections = mutableListOf<Int>()

        val ic: InputConnection =
            Proxy.newProxyInstance(
                InputConnection::class.java.classLoader,
                arrayOf(InputConnection::class.java),
            ) { _, method, args ->
                when (method.name) {
                    "getExtractedText" -> {
                        check(!readsForbidden) { "host read while idle" }
                        text?.let {
                            ExtractedText().apply {
                                this.text = it
                                this.startOffset = this@FakeHost.startOffset
                                this.selectionStart = this@FakeHost.selectionStart
                                this.selectionEnd = cursor
                            }
                        }
                    }
                    "setComposingRegion" -> {
                        regions += (args!![0] as Int) to (args[1] as Int)
                        true
                    }
                    "setSelection" -> {
                        selections += args!![0] as Int
                        true
                    }
                    "setComposingText", "finishComposingText", "commitText", "deleteSurroundingText" -> {
                        writes += method.name
                        true
                    }
                    else -> null
                }
            } as InputConnection
    }

    private fun preedit(display: String) =
        ComposingTransition(
            rawInput = display,
            displayText = display,
            effects = listOf(Effect.UpdatePreedit(display)),
            isComposing = true,
        )

    /** Puts a fresh manager into composing state with [display] written to a matching host. */
    private fun composing(
        display: String,
        delegate: ComposingDelegate = DefaultComposingDelegate,
    ) = composingManagerForTest(delegate).also { it.applyTransition(preedit(display), FakeHost(display).ic) }

    @Test
    fun reconcile_whenIdle_keepsWithoutReadingHost() {
        val manager = composingManagerForTest()
        manager.reconcileWithHost(FakeHost("x", readsForbidden = true).ic)
        assertFalse(manager.isComposing())
    }

    @Test
    fun reconcile_duringSelfCommit_skipsWithoutReadingHost() {
        val manager = composing("a")
        manager.selfCommitInProgress = true
        manager.reconcileWithHost(FakeHost("z", readsForbidden = true).ic)
        assertTrue(manager.isComposing())
    }

    @Test
    fun reconcile_hostStillHoldsPreedit_keepsStateAndGeneration_andReassertsSpan() {
        // trace: doc "abcd", first key wrote "tai" → "abcdtai|"; a stale focus-move report (-1/-1,
        // its own selEnd=4) arrives; the snapshot (ordered after the write) shows "tai" before
        // cursor 7 → keep, no generation bump, span re-asserted at [4,7) — never [1,4) from the
        // report's stale coordinate.
        val manager = composing("tai")
        val tokenBefore = manager.stateToken()
        val host = FakeHost("abcdtai")

        manager.reconcileWithHost(host.ic)

        assertTrue(manager.isComposing())
        assertEquals("tai", manager.getRawInput())
        assertEquals(tokenBefore, manager.stateToken())
        assertEquals(listOf(4 to 7), host.regions)
        assertTrue(host.writes.isEmpty())
    }

    @Test
    fun reconcile_partialExtraction_usesStartOffset() {
        // trace: host extracted only "xtai" starting at document offset 100, cursor at local 4.
        val manager = composing("tai")
        val host = FakeHost("xtai", startOffset = 100)
        manager.reconcileWithHost(host.ic)
        assertEquals(listOf(101 to 104), host.regions)
    }

    @Test
    fun reconcile_unicodePreedit_keeps() {
        // trace: "o͘" = 'o' + U+0358 (two UTF-16 units); region length follows the display text.
        val manager = composing("o͘")
        val host = FakeHost("o͘")
        manager.reconcileWithHost(host.ic)
        assertTrue(manager.isComposing())
        assertEquals(listOf(0 to 2), host.regions)
    }

    @Test
    fun reconcile_hostLostPreedit_clearsOnceWithoutWritingHost() {
        // trace: real tap-away — cursor moved, the text before it is no longer the preedit.
        val manager = composing("a")
        val generationBefore = manager.stateToken().generation
        val host = FakeHost("az", cursor = 2)

        manager.reconcileWithHost(host.ic)

        assertFalse(manager.isComposing())
        assertNull(manager.getRawInput())
        assertEquals(generationBefore + 1, manager.stateToken().generation)
        assertTrue("INVARIANT_composing_external_region_clear_discards_state: no IC write", host.writes.isEmpty())
        assertTrue(host.regions.isEmpty())
        // A second report for the same clear finds an idle manager → no read, no further bump.
        manager.reconcileWithHost(FakeHost("az", readsForbidden = true).ic)
        assertEquals(generationBefore + 1, manager.stateToken().generation)
    }

    @Test
    fun reconcile_cursorMovedInsidePreedit_clears() {
        // trace: "tai" written, user tapped between "t" and "a" → text before cursor is "t".
        val manager = composing("tai")
        manager.reconcileWithHost(FakeHost("tai", cursor = 1).ic)
        assertFalse(manager.isComposing())
    }

    @Test
    fun reconcile_nonCollapsedSelection_clears() {
        val manager = composing("tai")
        val host = FakeHost("tai", cursor = 3, selectionStart = 0)
        manager.reconcileWithHost(host.ic)
        assertFalse(manager.isComposing())
        assertTrue(host.regions.isEmpty())
    }

    @Test
    fun reconcile_nullRead_clears() {
        val manager = composing("a")
        manager.reconcileWithHost(FakeHost(null).ic)
        assertFalse(manager.isComposing())
    }

    @Test
    fun reconcile_nullConnection_clears() {
        val manager = composing("a")
        manager.reconcileWithHost(null)
        assertFalse(manager.isComposing())
    }

    @Test
    fun onHostSelectionUpdate_onlyMinusOneMinusOneReconciles() {
        val manager = composing("a")
        // Acknowledged composing range → no read at all.
        manager.onHostSelectionUpdate(candidatesStart = 0, candidatesEnd = 1, ic = FakeHost("z", readsForbidden = true).ic)
        assertTrue(manager.isComposing())
        manager.onHostSelectionUpdate(candidatesStart = -1, candidatesEnd = -1, ic = FakeHost("z").ic)
        assertFalse(manager.isComposing())
    }

    @Test
    fun reconcile_commitAThenStartB_lateCallbackForA_keepsB() {
        // trace: commit "A" → start "b" → A's late -1/-1 report. isComposing is true again
        // for B, so the snapshot must be compared against B's preedit.
        val manager = composing("a")
        manager.applyTransition(
            ComposingTransition(
                rawInput = "",
                displayText = "",
                effects = listOf(Effect.CommitTextReplacingPreedit("A")),
                isComposing = false,
            ),
            FakeHost("a").ic,
        )
        assertFalse(manager.isComposing())
        manager.applyTransition(preedit("b"), FakeHost("Ab").ic)

        manager.reconcileWithHost(FakeHost("Ab").ic)

        assertTrue(manager.isComposing())
        assertEquals("b", manager.getRawInput())
    }

    @Test
    fun reconcile_reentrantFromInsideWrite_seesNewPreedit() {
        // trace: a host that fires onUpdateSelection synchronously from inside setComposingText;
        // the mirror is assigned before the effects run, so the re-entrant reconcile keeps.
        var reentrantKept: Boolean? = null
        lateinit var manager: ComposingManager
        val delegate =
            ComposingDelegate { effect, ic ->
                if (effect is Effect.UpdatePreedit) {
                    manager.reconcileWithHost(FakeHost(effect.display).ic)
                    reentrantKept = manager.isComposing()
                }
                DefaultComposingDelegate.execute(effect, ic)
            }
        manager = composingManagerForTest(delegate)

        manager.applyTransition(preedit("ta"), FakeHost("ta").ic)

        assertEquals(true, reentrantKept)
        assertTrue(manager.isComposing())
    }

    @Test
    fun inputViewFinished_dropsMirror_soHostReportCannotReMarkFinishedText() {
        // trace: `tsit` composing → BACK (framework already ran finishComposingText, host holds
        // plain "Tsit") → late `-1/-1` reconcile must not re-mark it; the re-show region clear
        // then has nothing to delete (USER device report 2026-10-12).
        val manager = composing("Tsit")
        manager.onInputViewFinished()
        assertFalse(manager.isComposing())
        val host = FakeHost("Tsit", readsForbidden = true)
        manager.reconcileWithHost(host.ic)
        assertTrue(host.regions.isEmpty())
        assertTrue(host.writes.isEmpty())
    }

    @Test
    fun inputViewFinished_whenIdle_writesNothing() {
        val manager = composingManagerForTest()
        val host = FakeHost("Tsit ", readsForbidden = true)
        manager.onInputViewFinished()
        manager.reconcileWithHost(host.ic)
        assertFalse(manager.isComposing())
        assertTrue(host.writes.isEmpty())
    }

    @Test
    fun bumpGeneration_zeroesMirror_soReconcileCannotKeepDroppedComposition() {
        // trace: input-mode switch with no InputConnection bumps the generation (engine will
        // drop its state) — the mirror must not survive to be "kept" by a later same-text reconcile.
        val manager = composing("a")
        manager.bumpGeneration()
        assertFalse(manager.isComposing())
        manager.reconcileWithHost(FakeHost("a", readsForbidden = true).ic)
        assertFalse(manager.isComposing())
    }

    // region Hakfa fork: the host cursor moved into the composition (tap / space swipe)

    /** A fake engine caret over [positions] (display offsets the caret can sit at); records every move asked of it. */
    private class FakeCaretEngine(
        private val display: String,
        private val positions: List<Int>,
        start: Int = positions.last(),
    ) {
        var index = positions.indexOf(start)
        val moves = mutableListOf<CaretDirection>()

        fun move(direction: CaretDirection): ComposingTransition {
            moves += direction
            index =
                when (direction) {
                    CaretDirection.CARET_DIRECTION_START -> 0
                    CaretDirection.CARET_DIRECTION_END -> positions.lastIndex
                    CaretDirection.CARET_DIRECTION_LEFT -> maxOf(0, index - 1)
                    CaretDirection.CARET_DIRECTION_RIGHT -> minOf(positions.lastIndex, index + 1)
                    else -> index
                }
            return ComposingTransition(display, display, emptyList(), true, caretUtf16 = positions[index])
        }
    }

    private class RecordingDelegate : ComposingDelegate {
        val effects = mutableListOf<Effect>()

        override fun execute(
            effect: Effect,
            ic: InputConnection,
        ) {
            effects += effect
        }
    }

    @Test
    fun hostCursorInsideTheComposition_movesTheEngineCaretThere_andWritesItBack() {
        // trace: "ab" + preedit "tâi" at [2,5); the user taps between "t" and "â": the host cursor is 3 →
        // display offset 1. Nearer edge is START (0), one RIGHT step lands on 1.
        val delegate = RecordingDelegate()
        val manager = composing("tâi", delegate)
        val engine = FakeCaretEngine("tâi", listOf(0, 1, 2, 3))
        manager.caretMove = engine::move
        delegate.effects.clear()

        // The caller re-lists the candidates (cut at the caret) exactly when this answers true.
        val moved = manager.onHostSelectionUpdate(2, 5, FakeHost("abtâi", cursor = 3).ic, selectionStart = 3, selectionEnd = 3)

        assertTrue(moved)
        assertEquals(listOf(CaretDirection.CARET_DIRECTION_START, CaretDirection.CARET_DIRECTION_RIGHT), engine.moves)
        assertEquals(listOf<Effect>(Effect.UpdatePreedit("tâi", 1)), delegate.effects)
        // The echo of our own cursor write asks for nothing more.
        assertFalse(manager.onHostSelectionUpdate(2, 5, FakeHost("abtâi", cursor = 3).ic, selectionStart = 3, selectionEnd = 3))
        assertEquals(2, engine.moves.size)
    }

    @Test
    fun hostCursorAtTheEndOfTheComposition_asksNothing() {
        val manager = composing("tâi")
        val engine = FakeCaretEngine("tâi", listOf(0, 1, 2, 3))
        manager.caretMove = engine::move

        manager.onHostSelectionUpdate(2, 5, FakeHost("abtâi", readsForbidden = true).ic, selectionStart = 5, selectionEnd = 5)

        assertTrue(engine.moves.isEmpty())
    }

    @Test
    fun aStaleCursorReport_isLeftAlone() {
        // trace: the report says 3, but the host snapshot (ordered after our write) has the cursor at 5.
        val manager = composing("tâi")
        val engine = FakeCaretEngine("tâi", listOf(0, 1, 2, 3))
        manager.caretMove = engine::move

        manager.onHostSelectionUpdate(2, 5, FakeHost("abtâi", cursor = 5).ic, selectionStart = 3, selectionEnd = 3)

        assertTrue(engine.moves.isEmpty())
    }

    @Test
    fun aCursorOutsideTheRegion_orARegionThatIsNotThePreedit_isLeftAlone() {
        val manager = composing("tâi")
        val engine = FakeCaretEngine("tâi", listOf(0, 1, 2, 3))
        manager.caretMove = engine::move

        manager.onHostSelectionUpdate(2, 5, FakeHost("abtâi", cursor = 1).ic, selectionStart = 1, selectionEnd = 1)
        manager.onHostSelectionUpdate(2, 4, FakeHost("abtâi", cursor = 3).ic, selectionStart = 3, selectionEnd = 3)
        manager.onHostSelectionUpdate(2, 5, FakeHost("abtâi", cursor = 3, selectionStart = 2).ic, selectionStart = 3, selectionEnd = 4)

        assertTrue(engine.moves.isEmpty())
    }

    @Test
    fun whenTheWalkOvershoots_theNearerSideWins() {
        // trace: the caret can only sit at 0, 2 or 3 (a tone mark folds two characters into one unit); target 2
        // is reached exactly from the end by one LEFT; target 1 lands between 0 and 2 — a tie keeps 0.
        val delegate = RecordingDelegate()
        val manager = composing("tâi", delegate)
        val engine = FakeCaretEngine("tâi", listOf(0, 2, 3))
        manager.caretMove = engine::move
        delegate.effects.clear()

        manager.moveCaretTo(1, FakeHost("tâi").ic)

        assertEquals(listOf<Effect>(Effect.UpdatePreedit("tâi", 0)), delegate.effects)
    }

    @Test
    fun theStateTokenChangesWithTheCaret_soAFetchFromAnotherCaretIsStale() {
        val manager = composing("tâi")
        val atEnd = manager.stateToken()
        manager.applyTransition(
            ComposingTransition("tai", "tâi", listOf(Effect.UpdatePreedit("tâi", 1)), true, caretUtf16 = 1),
            FakeHost("abtâi").ic,
        )
        assertTrue(atEnd != manager.stateToken())
    }

    @Test
    fun typingAfterTheCaretMoved_keepsTheEngineCaretFromTheTransition() {
        val manager = composing("tâi")
        val host = FakeHost("abtâi")
        manager.applyTransition(
            ComposingTransition("tai", "tâi", listOf(Effect.UpdatePreedit("tâi", 1)), true, caretUtf16 = 1),
            host.ic,
        )
        // trace: the default delegate wrote the preedit, then put the host cursor at region start 2 + caret 1.
        assertEquals(listOf(3), host.selections)
        val engine = FakeCaretEngine("tâi", listOf(0, 1, 2, 3), start = 1)
        manager.caretMove = engine::move

        // The host cursor is where the engine says it is: nothing to follow.
        manager.onHostSelectionUpdate(2, 5, FakeHost("abtâi", readsForbidden = true).ic, selectionStart = 3, selectionEnd = 3)

        assertTrue(engine.moves.isEmpty())
    }

    // endregion
}
