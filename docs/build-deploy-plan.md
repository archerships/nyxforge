# NyxForge Build and Deploy Plan

Status: planning spec as of 2026-04-25.

Goal: develop NyxForge as a Flutter + Rust/WASM app, then ship the CLI/backend
and app shell across macOS/Homebrew, Debian/apt, Android/F-Droid and
Obsidian-adjacent workflows, Arch/pacman, SUSE, OpenBSD, and Windows. The plan
separates the primary development target from package-manager integration and
portable binary strategy.

## 1. Distribution Principles

- Ship native package metadata for each ecosystem even when the payload is a
  portable binary.
- Make the Flutter frontend plus Rust/WASM backend module the primary
  development target for user-facing product work.
- Keep the CLI/backend artifact independent from Flutter, Rust/WASM module, and
  Android packaging.
- Prefer reproducible release artifacts with checksums and signatures.
- Keep plugin boundaries visible in packaging: core CLI first, optional feature
  packs later.
- Treat APE as a useful CLI artifact, not as a replacement for `.deb`,
  Homebrew formulae, APKs, OpenBSD ports, or Windows installers.
- Keep source builds available for distro maintainers even when publishing
  binary packages.

## 2. Artifact Matrix

| Artifact | Purpose | Target |
| :--- | :--- | :--- |
| Flutter web app + `nyxforge_web.wasm` | Primary app development artifact: Flutter UI, Rust/WASM backend logic | browser, local webview shells, hosted demo |
| `nyx` native Rust binary | Primary CLI/backend executable | package managers and source builds |
| `nyx.com` APE binary | Portable CLI experiment / Windows-first single-file CLI | Windows, Linux/macOS/BSD smoke tests |
| `nyxforge-hub` desktop shell | Desktop wrapper around the app/backend | macOS/Linux packages after WASM app stabilizes |
| Android APK/AAB | Mobile app or companion app | F-Droid and direct test builds |
| Obsidian plugin package | Notes/research workflow integration, if built | Obsidian community/plugin install |
| checksums/signatures/SBOM | release integrity | all platforms |

## 3. Primary Development Target: Flutter + Rust/WASM App

The primary user-facing development target is a Flutter app backed by Rust
compiled to WebAssembly:

- Flutter owns layout, navigation, forms, and interaction state
- Rust owns `.bounty` parsing, validation, cryptographic checks, deterministic
  serialization, and other performance/security-sensitive backend logic
- Rust is compiled to WebAssembly for browser and webview execution
- mock backend or local RPC during early UI work
- packaged later into desktop/mobile shells if needed

This follows the cross-platform split described in the Flutter/Rust/WASM
reference article: Flutter provides UI flexibility, while Rust/WASM handles
computationally intensive or correctness-critical logic.

This changes the meaning of the OS package targets:

- Homebrew/apt/pacman/SUSE/OpenBSD primarily install the CLI/backend and may
  optionally install a static Flutter web + Rust/WASM app bundle.
- Desktop app packages are wrappers around the stabilized Flutter/Rust/WASM app,
  not the first implementation target.
- Android/F-Droid should package the Flutter frontend plus Rust/WASM backend
  module through a native Android shell only after mobile scope is explicit.
- Obsidian integration remains a separate plugin/import-export track.

Development gates:

- app loads from static files without CDN dependencies
- browser smoke test passes in Chromium and Firefox
- offline/local-first flows work against mock data
- `.bounty` inspect/verify path works through Rust/WASM or local RPC
- Dart-to-WASM interop is covered by tests for success, failure, and malformed
  input paths
- generated app bundle has reproducible hashes
- accessibility and keyboard navigation pass smoke checks

## 4. APE Strategy

APE can be incorporated into other installation methods as the installed payload:

- Homebrew can install an APE file into `bin/nyx` or `bin/nyx.com`.
- Debian/apt can ship an APE inside a `.deb`.
- Arch/pacman can ship an APE inside a `pkg.tar.zst`.
- SUSE can ship an APE inside an `.rpm`.
- OpenBSD ports/packages can install an APE if it runs under the supported
  OpenBSD release and passes local policy checks.
- Windows can use the APE directly as `nyx.com`, or package it inside a
  winget/Scoop/MSIX/zip workflow later.

APE does not replace package formats. Package managers still need native
metadata: name, version, architecture, dependencies, license, checksums,
install path, man pages, shell completions, and conflict rules.

### 4.1 Recommended APE role

Use APE as an experimental or secondary CLI artifact until proven:

- `nyx-${version}.com` for direct download and Windows testing
- optional payload in packages for platforms where it passes smoke tests
- not the first packaging path for the Flutter/Rust/WASM app or desktop shell
- not relevant to Android APK/F-Droid packaging

### 4.2 APE risks

- Rust + Cosmopolitan support must be proven. Cosmopolitan is strongest for
  C/C++ CLI programs; NyxForge's Rust dependencies may not cross-compile cleanly
  to APE without a shim or constrained feature set.
- SQLite/SQLCipher, networking, DNS, file locking, terminal behavior, and crypto
  libraries need platform smoke tests.
- macOS codesigning/notarization expectations may favor native Mach-O builds for
  GUI or mainstream distribution.
- Linux distro policy may prefer source-built native binaries, even if an APE
  works technically.
- Architecture metadata still matters. A package containing an APE should be
  labelled for the architectures it is tested on, not treated as magical
  architecture-independent data.

### 4.3 APE acceptance gate

Before using APE in any official package:

- `nyx --version` works on macOS, Debian, Arch, SUSE, OpenBSD, and Windows.
- `nyx --help` renders correctly in default terminals.
- `.bounty` create/inspect/verify works in a temp directory.
- network/DNS operations used by node or judge commands work.
- SQLite and file locking behave correctly.
- process exit codes match native builds.
- binary can be stripped/released with separate debug artifact.

## 5. Platform Plans

### 5.1 Homebrew on macOS

Initial package: `nyx`.

Plan:

- publish a tap, for example `nyxforge/tap`
- formula installs native macOS Rust binary first
- include shell completions and man page when available
- optionally expose APE as `nyx-ape` or install APE only if it passes macOS
  smoke tests
- optionally install static Flutter web + Rust/WASM app assets under the package
  prefix
- later add cask for `nyxforge-hub.app` when the desktop shell is ready

Gate:

- `brew install nyxforge/tap/nyx`
- `nyx --version`
- `nyx bounty create --help`
- key generation and `.bounty` temp-file tests

### 5.2 Debian/apt

Initial package: `.deb` for Debian stable and Ubuntu LTS.

Plan:

- package native Linux builds as the default
- install CLI at `/usr/bin/nyx`
- install docs under `/usr/share/doc/nyxforge`
- install man pages under `/usr/share/man`
- ship shell completions if generated
- optionally ship static Flutter web + Rust/WASM app assets under
  `/usr/share/nyxforge/web`
- use maintainer scripts only when needed
- optional separate package: `nyxforge-ape`, if APE is useful and policy-safe

Gate:

- `dpkg -i nyxforge_${version}_${arch}.deb`
- `apt install ./nyxforge_${version}_${arch}.deb`
- `nyx --version`
- Python CLI smoke tests with `NYX_BIN=/usr/bin/nyx`

### 5.3 Arch Linux / pacman

Initial package: `PKGBUILD` for AUR or custom repository.

Plan:

- source package builds from release tarball with Cargo
- binary package installs `/usr/bin/nyx`
- optional `nyxforge-bin` package installs prebuilt native binary
- optional `nyxforge-ape-bin` installs tested APE
- optional web assets package installs the static Flutter web + Rust/WASM app
  bundle
- include completions/man pages

Gate:

- `makepkg -si`
- `pacman -Qi nyxforge`
- CLI smoke tests in a clean Arch container or VM

### 5.4 SUSE Linux

Initial package: RPM, preferably via OBS once release process stabilizes.

Plan:

- build native RPM from source where possible
- install `/usr/bin/nyx`, docs, completions, man pages
- optionally install static Flutter web + Rust/WASM app assets
- optional APE payload only after openSUSE smoke tests
- keep spec file independent from Debian packaging

Gate:

- `zypper install ./nyxforge-${version}.${arch}.rpm`
- CLI smoke tests on openSUSE Leap and Tumbleweed

### 5.5 OpenBSD

Initial package: ports-style source build or binary tarball for testers.

Plan:

- prioritize CLI/backend and static Flutter web + Rust/WASM app assets, not a
  heavy desktop UI
- validate Rust dependencies on OpenBSD
- avoid Linux-specific assumptions in file paths, sockets, and process handling
- APE may be tested, but a normal OpenBSD port/source build is more acceptable
  for long-term distribution

Gate:

- `cargo build --release` on OpenBSD
- `nyx --version`
- `.bounty` create/inspect/verify smoke tests
- network and file-locking tests

### 5.6 Windows

Initial package: direct zip with `nyx.exe` or `nyx.com`, then package-manager
integration after CLI stabilizes.

Plan:

- native Rust Windows build is baseline for the CLI/backend
- static Flutter web + Rust/WASM app bundle can ship in the same zip for
  browser/local shell use
- APE `nyx.com` is a strong candidate for direct Windows CLI distribution if
  the Rust dependency stack can be built under Cosmopolitan or isolated behind a
  C-compatible shim
- later add Scoop/winget manifests
- keep debug symbols separate

Gate:

- PowerShell: `.\nyx.exe --version` or `.\nyx.com --version`
- create/inspect/verify `.bounty` in `%TEMP%`
- path, Unicode, terminal, and exit-code tests

### 5.7 Android / F-Droid

Initial package: Android app only after a mobile/companion scope is explicit.

Plan:

- use standard Android APK/AAB builds, not APE
- package the Flutter frontend plus Rust/WASM backend through a minimal native
  Android shell when mobile scope is explicit
- maintain F-Droid metadata separately
- ensure all dependencies are open source and reproducible
- avoid Google Play-only services
- for Obsidian-adjacent workflows, consider an import/export or share-target
  companion rather than embedding Obsidian

Gate:

- reproducible Gradle build
- F-Droid scanner passes
- no proprietary SDKs
- import/export `.bounty` test
- offline-first operation where possible

### 5.8 Obsidian

Obsidian is not an OS package target. Treat it as a separate integration track.

Plan:

- optional Obsidian plugin for reading/writing bounty notes, evidence manifests,
  or research dossiers
- package as a normal Obsidian plugin: `manifest.json`, `main.js`, `styles.css`
- do not depend on APE inside the plugin
- communicate with `nyx` through explicit file import/export or localhost only
  if the user enables it

Gate:

- plugin loads in Obsidian desktop
- no network access unless user enables it
- can parse/export `.bounty` metadata or evidence manifest safely

## 6. Release Pipeline

### 6.1 Build jobs

Minimum release jobs:

- Flutter web + Rust/WASM app: static bundle, Chromium/Firefox smoke tests
- macOS native CLI: arm64 and x86_64 if feasible
- Linux native CLI: x86_64 first, arm64 later
- Windows native CLI: x86_64
- OpenBSD source-build smoke test
- Android APK when mobile scope exists
- APE experimental job

### 6.2 Package jobs

Package artifacts:

- Homebrew formula/tap update
- `.deb`
- Arch `PKGBUILD`
- RPM spec/package for SUSE
- OpenBSD port notes or tester tarball
- Windows zip plus future Scoop/winget manifests
- Android APK/F-Droid metadata when ready
- static Flutter web + Rust/WASM app bundle and checksums

### 6.3 Verification jobs

Every packaged artifact must pass:

- Flutter web + Rust/WASM app loads from local static files
- `nyx --version`
- `nyx --help`
- `nyx bounty create --help`
- temp-dir `.bounty` create/inspect/verify smoke test
- checksum verification
- signature verification when signing is enabled

Additional gates:

- `python3 bin/consistency-check`
- relevant Python CLI tests with `NYX_BIN` pointed at the packaged binary
- Rust crate tests before packaging
- stagenet E2E before MVP release candidate

## 7. Versioning and Signing

Recommended:

- semantic versioning for CLI/backend
- release tag `nyxforge-vX.Y.Z`
- artifact names include version, platform, architecture, and artifact type
- publish SHA-256 sums
- sign release checksums with minisign, signify, or GPG
- generate SBOM once dependencies stabilize

Example names:

- `nyxforge-cli-v0.1.0-macos-arm64.tar.gz`
- `nyxforge-cli-v0.1.0-linux-x86_64.tar.gz`
- `nyxforge-cli-v0.1.0-windows-x86_64.zip`
- `nyxforge-cli-v0.1.0-ape.com`
- `nyxforge-web-v0.1.0.tar.gz`
- `nyxforge_0.1.0_amd64.deb`
- `nyxforge-0.1.0-1-x86_64.pkg.tar.zst`
- `nyxforge-0.1.0-1.x86_64.rpm`

## 8. Recommended Implementation Order

1. Define release artifact names and smoke-test commands.
2. Build the Flutter frontend plus Rust/WASM backend module as the primary
   user-facing development artifact.
3. Add Dart-to-WASM interop tests and browser smoke tests for local static app
   loading.
4. Build native CLI binaries for macOS, Linux, and Windows.
5. Add Homebrew formula for macOS.
6. Add `.deb` packaging for Debian/Ubuntu.
7. Add Arch `PKGBUILD`.
8. Add SUSE RPM spec.
9. Add OpenBSD source-build instructions and smoke tests.
10. Prototype APE CLI build and decide whether it is official, experimental, or
   direct-download only.
11. Add Android/F-Droid only after mobile scope is defined.
12. Add Obsidian plugin only after the file/evidence workflow is stable.

## 9. Answer: Can APE Be Incorporated Into Other Installers?

Yes. APE can be incorporated as the executable payload inside Homebrew, apt,
pacman, SUSE RPM, OpenBSD package workflows, and Windows zip/package-manager
workflows.

But APE is not itself a package-manager format. It does not provide package
metadata, dependency policy, file ownership, update channels, signatures,
maintainer scripts, man pages, desktop integration, Android manifests, or
F-Droid metadata. Those still belong to each native packaging ecosystem.

Recommended NyxForge policy:

- native builds are the default official package payload until APE is proven
- APE is an experimental direct-download artifact for the CLI
- APE may become the payload of `-bin` style packages after the acceptance gate
- Android, Flutter/Rust/WASM app, and desktop-shell distribution should not use
  APE
