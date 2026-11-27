//! Every GUID this text service owns. Reallocated for HakSpring-Kipan
//! to isolate COM/TSF registration from upstream TaigiKeyboard.

use windows::core::GUID;

/// The COM class the host creates (`CLSID\{…}\InProcServer32`).
pub const CLSID_TEXT_SERVICE: GUID = GUID::from_u128(0x57109325_54F7_4C4B_A09C_28E1DA812201);
/// The one language profile, under LANGID 0x0404.
pub const GUID_PROFILE: GUID = GUID::from_u128(0x1E72B723_6AAA_46D5_8E30_F8E817FCD2D6);
/// The display attribute the composition's preedit is drawn with (PR5b).
pub const GUID_DISPLAY_ATTRIBUTE_INPUT: GUID =
    GUID::from_u128(0xFD6DB448_6159_4AEE_A6E8_AEF5F99EF9BB);
/// The UI-less candidate list element (`ITfUIElement::GetGUID`).
pub const GUID_CANDIDATE_UI_ELEMENT: GUID = GUID::from_u128(0xB3F64718_BCA6_4848_83D8_E73486244390);
/// Preserved key: open the settings window (PR5b registers it).
pub const GUID_PRESERVED_KEY_SETTINGS: GUID =
    GUID::from_u128(0x64F5D1E4_B44F_47E7_AAC2_AEF2AB876723);
/// Preserved key: TL ↔ POJ (PR5b registers it).
pub const GUID_PRESERVED_KEY_ROMANIZATION: GUID =
    GUID::from_u128(0x1DBF8884_B786_4C64_A7CD_701AC5FF4B78);
/// Preserved key: TPS ↔ the romanization last used (allocated 2026-10-03,
/// desktop TPS P3).
pub const GUID_PRESERVED_KEY_TPS: GUID = GUID::from_u128(0x9DC7D742_9165_4507_B384_D51D5302FB24);
/// Preserved key: show / hide the TPS key panel (allocated 2026-10-04,
/// desktop TPS P5).
pub const GUID_PRESERVED_KEY_TPS_KEYBOARD: GUID =
    GUID::from_u128(0x3547B54C_599C_4A0D_8C0E_6DB652FEFFEC);
/// Preserved key: cycle Candidate Display (allocated 2026-09-02 for the shortcut).
pub const GUID_PRESERVED_KEY_CYCLE_CANDIDATE_DISPLAY_MODE: GUID =
    GUID::from_u128(0x5CC3E0FB_CA05_4539_9D11_4F321231F2D2);
/// Preserved key: toggle the symbol picker (allocated 2026-09-11: the key sink
/// never saw the chord, `ShortcutAction::needs_key_context`).
pub const GUID_PRESERVED_KEY_SYMBOL_PICKER: GUID =
    GUID::from_u128(0x443A3E57_A1C1_4F95_ADD6_3DD1FA239D1E);
/// Preserved key: toggle the Telex guide (allocated 2026-09-09 for the shortcut).
pub const GUID_PRESERVED_KEY_TELEX_GUIDE: GUID =
    GUID::from_u128(0x0B611E6B_19DF_493A_AF6D_81D2F5C05ABF);
/// Preserved key: toggle dual-script annotation (HakSpring-Kipan Phase 2).
pub const GUID_PRESERVED_KEY_TOGGLE_BOTH_SCRIPTS: GUID =
    GUID::from_u128(0x69CC157E_BF58_4E4D_9C21_5521C1F577E6);
/// Preserved key: cycle dual-script annotation template (HakSpring-Kipan Phase 2).
pub const GUID_PRESERVED_KEY_CYCLE_ANNOTATION_TEMPLATE: GUID =
    GUID::from_u128(0x25B3BE65_CFD5_4385_9058_A822D918DB5F);
/// Preserved key: cycle active lect (HakSpring-Kipan Phase 2).
pub const GUID_PRESERVED_KEY_CYCLE_LECT: GUID =
    GUID::from_u128(0x2B1DA411_B197_44F9_B1EB_EC88CCB84C9C);

/// Traditional Chinese (Taiwan). The ONLY profile language (roadmap W7,
/// Codex F5): no duplicate under en-US.
pub const LANGID_ZH_TW: u16 = 0x0404;
