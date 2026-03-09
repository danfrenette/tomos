//! Tomos – Ruby-native text chunking backed by [`text_splitter`] and [`tiktoken_rs`].
//!
//! Exposes `Tomos::Text` and `Tomos::Markdown` to Ruby via Magnus. Each wraps a
//! token-aware splitter that produces `Tomos::Chunk` objects carrying metadata
//! (token count, byte offset/length, SHA-256 chunk ID).

use magnus::{function, method, prelude::*, Error, Ruby};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, MarkdownSplitter, TextSplitter};
use tiktoken_rs::get_bpe_from_model;

type RbResult<T> = Result<T, Error>;

/// Resolve a tiktoken BPE tokenizer by model name, mapping unknown models to
/// a Ruby `ArgumentError`.
fn resolve_bpe(ruby: &Ruby, model: &str) -> RbResult<tiktoken_rs::CoreBPE> {
    get_bpe_from_model(model).map_err(|e| {
        Error::new(
            ruby.exception_arg_error(),
            format!("unrecognized tiktoken model '{model}': {e}"),
        )
    })
}

/// Build a [`ChunkConfig`] with the given token `capacity` and `overlap`,
/// sized by the BPE tokenizer for `model`.
fn build_chunk_config(
    ruby: &Ruby,
    model: &str,
    capacity: usize,
    overlap: usize,
) -> RbResult<ChunkConfig<tiktoken_rs::CoreBPE>> {
    let bpe = resolve_bpe(ruby, model)?;

    let config = ChunkConfig::new(capacity)
        .with_overlap(overlap)
        .map_err(|e| {
            Error::new(
                ruby.exception_arg_error(),
                format!("invalid chunk config: {e}"),
            )
        })?
        .with_sizer(bpe);

    Ok(config)
}

/// Produce a hex-encoded SHA-256 digest of `text` for use as a chunk identifier.
fn chunk_id(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Collect splitter output into a Ruby array of [`RbChunk`] objects.
fn collect_chunks<'a>(
    ruby: &Ruby,
    iter: impl Iterator<Item = (usize, &'a str)>,
    bpe: &tiktoken_rs::CoreBPE,
) -> magnus::RArray {
    let ary = ruby.ary_new();
    for (offset, slice) in iter {
        let _ = ary.push(RbChunk {
            text: slice.to_owned(),
            token_count: bpe.encode_ordinary(slice).len(),
            byte_offset: offset,
            byte_length: slice.len(),
            chunk_id: chunk_id(slice),
        });
    }
    ary
}

/// Count tokens in `text` using the given BPE tokenizer.
fn count_tokens_with_bpe(bpe: &tiktoken_rs::CoreBPE, text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    bpe.encode_ordinary(text).len()
}

/// Standalone token-counting entry point exposed as a class-level method on
/// both `Tomos::Text` and `Tomos::Markdown`.
fn count_tokens(ruby: &Ruby, text: String, model: String) -> RbResult<usize> {
    let bpe = resolve_bpe(ruby, &model)?;
    Ok(count_tokens_with_bpe(&bpe, &text))
}

/// A single chunk produced by a splitter, exposed to Ruby as `Tomos::Chunk`.
#[derive(Debug)]
#[magnus::wrap(class = "Tomos::Chunk", free_immediately, size)]
struct RbChunk {
    text: String,
    token_count: usize,
    byte_offset: usize,
    byte_length: usize,
    chunk_id: String,
}

impl RbChunk {
    fn text(&self) -> &str {
        &self.text
    }

    fn token_count(&self) -> usize {
        self.token_count
    }

    fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    fn byte_length(&self) -> usize {
        self.byte_length
    }

    fn chunk_id(&self) -> &str {
        &self.chunk_id
    }
}

/// Generates a splitter wrapper struct with `new`, `chunks`, and `count_tokens`
/// methods. Both `RbText` and `RbMarkdown` share identical structure and logic,
/// differing only in the underlying splitter type.
macro_rules! define_splitter {
    ($name:ident, $class:literal, $splitter:ty) => {
        #[magnus::wrap(class = $class, free_immediately, size)]
        struct $name {
            splitter: $splitter,
            bpe: tiktoken_rs::CoreBPE,
        }

        impl $name {
            fn new(
                ruby: &Ruby,
                model: String,
                capacity: usize,
                overlap: usize,
            ) -> RbResult<Self> {
                let config = build_chunk_config(ruby, &model, capacity, overlap)?;
                let bpe = resolve_bpe(ruby, &model)?;
                Ok(Self {
                    splitter: <$splitter>::new(config),
                    bpe,
                })
            }

            fn chunks(&self, text: String) -> magnus::RArray {
                let ruby = Ruby::get().expect("chunks called outside Ruby thread");
                collect_chunks(&ruby, self.splitter.chunk_indices(&text), &self.bpe)
            }

            fn count_tokens(&self, text: String) -> usize {
                count_tokens_with_bpe(&self.bpe, &text)
            }
        }
    };
}

define_splitter!(RbText, "Tomos::Text", TextSplitter<tiktoken_rs::CoreBPE>);
define_splitter!(
    RbMarkdown,
    "Tomos::Markdown",
    MarkdownSplitter<tiktoken_rs::CoreBPE>
);

#[magnus::init]
fn init(ruby: &Ruby) -> RbResult<()> {
    let module = ruby.define_module("Tomos")?;

    let chunk = module.define_class("Chunk", ruby.class_object())?;
    chunk.define_method("text", method!(RbChunk::text, 0))?;
    chunk.define_method("token_count", method!(RbChunk::token_count, 0))?;
    chunk.define_method("byte_offset", method!(RbChunk::byte_offset, 0))?;
    chunk.define_method("byte_length", method!(RbChunk::byte_length, 0))?;
    chunk.define_method("chunk_id", method!(RbChunk::chunk_id, 0))?;

    let text = module.define_class("Text", ruby.class_object())?;
    text.define_singleton_method("_new", function!(RbText::new, 3))?;
    text.define_method("chunks", method!(RbText::chunks, 1))?;
    text.define_method("count_tokens", method!(RbText::count_tokens, 1))?;
    text.define_singleton_method("_count_tokens", function!(count_tokens, 2))?;

    let markdown = module.define_class("Markdown", ruby.class_object())?;
    markdown.define_singleton_method("_new", function!(RbMarkdown::new, 3))?;
    markdown.define_method("chunks", method!(RbMarkdown::chunks, 1))?;
    markdown.define_method("count_tokens", method!(RbMarkdown::count_tokens, 1))?;
    markdown.define_singleton_method("_count_tokens", function!(count_tokens, 2))?;

    Ok(())
}
