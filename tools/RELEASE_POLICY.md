# Publication rules

- Aurora AI 0.8.6: publish matching installer + SHA-256; describe live integration testing accurately. Installer should be a prerelease if full runtime acceptance is pending.
- Vanta FX V136: a developer prerelease only, not a production game-proof release. Keep previous stable/released candidate on its own separate version.
- Thalvoryn V86: review-only informational prerelease, NO complete ZIP assets until third-party NBT/structure redistribution rights are verified.
- Release tags are immutable; new code goes into the next tag. Source README, projects.json and Pages status must be updated in the same batch.
- Check SHA-256 before and after upload. Do not upload private chat exports, secrets, generated logs or machine-local toolchain paths.
