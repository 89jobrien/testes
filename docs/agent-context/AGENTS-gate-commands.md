# Gate commands

Read this before running the validation suite, or before reporting that a change was validated.

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo clippy --workspace --all-targets -- -D warnings
```

Run all five after any change and report exact results — don't claim validation if
tooling is unavailable, say so instead. There is no `taskit`/`xtask` wrapper in this
repo; these are the literal commands, run across the full workspace.
