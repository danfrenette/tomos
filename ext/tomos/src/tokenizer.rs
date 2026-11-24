use std::{fs, sync::Arc};

use magnus::{typed_data::Obj, Error, Ruby};
use sha2::{Digest, Sha256};
use text_splitter::ChunkSizer;
use tiktoken_rs::{get_bpe_from_model, CoreBPE};
use tokenizers::Tokenizer as HfTokenizer;

use crate::RbResult;

#[derive(Clone)]
pub(crate) enum Tokenizer {
    Tiktoken(Arc<CoreBPE>),
    Voyage(Arc<HfTokenizer>),
}

impl Tokenizer {
    pub(crate) fn resolve(
        ruby: &Ruby,
        model: Option<String>,
        voyage: Option<Obj<RbVoyage>>,
    ) -> RbResult<Self> {
        match (model, voyage) {
            (Some(model), None) => get_bpe_from_model(&model)
                .map(|bpe| Self::Tiktoken(Arc::new(bpe)))
                .map_err(|e| {
                    Error::new(
                        ruby.exception_arg_error(),
                        format!("unrecognized tiktoken model '{model}': {e}"),
                    )
                }),
            (None, Some(voyage)) => Ok(Self::Voyage(Arc::clone(&voyage.tokenizer))),
            _ => Err(Error::new(
                ruby.exception_arg_error(),
                "provide either model: or tokenizer:",
            )),
        }
    }
}

impl ChunkSizer for Tokenizer {
    fn size(&self, text: &str) -> usize {
        match self {
            Self::Tiktoken(bpe) => bpe.size(text),
            Self::Voyage(tokenizer) => tokenizer
                .encode_fast(text, true)
                .expect("pinned Voyage tokenizer must encode valid UTF-8")
                .len(),
        }
    }
}

#[magnus::wrap(class = "Tomos::Voyage", free_immediately, size)]
pub(crate) struct RbVoyage {
    tokenizer: Arc<HfTokenizer>,
}

impl RbVoyage {
    const MODEL: &str = "voyage-code-4";
    const REVISION: &str = "ac780b405e54ff8b3aff681270e0e03840fed2f3";
    const SHA256: &str = "c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539";

    pub(crate) fn load(ruby: &Ruby, path: String) -> RbResult<Self> {
        let bytes = fs::read(&path).map_err(|e| {
            Error::new(
                ruby.exception_arg_error(),
                format!("cannot read Voyage tokenizer at '{path}': {e}"),
            )
        })?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        if digest != Self::SHA256 {
            return Err(Error::new(
                ruby.exception_arg_error(),
                format!(
                    "Voyage tokenizer must match {} revision {} (SHA-256 {})",
                    Self::MODEL,
                    Self::REVISION,
                    Self::SHA256
                ),
            ));
        }
        let tokenizer = HfTokenizer::from_bytes(bytes).map_err(|e| {
            Error::new(
                ruby.exception_arg_error(),
                format!("cannot load Voyage tokenizer: {e}"),
            )
        })?;
        Ok(Self {
            tokenizer: Arc::new(tokenizer),
        })
    }

    pub(crate) fn model(&self) -> &'static str {
        Self::MODEL
    }

    pub(crate) fn revision(&self) -> &'static str {
        Self::REVISION
    }
}
