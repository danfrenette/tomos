# tomos

Token-aware text chunking for RAG pipelines, powered by Rust.

Tomos wraps the [`text-splitter`](https://github.com/benbrandt/text-splitter) Rust crate with [tiktoken](https://github.com/zurawiki/tiktoken-rs) tokenization and exposes two splitter classes to Ruby: `Tomos::Text` for plain text and `Tomos::Markdown` for Markdown documents. Each chunk carries its token count, byte position, and a SHA-256 content fingerprint.

## Installation

```ruby
gem "tomos"
```

Because tomos includes a native Rust extension, you'll need a Rust toolchain installed. The gem compiles on `bundle install`.

## Usage

### Splitting text into chunks

```ruby
splitter = Tomos::Text.new(model: "gpt-4", capacity: 512)
chunks = splitter.chunks("A long document goes here...")

chunks.each do |chunk|
  chunk.text         # => String  — the chunk content
  chunk.token_count  # => Integer — tokens in this chunk
  chunk.byte_offset  # => Integer — start position in the original string
  chunk.byte_length  # => Integer — byte length of the chunk
  chunk.chunk_id     # => String  — 64-char SHA-256 hex digest
end
```

The `capacity` is the maximum number of tokens per chunk. An optional `overlap` keyword shares tokens between adjacent chunks, which helps preserve context at boundaries:

```ruby
splitter = Tomos::Text.new(model: "gpt-4", capacity: 512, overlap: 50)
```

### Splitting Markdown

`Tomos::Markdown` is Markdown-structure-aware — it respects headers, lists, and code fences when deciding where to split:

```ruby
splitter = Tomos::Markdown.new(model: "gpt-4", capacity: 512)
chunks = splitter.chunks(File.read("document.md"))
```

Note: tokenization is over the raw input string regardless of splitter type; `Markdown` differs only in where it chooses split boundaries.

### Counting tokens

Count tokens directly without constructing a splitter:

```ruby
# Class method — resolves the tokenizer fresh each call
Tomos::Text.count_tokens("Hello, world!", model: "gpt-4")
# => 4

Tomos::Markdown.count_tokens("# Hello\n\nWorld", model: "gpt-4")
# => 4
```

If you already have a splitter instance, the instance method reuses its already-resolved tokenizer:

```ruby
splitter = Tomos::Text.new(model: "gpt-4", capacity: 512)
splitter.count_tokens("Hello, world!")
# => 4
```

Both forms return `0` for empty input and raise `ArgumentError` for unrecognized model names.

### Supported models

Any model name recognized by tiktoken, including:

- `gpt-4`, `gpt-4o`, `gpt-4.1`, `gpt-5`
- `o1`, `o3`, `o4` and their versioned variants (e.g. `o1-mini`, `gpt-4o-2024-05-13`)
- `gpt-3.5-turbo`
- `text-embedding-ada-002`, `text-embedding-3-small`, `text-embedding-3-large`

Unrecognized model names raise `ArgumentError`.

## Chunk metadata

Each `Tomos::Chunk` exposes:

| Method | Type | Description |
|---|---|---|
| `text` | `String` | The chunk content |
| `token_count` | `Integer` | Number of tokens in this chunk |
| `byte_offset` | `Integer` | Start byte position in the source string |
| `byte_length` | `Integer` | Byte length of the chunk |
| `chunk_id` | `String` | 64-char lowercase SHA-256 hex digest of the chunk text |

The byte metadata lets you map a chunk back to its exact position in the source:

```ruby
source[chunk.byte_offset, chunk.byte_length] == chunk.text # => true
```

The `chunk_id` is deterministic — the same text always produces the same ID, regardless of model, capacity, or overlap.

## License

MIT
