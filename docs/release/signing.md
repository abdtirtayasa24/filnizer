# Release Signing with SignPath.io Open Source

Filnizer release artifacts should be signed through the SignPath.io Open Source flow. Do not commit certificates, private keys, API tokens, or signing credentials to this repository.

## Intended flow

1. Build the portable Windows artifact with `npm run package:portable`.
2. Upload `artifacts/Filnizer-<version>-portable-windows-x64.zip` as a CI artifact.
3. Configure the Filnizer project in SignPath.io as an Open Source project.
4. Connect the repository/build pipeline to SignPath using the provider-supported integration.
5. Submit the portable ZIP or contained `Filnizer.exe` for signing according to the approved SignPath project policy.
6. Download the signed release artifact from the signing workflow.
7. Run the release smoke checklist before publishing.

## Repository rules

- Signing credentials must live only in the CI/signing provider secret store.
- Do not place `.pfx`, `.p12`, private keys, certificates with private material, or API tokens in git.
- Do not print signing tokens or certificate metadata that exposes secrets in logs.
- Keep unsigned and signed artifacts clearly named in CI output.

## CI placeholder

`.github/workflows/release-signing-placeholder.yml` builds and uploads the unsigned portable artifact. A maintainer should replace the explicit placeholder step with the approved SignPath.io integration after SignPath project onboarding is complete.
