# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it ships.

## [Unreleased]

### Added

- macOS spike shell (`spike-shell`) on NSH 0.4.0: PDF via pdfium-render, EPUB via WKWebView, Knuth–Plass justification with on-disk line-break cache, trim presets, window chrome prototypes.
- Rust workspace crates: `spike-report`, `spike-epub`, `spike-pdf`, `spike-measure`, `spike-shell`.
- `scripts/verify` UI verification harness and `docs/verify` records.
- Open-source docs: README, CONTRIBUTING, SECURITY, CI, Dependabot, issue forms.
- Dual license **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`).

### Changed

- EPUB direction under evaluation toward an owned pure-Rust typesetter (WKWebView remains default until then).
