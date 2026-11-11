# macOS release

α1 ships as a signed disk image from GitHub Actions. A person installs it by dragging Slowpage onto Applications. Local `just app` and `just dmg` stay ad hoc for development.

The workflow is `.github/workflows/release.yml`. It runs on a `v*` tag and on `workflow_dispatch`. A tag publishes a GitHub Release. A manual run does not, unless its `publish` input is turned on. This repository does not push that tag from the α1 pull request.

The Release assets are the disk image `Slowpage-<version>.dmg`. α2 can add `latest.json` next to it for nsh-updater. α1 does not write that file. The self-updater landed in NSH after 0.4.0 and is only on nightly, so it stays out of this release.

## Secrets

The signed job reads these GitHub Actions secrets from the environment and never prints them. A missing name fails the job with `missing secret: <NAME>`. It does not fall back to an ad hoc signature.

| Secret | Role |
| --- | --- |
| `MACOS_CERT_P12_BASE64` | Base64 of the Developer ID Application PKCS#12 |
| `MACOS_CERT_P12_PASSWORD` | Password for that PKCS#12 |
| `APPLE_TEAM_ID` | Apple team id inside the certificate name |
| `ASC_API_KEY_P8` | App Store Connect API key, PEM or base64 PEM |
| `ASC_API_KEY_ID` | Key id for `notarytool` |
| `ASC_API_ISSUER_ID` | Issuer id for `notarytool` |

The job creates a temporary keychain with a random password, imports the certificate, signs, notarizes, staples, and deletes the keychain.

Notarization returns HTTP 403 when the Apple Developer Program agreement is missing or expired. The job still uploads the signed disk image, then fails. Sign the agreement at [developer.apple.com/account](https://developer.apple.com/account) and run the workflow again. That is an account step, not a missing secret.

The app executable is the release `spike-shell` binary. Packaging reads the name, bundle id, and icon from `spikes/shell/nsh.json`, copies that binary into `Contents/MacOS`, and finishes load commands, the icon, `Info.plist`, and `zh-Hans.lproj/InfoPlist.strings` before the first `codesign`. The icon source is `assets/icon/Slowpage.svg`. `spike-icon` rasterizes every icns size from 16 through 1024, and the 16 and 32 point @1x slots use a wider-eyed slab with the page sliver and back rim left out. `scripts/check-reader-bundle.sh` fails the build when `CFBundleExecutable` is not that binary, the bundle id is not `app.slowpage.reader`, or the binary contains the text SuperReader spike. After signing, `scripts/packaged-open.sh` launches the app with `open`, then opens a fixture EPUB and a fixture PDF, and uploads a screenshot of each window.

Signing order is the inner `libpdfium.dylib`, then `Slowpage.app` with hardened runtime, a timestamp, and `scripts/entitlements.plist`, then the disk image. After the app is signed, and again for the copy inside the mounted disk image, the build runs `codesign --verify --deep --strict --verbose=2`. A failed verify fails the job. `notarytool` uses the API key. `stapler` tickets both the disk image and the app.

## Local development

`SR_SIGN_IDENTITY` defaults to `-` (ad hoc). `SR_NOTARY_PROFILE` is a `notarytool` keychain profile. Leave it unset to skip notarization. When the identity is a Developer ID, `just dmg` signs the disk image and, if the profile is set, submits it and staples it.

`just app` writes `dist/Slowpage.app`. `just dmg` writes `dist/Slowpage-0.1.0-alpha.1.dmg`. `just install` copies the app to `/Applications` on this Mac. The copy is a developer convenience. The release people install is the disk image.

## Finder open

`Info.plist` lists EPUB (`org.idpf.epub-container`) and PDF (`com.adobe.pdf`). `nsh.json` has no document-type field, so the package script writes the types before it signs.

NSH 0.4.0 does not deliver those files into the Rust app by itself. `main` adds `application:openURLs:` to `NshAppDelegate` before `run_app`, so Finder, `open -a`, and a Dock drop open the book. File > Open (⌘O) and dropping a book on the window do the same.
