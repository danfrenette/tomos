use magnus::{function, method, prelude::*, Error, Ruby};
use text_splitter::{ChunkConfig, MarkdownSplitter, TextSplitter};
use tiktoken_rs::get_bpe_from_model;

type RbResult<T> = Result<T, Error>;

fn build_chunk_config(
    ruby: &Ruby,
    model: &str,
    capacity: usize,
    overlap: usize,
) -> RbResult<ChunkConfig<tiktoken_rs::CoreBPE>> {
    let bpe = get_bpe_from_model(model).map_err(|e| {
        Error::new(
            ruby.exception_arg_error(),
            format!("unrecognized tiktoken model '{model}': {e}"),
        )
    })?;

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

/// Splits unstructured text along Unicode boundaries (sentences, words,
/// grapheme clusters). Works well on transcripts and content with no
/// paragraph or section structure.
#[magnus::wrap(class = "Tomos::Text", free_immediately, size)]
struct RbText {
    splitter: TextSplitter<tiktoken_rs::CoreBPE>,
}

impl RbText {
    fn new(ruby: &Ruby, model: String, capacity: usize, overlap: usize) -> RbResult<Self> {
        let config = build_chunk_config(ruby, &model, capacity, overlap)?;
        Ok(Self {
            splitter: TextSplitter::new(config),
        })
    }

    fn chunks(&self, text: String) -> Vec<String> {
        self.splitter.chunks(&text).map(str::to_owned).collect()
    }
}

/// Splits CommonMark/GFM markdown along structural boundaries (headings,
/// code fences, list items, block elements) in addition to the Unicode
/// levels that `RbText` uses. Degrades gracefully to plain-text splitting
/// when the input contains no markdown structure.
#[magnus::wrap(class = "Tomos::Markdown", free_immediately, size)]
struct RbMarkdown {
    splitter: MarkdownSplitter<tiktoken_rs::CoreBPE>,
}

impl RbMarkdown {
    fn new(ruby: &Ruby, model: String, capacity: usize, overlap: usize) -> RbResult<Self> {
        let config = build_chunk_config(ruby, &model, capacity, overlap)?;
        Ok(Self {
            splitter: MarkdownSplitter::new(config),
        })
    }

    fn chunks(&self, text: String) -> Vec<String> {
        self.splitter.chunks(&text).map(str::to_owned).collect()
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> RbResult<()> {
    let module = ruby.define_module("Tomos")?;

    let text = module.define_class("Text", ruby.class_object())?;
    text.define_singleton_method("new", function!(RbText::new, 3))?;
    text.define_method("chunks", method!(RbText::chunks, 1))?;

    let markdown = module.define_class("Markdown", ruby.class_object())?;
    markdown.define_singleton_method("new", function!(RbMarkdown::new, 3))?;
    markdown.define_method("chunks", method!(RbMarkdown::chunks, 1))?;

    Ok(())
}
