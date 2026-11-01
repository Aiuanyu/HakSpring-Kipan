package com.siansiansu.taigikeyboard.ime.settings

/**
 * Hakfa fork: how Annotate in Brackets writes the pair of scripts, `X` leading and `Y` annotating — one
 * choice per lead order ([BracketAnnotation]). The engine writes the commit
 * (`protos` `AnnotationTemplate::wrap`); [preview] is what the candidate cell shows meanwhile.
 *
 * Mirrors desktop-core `BracketTemplate` and macOS `BracketTemplate`; [storageValue] is identical on all.
 */
enum class BracketTemplate(
    val storageValue: String,
    /** The settings row: the template with `X`, `Y` standing for the lead and the annotation. */
    val exampleLabel: String,
) {
    ROUND("round", "X (Y)"),
    SQUARE("square", "X [Y]"),
    CURLY("curly", "X {Y}"),
    RUBY("ruby", "<ruby>X<rt>Y</rt></ruby>"),
    ;

    /** What the template writes for a lead [x] annotated by [y]. */
    fun wrap(
        x: String,
        y: String,
    ): String =
        when (this) {
            ROUND -> "$x ($y)"
            SQUARE -> "$x [$y]"
            CURLY -> "$x {$y}"
            RUBY -> "<ruby>$x<rt>$y</rt></ruby>"
        }

    /** The settings row's value: [wrap] with the tags trimmed (`X <rt>Y` for ruby), short enough for two to share a line. */
    fun shortWrap(
        x: String,
        y: String,
    ): String = if (this == RUBY) "$x <rt>$y" else wrap(x, y)

    /**
     * The annotation as a candidate cell previews it: in the brackets the commit writes, or — for ruby, whose
     * tags would fill the cell — `ᴿᵀ ` ahead of it.
     */
    fun preview(annotation: String): String =
        when (this) {
            ROUND -> "($annotation)"
            SQUARE -> "[$annotation]"
            CURLY -> "{$annotation}"
            RUBY -> "ᴿᵀ $annotation"
        }

    companion object {
        /** Coerce a stored raw string into a template; unknown / absent values fall back to [ROUND]. */
        fun fromStorage(raw: String?): BracketTemplate = entries.firstOrNull { it.storageValue == raw } ?: ROUND
    }
}

/** Hakfa fork: Annotate in Brackets is on; the template for each lead order. `null` where it is off. */
data class BracketAnnotation(
    val hanjiFirst: BracketTemplate = BracketTemplate.ROUND,
    val romanFirst: BracketTemplate = BracketTemplate.ROUND,
) {
    /** The template the commit takes for the lead [isHanjiFirst] names. */
    fun templateFor(isHanjiFirst: Boolean): BracketTemplate = if (isHanjiFirst) hanjiFirst else romanFirst
}
