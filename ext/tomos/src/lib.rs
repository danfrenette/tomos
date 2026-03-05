use magnus::{function, method, prelude::*, Error, Ruby};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, MarkdownSplitter, TextSplitter};
use tiktoken_rs::get_bpe_from_model;

type RbResult<T> = Result<T, Error>;

fn resolve_bpe(ruby: &Ruby, model: &str) -> RbResult<tiktoken_rs::CoreBPE> {
    get_bpe_from_model(model).map_err(|e| {
        Error::new(
            ruby.exception_arg_error(),
            format!("unrecognized tiktoken model '{model}': {e}"),
        )
    })
}

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

fn chunk_id(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn collect_chunks<'a>(
    iter: impl Iterator<Item = (usize, &'a str)>,
    bpe: &tiktoken_rs::CoreBPE,
) -> magnus::RArray {
    let ary = Ruby::get().unwrap().ary_new();
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

#[magnus::wrap(class = "Tomos::Text", free_immediately, size)]
struct RbText {
    splitter: TextSplitter<tiktoken_rs::CoreBPE>,
    bpe: tiktoken_rs::CoreBPE,
}

impl RbText {
    fn new(ruby: &Ruby, model: String, capacity: usize, overlap: usize) -> RbResult<Self> {
        let config = build_chunk_config(ruby, &model, capacity, overlap)?;
        let bpe = resolve_bpe(ruby, &model)?;
        Ok(Self {
            splitter: TextSplitter::new(config),
            bpe,
        })
    }

    fn chunks(&self, text: String) -> magnus::RArray {
        collect_chunks(self.splitter.chunk_indices(&text), &self.bpe)
    }
}

#[magnus::wrap(class = "Tomos::Markdown", free_immediately, size)]
struct RbMarkdown {
    splitter: MarkdownSplitter<tiktoken_rs::CoreBPE>,
    bpe: tiktoken_rs::CoreBPE,
}

impl RbMarkdown {
    fn new(ruby: &Ruby, model: String, capacity: usize, overlap: usize) -> RbResult<Self> {
        let config = build_chunk_config(ruby, &model, capacity, overlap)?;
        let bpe = resolve_bpe(ruby, &model)?;
        Ok(Self {
            splitter: MarkdownSplitter::new(config),
            bpe,
        })
    }

    fn chunks(&self, text: String) -> magnus::RArray {
        collect_chunks(self.splitter.chunk_indices(&text), &self.bpe)
    }
}

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
    text.define_singleton_method("new", function!(RbText::new, 3))?;
    text.define_method("chunks", method!(RbText::chunks, 1))?;

    let markdown = module.define_class("Markdown", ruby.class_object())?;
    markdown.define_singleton_method("new", function!(RbMarkdown::new, 3))?;
    markdown.define_method("chunks", method!(RbMarkdown::chunks, 1))?;

    Ok(())
}
