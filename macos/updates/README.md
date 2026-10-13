# macOS update manifest

The macOS input method checks for updates by fetching one static JSON file:

    https://taigikeyboard.tw/appcast/macos.json

It compares `version` against the installed `CFBundleShortVersionString` and,
when the manifest is strictly newer, offers the update: from the settings window
the app downloads `packageURL` itself and opens it in Installer.app, and
everywhere else — the system notification, an old manifest, a development build
— it opens `downloadPageURL` in the browser.

The file is still unsigned, and that is deliberate now that something it names
gets executed. Whoever could rewrite this manifest could name any package;
what stops that package being installed is its own Developer ID signature,
checked against the team that signed the running copy. Signing the manifest as
well would add a second key to protect and would be read by the same code that
already has to distrust it. Gatekeeper cannot make this call alone — it accepts
any correctly notarized package, including one an attacker notarized under their
own Developer ID — which is why the team is pinned rather than inferred.

The readers are `macos/Sources/TaigiInputMethodCore/Updates/UpdateChecker.swift`
(the check) and `UpdateInstallation.swift` / `UpdatePackageVerifier.swift` (the
download and what it has to prove).

## Where it lives

The file itself is **not** in this repository. It is served from the website
repository, `taigikeyboard/taigikeyboard.github.io`, at `appcast/macos.json`.

What puts it there rather than here: **`UpdateChecker.publishedURL` is compiled
into every shipped build** and old installs request it forever, so the manifest
has to sit at a fixed URL on a domain the project controls — one that can be
repointed at different hosting later without stranding them. The package it
announces is a release asset in this repository; the manifest is not.

Keeping a second copy here to review would only give it somewhere to drift, and
a hand-edited manifest can go live before the package it announces exists.

### One published fact, one committed file

A release writes exactly one file per platform over there — this one is
`_data/macos_release.json`; the Windows and Linux files follow the same rule
(`windows/updates/README.md`, `linux-release.md` § No in-app update):

```json
{
  "version": "3.6.6",
  "tag": "desktop-3.6.6",
  "downloadURL": "https://dl.taigikeyboard.tw/desktop/TaigiKeyboard-3.6.6.pkg",
  "releasePageURL": "https://github.com/taigikeyboard/taigikeyboard/releases/tag/desktop-3.6.6"
}
```

That file is what the site's macOS download button links at — straight at the
package so the download starts on one click, which is why its URL carries the
version, and deliberately not `/releases/latest/download/...`, since `latest`
resolves repository-wide and the last release here may be a Windows one.

The manifest is **rendered** from it by the site's own build:
`appcast/macos.json` over there is a Jekyll template reading
`site.data.macos_release`, mapping `releasePageURL` → `downloadPageURL` and
`downloadURL` → `packageURL`. Nothing writes it directly.

Why one file: every GitHub Pages run deploys the tree of *its own* commit, so
two files committed seconds apart can race and leave the manifest a release
behind the download button (it did, on 2026-08-28). One file cannot race with
itself, and it no longer states the version and the two URLs twice under
different names.

## Wire format

| Field | Meaning |
|---|---|
| `version` | Newest downloadable version, dotted integers only (`3.6.5`). No suffixes — the checker rejects them. |
| `downloadPageURL` | Page the user lands on, `https` only. Required: it is the only route a system notification, a development build, or a pre-3.6.6 install has. |
| `packageURL` | The `.pkg` itself, `https` only. Optional. Naming it lets the app download and install without a browser; a manifest without it behaves exactly as manifests did before 3.6.6. |

Unknown extra fields are ignored by old installs, so the format can grow. A
`packageURL` that fails validation is dropped on its own rather than failing the
whole manifest — it carries an added convenience, and one mistake in it must not
be able to silence update checking for every installed copy.

`downloadPageURL` arrives *in* the manifest, so unlike `publishedURL` it is not
baked into any build and can be repointed at any time — at a download page on
the website, for instance, once one exists.

Before the first release the manifest reads `0.0.0`, which is older than every
build in existence and therefore notifies nobody. That is what "nothing has been
published yet" looks like on the wire.

## Publishing a release

Staged, tested, published and announced as one desktop release:
[`docs/architecture/desktop-release.md` § The flow](../../docs/architecture/desktop-release.md#the-flow).
The announcement writes `_data/macos_release.json` only after fetching the
package anonymously (§ What announcing checks). One-time Apple account setup:
[`docs/architecture/macos-release.md`](../../docs/architecture/macos-release.md).
