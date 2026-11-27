//! Which dictionaries and Hakfa lects the engine draws from, port of
//! `DictionaryTogglesView.swift`.
//!
//! Six lects ("四南海大平安": 四縣, 南四縣, 海陸, 大埔, 饒平, 詔安) under the
//! MOE Hakfa dictionary, with sub-dialect switches under 南四縣 (美濃音) and
//! 饒平 (卓蘭音, 桃園音), locked certification / HakSpring / Kungge rows, and
//! a separate output option for tone 24 → 33 (四縣 / 南四縣).

use super::reset_row;
use crate::winui::cards;
use crate::winui::window::{Message, ResetScope, SettingsWindow};
use taigi_desktop_core::settings::{keys, Lect};
use taigi_desktop_core::strings::{StringKey, StringResolver};
use windows_reactor::*;

pub fn view(
    window: &SettingsWindow,
    strings: &StringResolver,
    context: &mut ViewContext<SettingsWindow>,
) -> View {
    let document = window.document();
    let is_kautian_enabled = document.bool(&keys::IS_KAUTIAN_ENABLED);
    let enabled_lect_count = Lect::ALL
        .iter()
        .filter(|lect| document.bool(&keys::lect_enabled(**lect)))
        .count();
    let is_xiien_enabled = document.bool(&keys::lect_enabled(Lect::Xiien));
    let is_namxiian_enabled = document.bool(&keys::lect_enabled(Lect::Namxiian));
    let is_ngiaupin_enabled = document.bool(&keys::lect_enabled(Lect::Ngiaupin));

    let mut moe_children: Vec<(&'static str, View)> = Vec::new();
    for &lect in Lect::ALL {
        let key = keys::lect_enabled(lect);
        let is_on = document.bool(&key);
        let can_toggle = is_kautian_enabled && !(is_on && enabled_lect_count <= 1);
        moe_children.push((
            key.name,
            cards::sub_row(
                lect.display_name(),
                is_kautian_enabled,
                cards::switch(
                    is_on,
                    can_toggle,
                    context.callback(move |value| Message::SetSwitch(key, value)),
                ),
            ),
        ));
        if lect == Lect::Namxiian {
            let sub_enabled = is_kautian_enabled && is_namxiian_enabled;
            moe_children.push((
                keys::IS_MEINONG_ENABLED.name,
                cards::sub_row(
                    "美濃音 (Minung)",
                    sub_enabled,
                    cards::switch(
                        document.bool(&keys::IS_MEINONG_ENABLED),
                        sub_enabled,
                        context
                            .callback(|value| Message::SetSwitch(keys::IS_MEINONG_ENABLED, value)),
                    ),
                ),
            ));
        }
        if lect == Lect::Ngiaupin {
            let sub_enabled = is_kautian_enabled && is_ngiaupin_enabled;
            moe_children.push((
                keys::IS_ZHUOLAN_ENABLED.name,
                cards::sub_row(
                    "卓蘭音",
                    sub_enabled,
                    cards::switch(
                        document.bool(&keys::IS_ZHUOLAN_ENABLED),
                        sub_enabled,
                        context
                            .callback(|value| Message::SetSwitch(keys::IS_ZHUOLAN_ENABLED, value)),
                    ),
                ),
            ));
            moe_children.push((
                keys::IS_TAOYUAN_ENABLED.name,
                cards::sub_row(
                    "桃園音",
                    sub_enabled,
                    cards::switch(
                        document.bool(&keys::IS_TAOYUAN_ENABLED),
                        sub_enabled,
                        context
                            .callback(|value| Message::SetSwitch(keys::IS_TAOYUAN_ENABLED, value)),
                    ),
                ),
            ));
        }
    }

    // Certification list follows MOE lects (no standalone 南四縣; 四縣 follows 四縣 || 南四縣).
    let cert_children: [(&'static str, View); 5] = [
        (
            "cert.xiien",
            cards::sub_row(
                Lect::Xiien.display_name(),
                false,
                cards::switch(
                    is_xiien_enabled || is_namxiian_enabled,
                    false,
                    context.callback(|_| Message::SetChoice(None)),
                ),
            ),
        ),
        (
            "cert.hoiliug",
            cards::sub_row(
                Lect::Hoiliug.display_name(),
                false,
                cards::switch(
                    document.bool(&keys::lect_enabled(Lect::Hoiliug)),
                    false,
                    context.callback(|_| Message::SetChoice(None)),
                ),
            ),
        ),
        (
            "cert.taibu",
            cards::sub_row(
                Lect::Taibu.display_name(),
                false,
                cards::switch(
                    document.bool(&keys::lect_enabled(Lect::Taibu)),
                    false,
                    context.callback(|_| Message::SetChoice(None)),
                ),
            ),
        ),
        (
            "cert.ngiaupin",
            cards::sub_row(
                Lect::Ngiaupin.display_name(),
                false,
                cards::switch(
                    is_ngiaupin_enabled,
                    false,
                    context.callback(|_| Message::SetChoice(None)),
                ),
            ),
        ),
        (
            "cert.zhioon",
            cards::sub_row(
                Lect::Zhioon.display_name(),
                false,
                cards::switch(
                    document.bool(&keys::lect_enabled(Lect::Zhioon)),
                    false,
                    context.callback(|_| Message::SetChoice(None)),
                ),
            ),
        ),
    ];

    View::fragment((
        cards::section_title(strings.resolve(StringKey::DictionaryMoeSectionTitle)),
        cards::group(
            cards::line(
                strings.resolve(StringKey::CommonMoeDict),
                cards::switch(
                    is_kautian_enabled,
                    true,
                    context.callback(|is_on| Message::SetSwitch(keys::IS_KAUTIAN_ENABLED, is_on)),
                ),
            ),
            moe_children,
        ),
        cards::group(
            cards::line(
                strings.resolve(StringKey::CommonNewwordDict),
                cards::switch(true, false, context.callback(|_| Message::SetChoice(None))),
            ),
            cert_children,
        ),
        cards::described_row(
            strings.resolve(StringKey::CommonHakspringDict),
            strings.resolve(StringKey::DictionaryHakspringDescription),
            cards::switch(true, false, context.callback(|_| Message::SetChoice(None))),
        ),
        cards::switch_row(
            strings.resolve(StringKey::CommonKunggeDict),
            true,
            false,
            context.callback(|_| Message::SetChoice(None)),
        ),
        cards::section_title("輸出選項"),
        cards::described_row(
            "24 調寫做 33（á→ā）",
            "美濃人个 24 調讀 33。淨改羅馬字个寫法，漢字、詞庫、學習資料都毋變；四縣、南四縣適用。",
            cards::switch(
                document.bool(&keys::IS_TONE33_ENABLED),
                is_xiien_enabled || is_namxiian_enabled,
                context.callback(|is_on| Message::SetSwitch(keys::IS_TONE33_ENABLED, is_on)),
            ),
        ),
        reset_row(strings, ResetScope::DictionarySources, context),
    ))
}
