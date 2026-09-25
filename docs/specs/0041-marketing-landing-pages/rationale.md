# Rationale — 0041. Marketing & Commercial Landing Pages

## Why First-Class Monorepo Integration?
Keeping the commercial landing site in the same `pnpm` workspace ensures that TypeScript, ESLint (`no-explicit-any`, named-exports-only), and Vitest checks run uniformly across both the Tauri desktop UI (`ui/`) and the commercial web application (`Landing pages/`).

## Why Bangladesh Coaching Center Data (`৳ BDT` & `EN` / `বাংলা`)?
Prospective coaching center directors and administrators in Bangladesh evaluate software based on whether it understands their daily reality: `HSC`/`SSC`/Admission batches, `৳` (BDT) fee collection via `Cash`, `bKash`, and `Nagad`, guardian SMS notifications, and bilingual English/Bangla interfaces. Updating `appData.ts` and pricing (`৳14,900` Single-PC, `৳29,900` Multi-PC LAN) eliminates the disconnect between the commercial website and the core desktop app.

## Why Real Release Metadata & Validated Lead Flows?
Desktop software buyers need confidence when downloading a Windows `.exe` installer. Providing the release version (`v1.0.0`), SHA-256 checksum verification, a working download artifact, and validated lead/contact forms with local + webhook persistence turns the landing site into a sell-ready commercial distribution channel.
