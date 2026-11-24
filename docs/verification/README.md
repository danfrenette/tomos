# Verification

- [Voyage text and Markdown tokenization](voyage.md) covers the pinned artifact, both splitters, standalone counting, and tiktoken compatibility.

Use the Ruby version in `.ruby-version` and the Rust toolchain described in [Installation](../../README.md#installation). The recipes use `rbenv exec` to select the pinned Ruby. From the checkout root, run `rbenv exec bundle check` to confirm dependencies and `cargo --version` to confirm the Rust toolchain. Run `rbenv exec bundle install` first if the Ruby dependencies are missing.

The recipes compile the native extension and run one-shot commands. Record the tested Git revision and command results in the PR or an ignored run directory. No server or database is needed.
