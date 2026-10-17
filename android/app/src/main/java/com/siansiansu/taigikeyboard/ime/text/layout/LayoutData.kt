// Pure data deserialized from layout JSON; LayoutManager copies it into a
// mutable ComputedLayoutData for popup injection.

package com.siansiansu.taigikeyboard.ime.text.layout

import com.siansiansu.taigikeyboard.ime.text.key.KeyData
import com.siansiansu.taigikeyboard.ime.text.keyboard.KeyboardMode
typealias LayoutDataArrangement = List<List<KeyData>>

data class LayoutData(
    val type: LayoutType,
    val name: String,
    val direction: String,
    val arrangement: LayoutDataArrangement = listOf(),
)

typealias ComputedLayoutDataArrangement = MutableList<MutableList<KeyData>>

data class ComputedLayoutData(
    val mode: KeyboardMode,
    val name: String,
    val direction: String,
    val arrangement: ComputedLayoutDataArrangement = mutableListOf(),
)
