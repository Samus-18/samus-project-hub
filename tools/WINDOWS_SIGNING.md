# Windows publisher signing and verification

Vanta FX V136 is **unsigned**, not a Windows Verified Publisher build. The developer PC had no accessible Authenticode code-signing certificate or key in CurrentUser/My.

- GitHub Release SHA-256, manifest hashes and protected download URL establish byte integrity/checkability, **not developer identity**, virus scanning or SmartScreen reputation.
- Real production signing needs a reputable CA issued code-signing certificate / managed Microsoft Artifact Signing, associated protected private key, SHA-256 Authenticode timestamping, and post-signature verification with Windows signtool and Get-AuthenticodeSignature.
- Reproducibly build/sign the final executable before computing checksum. Do not alter signed bytes after signing. Never commit any key.
- A validated helper lives in tools/sign-vanta-windows.ps1. It refuses missing/expired/non-code-signing certs and verifies output.
- Aurora AI NSIS installer also needs legitimate publisher signing. Thalvoryn ZIP distribution additionally needs rights clearance for all third-party structures.
