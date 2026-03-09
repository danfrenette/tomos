# Changelog

## Unreleased

### Added

- `Tomos::Text.count_tokens(text, model:)` and `Tomos::Markdown.count_tokens(text, model:)` class methods for counting tokens without constructing a splitter.
- `#count_tokens(text)` instance method on both `Tomos::Text` and `Tomos::Markdown`, reusing the BPE tokenizer already resolved at construction time.
- Both forms return `0` for empty input and raise `ArgumentError` for unrecognized models, consistent with the existing splitter API.

### Deprecated

Using `Tomos::Text.new(model:, capacity: <very_large_number>)` and reading `chunks(text).first.token_count` as a workaround for token counting is no longer necessary. Use `count_tokens` instead.
