# Voyage tokenization

A Ruby caller can use one pinned `voyage-code-4` tokenizer for text and Markdown chunk sizing, overlap, metadata, and independent token counts. Byte offsets identify the original excerpts, including decomposed Unicode and repeated passages.

## Prepare

Run the [dependency checks](README.md), compile the extension, and fetch the public tokenizer to an ignored directory. Downloading the artifact requires network access and no credentials.

```sh
rbenv exec bundle exec rake compile
mkdir -p tmp/voyage-reference
curl --fail --location \
  --output tmp/voyage-reference/tokenizer.json \
  https://huggingface.co/MongoDB/voyage-code-4/resolve/ac780b405e54ff8b3aff681270e0e03840fed2f3/tokenizer.json
shasum -a 256 tmp/voyage-reference/tokenizer.json
```

Expect SHA-256 `c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539`.

## Verify

```sh
TOMOS_VOYAGE_TOKENIZER=tmp/voyage-reference/tokenizer.json rbenv exec bundle exec rake verify:voyage
rbenv exec bundle exec rake spec
```

The Voyage recipe exercises `.new`, `#chunks`, `.count_tokens`, and `#count_tokens` on `Tomos::Text` and `Tomos::Markdown`. It checks prose, code, Unicode and literal special markers against counts recorded from `voyageai` 0.6.0 `Client.count_tokens` with the same pinned file. It checks capacity and overlap, byte round trips, and rejection of missing or different artifacts. The regular suite covers existing `model:` callers.

Record the Git revision, artifact SHA-256, and both test results. The check uses the provider's local tokenizer reference. It makes no request to a hosted embedding service. The artifact can be removed when it is no longer needed. The harness removes its temporary invalid file.
