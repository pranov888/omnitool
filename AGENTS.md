# Omnitool Technical State and Agent Continuity

This internal tracking document preserves architecture decisions, execution state, and achievement roadmaps for Omnitool across agent sessions.

## 1. Project Architecture and Git Protocol
- Production Branch: `main` (Protected; default branch on GitHub).
- Development Branch: `dev` (Active development; all features land here first before production promotion).
- Workflow: Feature branches branch off `dev`, merge to `dev` via PR, verified through full test suite, then promoted to `main` via release PR.
- Strict Constraints: No emojis anywhere in commit messages, branch names, pull request descriptions, documentation, or code.

## 2. GitHub Achievement Roadmap Progress
- Quickdraw: Completed. Issue/PR lifecycle resolved under 5 minutes.
- Pull Shark:
  - Current Status: Bronze Tier (Multiplier label x2) unlocked on profile.
  - Progression Thresholds:
    - Base (x1): 2 merged PRs.
    - Bronze (x2): 16 merged PRs (Achieved).
    - Silver (x3): 128 merged PRs (In progress).
    - Gold (x4): 1,024 merged PRs.
  - Execution Strategy: Continue steady, legitimate feature PR additions. Do not spam automated PRs to protect account standing against GitHub anti-abuse policies.
- Pair Extraordinaire:
  - Milestone Requirement: Commit co-authored with a valid, active GitHub collaborator merged into the default branch (`main`).
  - Co-author Identity: `ARES <116806983+ARES49999@users.noreply.github.com>`.
  - Execution Status: Commit `ebb7ad9` merged into `dev` via PR #14 and into `main` via PR #15. Awaiting GitHub asynchronous batch queue badge indexing.
- YOLO:
  - Milestone Requirement: Merge a pull request without code review into the default branch.
  - Execution Status: Both PR #15 and PR #16 merged unreviewed directly into `main`. Awaiting GitHub asynchronous batch calculation.
- Galaxy Brain:
  - Milestone Requirement: 2 accepted answers on Q&A discussions asked by other users in public repositories.
  - Execution Status: Discussions #17 and #18 opened by `ARES49999`. Technical answers submitted by `pranov888` and marked as accepted answers. Verified on GitHub backend (`isAnswered: true`). Awaiting GitHub batch calculation.

## 3. Implemented Capabilities
- Crypto (`src/crypto.rs`): SHA-256, SHA-512, MD5, Base64 encode/decode, UUIDv4 generation, secure password generation.
- Codec (`src/codec.rs`): Percent URL encoding/decoding, Hexadecimal encoding/decoding.
- Text (`src/text_tool.rs`): Character/word/line/byte metrics, Case conversions (snake, camel, kebab, upper, lower).
- Time (`src/time_tool.rs`): Epoch timestamps (s, ms), ISO 8601 UTC conversions.
- QR (`src/qr.rs`): Terminal ASCII QR code rendering.
- Telemetry (`src/telemetry.rs`): System hardware overview, TCP probe latency, DNS resolution.
- JSON (`src/json_tool.rs`): Pretty-print formatting, minification, syntax validation.

## 4. Project Rules: Research, Design Standards, and Deployment Protocol
These rules are binding specifically for this project folder and its future evolutions:

### Rule 4.1: Deep Problem Research and Trend Alignment
- Every tool, feature, and sub-project built within this workspace must stem from deep research into real-world developer problems.
- All architecture and tooling choices must stay modern, high-performance, and aligned with current engineering trends rather than relying on obsolete or legacy paradigms.

### Rule 4.2: Dedicated Showcase Website Mandate
- Every project developed here must feature a dedicated, high-aesthetic showcase website.
- The website must fulfill two distinct objectives:
  1. Designer Portfolio: Exhibit world-class UI/UX design craftsmanship, motion design, typography, and polish to highlight design skills.
  2. Clear and Simple Project Explanations: Deconstruct complex technical utilities into simple, intuitive visual guides and interactive demos for developers.

### Rule 4.3: Website Repository Privacy and Vercel Hosting Protocol
- Repository Privacy: The source code repository for each showcase website must remain strictly **private** on GitHub to safeguard design assets, proprietary components, and architecture.
- Public Deployment: The website must be publicly deployed and hosted on **Vercel** for fast edge delivery, live portfolio access, and continuous deployment from the private repository.

## 5. Next Steps
- Monitor GitHub profile achievements tab for Pair Extraordinaire and YOLO recalculation.
- Add file checksum inspection utility (`omnitool crypto checksum <path>`).
- Implement string slugification and reverse utilities in `text_tool`.
- Plan and design the dedicated private showcase website for Omnitool on Vercel.
