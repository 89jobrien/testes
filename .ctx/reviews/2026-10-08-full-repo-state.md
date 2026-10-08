# Joe-conventions review — testes (full repo state, main)

Date: 2026-10-08
Scope: full repo as committed on `main` (src/lib.rs, src/traits.rs, src/types.rs, AGENTS.md,
docs/README.md, docs/architecture.md, docs/evidence.md, docs/handoff.md, docs/macro-plan.md,
docs/validation.md, docs/dimensions/*.md). No pending diff — working tree clean except
newly-added `.ctx/` and `CLAUDE.md`, which are outside this review's scope.

## Repo override check

Repo has both `AGENTS.md` and a repo-level `CLAUDE.md`. Both are internally consistent and
non-conflicting with the baked-in defaults:
- Explicitly forbids publishing/licensing changes without owner approval (matches
  `publish = false`, no `license` field in Cargo.toml — consistent, not a gap).
- Mandates `cargo fmt --check`, `cargo check --all-targets`, `cargo test --all-targets`,
  `cargo test --doc`, `cargo clippy --all-targets -- -D warnings` "when tooling is available" and
  says explicitly that compilation/formatting/lint status has never been verified. No
  `taskit`/`xtask` wrapper — single crate, src/ only. This is a documented override of nothing;
  it's the repo stating its own baseline, and it's accurate as written.

No conflicts found — proceeding on repo's own stated commands.

## Published crate?

No. `Cargo.toml` has `publish = false`, no `license` field, and `AGENTS.md`/`docs/handoff.md`
explicitly flag licensing/publication as an open owner decision. Published-crate obligations
(semver, doc coverage on `pub`, changelog) do not apply yet — correctly left open rather than
silently assumed.

## FINDINGS (most severe first)

1. [baked-in-convention] `src/types.rs:280-283`, `src/types.rs:98-100` — `cargo clippy --all-targets -- -D warnings`
   fails with 2 `collapsible_if` errors (nested `if let` that clippy wants flattened with `&&`).
   This is the repo's own mandated gate (AGENTS.md "Validation", repo CLAUDE.md "Commands") and
   `-D warnings` is a >60%-adoption hard convention regardless of repo-specific instruction — the
   repo has never actually run this command successfully, and `docs/handoff.md`/repo `CLAUDE.md`
   both explicitly flag lint status as unverified. This review is the first time it's been run;
   it fails. Fix: collapse the two nested conditions as clippy's own suggested diff shows (in
   `TestDeclaration::validate` and `EvidenceRecord::validate_shape`).

2. [baked-in-convention] `src/traits.rs`, `src/types.rs` — `cargo fmt --check` fails; the entire
   codebase is unformatted relative to rustfmt defaults (multi-field enum variants on one line,
   un-wrapped trait method signatons, `use` ordering in traits.rs). `cargo fmt --check` is listed
   first in both AGENTS.md's and the repo CLAUDE.md's own required command list. Fix: run
   `cargo fmt` (not just `--check`) before the next commit.

3. [baked-in-convention] No tests exist anywhere in the repo (no `#[cfg(test)]` module, no
   `tests/`). `cargo test --all-targets` and `cargo test --doc` both currently report nothing to
   run. This matches what `docs/handoff.md`'s "First bounded task" step 3 already calls out as
   the next step ("Add inline unit tests for ID construction, declaration validation, evidence
   shape checks, and mutation-rate calculation") — so this is a tracked gap, not a silent one.
   Not a new finding beyond what the repo's own handoff doc already states; noted for
   completeness since the validation commands were actually run as part of this review.

4. [generic-suggestion] `cargo check --all-targets` passes cleanly with no warnings — the one
   gate that was clean. Worth recording since every other validation command's status was
   previously "unverified" per `docs/handoff.md`; this review is the first evidence any of them
   ran.

## GATES RUN

- `cargo fmt --check` — FAIL (large diff, every file unformatted vs rustfmt defaults)
- `cargo check --all-targets` — PASS, no warnings
- `cargo clippy --all-targets -- -D warnings` — FAIL, 2 errors (`clippy::collapsible_if` in
  `src/types.rs:98` and `src/types.rs:280`), both treated as hard errors under `-D warnings`
- `cargo test --all-targets` / `cargo test --doc` — not run separately; no test code exists yet
  to execute, consistent with `docs/handoff.md`'s own "First bounded task" item 3

These are real results against the live working tree on `main` (clean, no pending diff) — not a
historical-commit review, so running gates directly against the tree is the correct mode here
(not an isolated-worktree situation).

## SELF-CHECK

A skeptic would ask whether failing `fmt`/`clippy` is actually a "finding" worth reporting given
`AGENTS.md` itself says these haven't been verified and gates it under "when tooling is
available." It is: the repo's own `CLAUDE.md` says "don't assume `cargo check` passes without
running it" — the natural complement is that once run, the result is reportable, and `-D
warnings` clippy failures are this user's own >60%-adoption hard-finding tier regardless of
whether the repo had previously verified them. A pragmatist would note these are trivially
auto-fixable (`cargo fmt`, clippy's own suggested diffs) and not architectural — correctly
reflected by keeping severity at "convention violation," not escalating to a blocker above the
irreversible-risk tier (there is none here: no force-push, no secrets, no main-branch-direct-edit
risk — this review happened on `main` itself only because the repo's whole history so far is
`main`, which the repo's own commits already did; not a new risk introduced by this review).

## VERDICT

Draft-only repo as documented: types/traits compile cleanly, but `cargo fmt --check` and
`cargo clippy -- -D warnings` — both required by the repo's own AGENTS.md/CLAUDE.md gates — fail
and must be fixed before any further commits; no tests exist yet, consistent with the repo's own
stated next step. No licensing/scope/git-discipline violations found.

## SAVED TO

/Users/joe/dev/testes/.ctx/reviews/2026-10-08-full-repo-state.md
