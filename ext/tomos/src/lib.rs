//! Tomos – Ruby-native text chunking backed by [`text_splitter`].
//!
//! Exposes `Tomos::Text` and `Tomos::Markdown` to Ruby via Magnus. Each wraps a
//! token-aware splitter that produces `Tomos::Chunk` objects carrying metadata
//! (token count, byte offset/length, SHA-256 chunk ID).

mod tokenizer;

use magnus::{function, method, prelude::*, typed_data::Obj, Error, Ruby};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, ChunkSizer, MarkdownSplitter, TextSplitter};
use tokenizer::{RbVoyage, Tokenizer};

type RbResult<T> = Result<T, Error>;

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
    tokenizer: &Tokenizer,
) -> magnus::RArray {
    let ary = ruby.ary_new();
    for (offset, slice) in iter {
        let _ = ary.push(RbChunk {
            text: slice.to_owned(),
            token_count: tokenizer.size(slice),
            byte_offset: offset,
            byte_length: slice.len(),
            chunk_id: chunk_id(slice),
        });
    }
    ary
}

/// Standalone token-counting entry point exposed by both splitters.
fn count_tokens(
    ruby: &Ruby,
    text: String,
    model: Option<String>,
    voyage: Option<Obj<RbVoyage>>,
) -> RbResult<usize> {
    Ok(Tokenizer::resolve(ruby, model, voyage)?.size(&text))
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
            tokenizer: Tokenizer,
        }

        impl $name {
            fn new(
                ruby: &Ruby,
                model: Option<String>,
                voyage: Option<Obj<RbVoyage>>,
                capacity: usize,
                overlap: usize,
            ) -> RbResult<Self> {
                let tokenizer = Tokenizer::resolve(ruby, model, voyage)?;
                let config = ChunkConfig::new(capacity)
                    .with_overlap(overlap)
                    .map_err(|e| {
                        Error::new(
                            ruby.exception_arg_error(),
                            format!("invalid chunk config: {e}"),
                        )
                    })?
                    .with_sizer(tokenizer.clone());
                Ok(Self {
                    splitter: <$splitter>::new(config),
                    tokenizer,
                })
            }

            fn chunks(&self, text: String) -> magnus::RArray {
                let ruby = Ruby::get().expect("chunks called outside Ruby thread");
                collect_chunks(&ruby, self.splitter.chunk_indices(&text), &self.tokenizer)
            }

            fn count_tokens(&self, text: String) -> usize {
                self.tokenizer.size(&text)
            }
        }
    };
}

define_splitter!(RbText, "Tomos::Text", TextSplitter<Tokenizer>);
define_splitter!(RbMarkdown, "Tomos::Markdown", MarkdownSplitter<Tokenizer>);

#[magnus::init]
fn init(ruby: &Ruby) -> RbResult<()> {
    let module = ruby.define_module("Tomos")?;

    let voyage = module.define_class("Voyage", ruby.class_object())?;
    voyage.define_singleton_method("_load", function!(RbVoyage::load, 1))?;
    voyage.define_method("model", method!(RbVoyage::model, 0))?;
    voyage.define_method("revision", method!(RbVoyage::revision, 0))?;

    let chunk = module.define_class("Chunk", ruby.class_object())?;
    chunk.define_method("text", method!(RbChunk::text, 0))?;
    chunk.define_method("token_count", method!(RbChunk::token_count, 0))?;
    chunk.define_method("byte_offset", method!(RbChunk::byte_offset, 0))?;
    chunk.define_method("byte_length", method!(RbChunk::byte_length, 0))?;
    chunk.define_method("chunk_id", method!(RbChunk::chunk_id, 0))?;

    let text = module.define_class("Text", ruby.class_object())?;
    text.define_singleton_method("_new", function!(RbText::new, 4))?;
    text.define_method("chunks", method!(RbText::chunks, 1))?;
    text.define_method("count_tokens", method!(RbText::count_tokens, 1))?;
    text.define_singleton_method("_count_tokens", function!(count_tokens, 3))?;

    let markdown = module.define_class("Markdown", ruby.class_object())?;
    markdown.define_singleton_method("_new", function!(RbMarkdown::new, 4))?;
    markdown.define_method("chunks", method!(RbMarkdown::chunks, 1))?;
    markdown.define_method("count_tokens", method!(RbMarkdown::count_tokens, 1))?;
    markdown.define_singleton_method("_count_tokens", function!(count_tokens, 3))?;

    Ok(())
}
