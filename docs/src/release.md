# Release guide

Releases use one version across Cargo, PyPI, npm, and the GitHub artifacts consumed by the Go bootstrap.

## One-time registry setup

1. Reserve `rfc-agent-cli` on crates.io, PyPI, and npm.
2. Configure trusted publishing or narrowly scoped release tokens in GitHub Actions.
3. In repository **Settings → Pages**, select **GitHub Actions** as the Pages source.
4. Protect the `main` branch and require the CI workflow before merging.

## Preparing a release

1. Merge a green pull request into `main`.
2. Choose the release version and update it consistently in `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md`. The Python version is derived from Cargo metadata.
3. Run the local checks:

   ```console
   cargo fmt --all -- --check
   cargo clippy --all-targets --locked -- -D warnings
   cargo test --locked
   go test ./cmd/rfc
   python -m maturin build --release --bindings bin --out dist
   mdbook build
   ```

4. Commit the version change and push a tag such as `v0.2.0-alpha.1`.

The tag starts cargo-dist, which builds the cross-platform archives, installers, checksums, npm package artifacts, attestations, and GitHub Release.

## Publishing registries

Publish only after the GitHub Release succeeds so npm and Go always resolve existing binary archives.

- Cargo: `cargo publish --locked`
- PyPI: publish the Maturin wheels using PyPI trusted publishing
- npm: publish the generated npm package using npm trusted publishing with provenance
- Go: no separate registry upload is needed; the module tag and GitHub Release are the distribution mechanism

For a prerelease, keep the same semantic version everywhere and use the appropriate npm dist-tag, such as `next`, rather than `latest`.
