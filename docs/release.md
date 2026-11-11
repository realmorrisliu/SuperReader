# macOS release

α1 ships as a signed disk image from GitHub Actions. A person installs it by dragging SuperReader onto Applications. Local `just app` and `just dmg` stay ad hoc for development.

The workflow is `.github/workflows/release.yml`. It runs on a `v*` tag and on `workflow_dispatch`. A tag publishes a GitHub Release. A manual run does not, unless its `publish` input is turned on. This repository does not push that tag from the α1 pull request.

The Release assets are the disk image `SuperReader-<version>.dmg`. α2 can add `latest.json` next to it for nsh-updater. α1 does not write that file. The self-updater landed in NSH after 0.4.0 and is only on nightly, so it stays out of this release.

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

Signing order is the inner `libpdfium.dylib`, then `SuperReader.app` with hardened runtime, a timestamp, and `scripts/entitlements.plist`, then the disk image. `notarytool` uses the API key. `stapler` tickets both the disk image and the app.

## Local development

`SR_SIGN_IDENTITY` defaults to `-` (ad hoc). `SR_NOTARY_PROFILE` is a `notarytool` keychain profile. Leave it unset to skip notarization. When the identity is a Developer ID, `just dmg` signs the disk image and, if the profile is set, submits it and staples it.

`just app` writes `dist/SuperReader.app`. `just dmg` writes `dist/SuperReader-0.1.0-alpha.1.dmg`. `just install` copies the app to `/Applications` on this Mac. The copy is a developer convenience. The release people install is the disk image.

## Finder open

`Info.plist` lists EPUB (`org.idpf.epub-container`) and PDF (`com.adobe.pdf`) after packaging, because `nsh.json` has no document-type field. The patch is `PlistBuddy`, then the bundle is signed again.

NSH 0.4.0 does not deliver those files into the Rust app. `NshAppDelegate` in `nsh-platform-macos` 0.4.0 implements `applicationShouldTerminateAfterLastWindowClosed:`, `applicationShouldTerminate:`, `applicationShouldHandleReopen:hasVisibleWindows:`, and `applicationWillTerminate:`. It does not implement `application:openFile:` or `application:openURLs:`. Finder Open With can show SuperReader. A double-click launches the app and drops the path. File > Open (⌘O) and dropping a book on the window are the paths that open it.
