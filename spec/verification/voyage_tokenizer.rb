# frozen_string_literal: true

require "spec_helper"
require "tempfile"

RSpec.describe "Voyage tokenization" do
  let(:tokenizer_path) { ENV.fetch("TOMOS_VOYAGE_TOKENIZER") }
  let(:tokenizer) { Tomos::Voyage.new(path: tokenizer_path) }
  let(:reference_counts) do
    # voyageai 0.6.0, MongoDB/voyage-code-4 at ac780b405e54ff8b3aff681270e0e03840fed2f3.
    {
      "A clock returns the current time. A timer schedules a retry." => 13,
      "def expired?(clock)\n  clock.now >= deadline\nend" => 12,
      "Cafe\u0301, déjà vu, 雪景色, 👩🏽‍💻" => 18,
      "<|im_start|>مرحبًا<|im_end|> and <|endoftext|>" => 8,
      "<|endoftext|>" => 1,
      "" => 0
    }
  end

  it "identifies the pinned model and artifact revision" do
    expect(tokenizer.model).to eq("voyage-code-4")
    expect(tokenizer.revision).to eq("ac780b405e54ff8b3aff681270e0e03840fed2f3")
  end

  it "rejects a missing artifact" do
    expect { Tomos::Voyage.new(path: "#{tokenizer_path}.missing") }
      .to raise_error(ArgumentError, /cannot read Voyage tokenizer/)
  end

  it "rejects content that differs from the pinned artifact" do
    Tempfile.create("tomos-voyage-artifact") do |file|
      file.write("{}")
      file.flush

      expect { Tomos::Voyage.new(path: file.path) }
        .to raise_error(ArgumentError, /Voyage tokenizer must match voyage-code-4 revision/)
    end
  end

  [Tomos::Text, Tomos::Markdown].each do |splitter_class|
    context "with #{splitter_class}" do
      it "matches Voyage's reference counts on the class and instance methods" do
        splitter = splitter_class.new(tokenizer: tokenizer, capacity: 12)

        reference_counts.each do |text, expected|
          expect(splitter_class.count_tokens(text, tokenizer: tokenizer)).to eq(expected)
          expect(splitter.count_tokens(text)).to eq(expected)
        end
      end

      it "fits a literal provider marker into a one-token chunk" do
        marker = "<|endoftext|>"
        chunks = splitter_class.new(tokenizer: tokenizer, capacity: 1).chunks(marker)

        expect(chunks.map { |chunk| [chunk.text, chunk.token_count] }).to eq([[marker, 1]])
      end

      it "returns exact, overlapping byte ranges with counts bounded by the configured capacity" do
        paragraph = ("Cafe\u0301 雪 clock timer retry ticks now " * 5).strip
        source = (splitter_class == Tomos::Markdown) ? "# Retry\n\n#{paragraph}\n\n```ruby\nputs :done\n```\n" : paragraph
        splitter = splitter_class.new(tokenizer: tokenizer, capacity: 12, overlap: 2)
        chunks = splitter.chunks(source)

        chunks.each do |chunk|
          expect(source.byteslice(chunk.byte_offset, chunk.byte_length)).to eq(chunk.text)
          expect(chunk.token_count).to be_between(1, 12)
          expect(chunk.token_count).to eq(splitter.count_tokens(chunk.text))
        end

        overlaps = chunks.each_cons(2).filter_map do |left, right|
          shared_bytes = left.byte_offset + left.byte_length - right.byte_offset
          source.byteslice(right.byte_offset, shared_bytes) if shared_bytes.positive?
        end
        expect(overlaps).to include("now Cafe\u0301")
        expect(overlaps.map { |text| splitter.count_tokens(text) }).to all(be_between(1, 2))
        without_overlap = splitter_class.new(tokenizer: tokenizer, capacity: 12).chunks(source)
        without_overlap.each_cons(2) do |left, right|
          expect(left.byte_offset + left.byte_length).to be <= right.byte_offset
        end
      end

      it "requires an unambiguous tokenizer choice" do
        expect { splitter_class.new(capacity: 12) }.to raise_error(ArgumentError, /either model: or tokenizer:/)
        expect { splitter_class.count_tokens("text") }.to raise_error(ArgumentError, /either model: or tokenizer:/)
        expect { splitter_class.new(model: "gpt-4", tokenizer: tokenizer, capacity: 12) }
          .to raise_error(ArgumentError, /either model: or tokenizer:/)
        expect { splitter_class.count_tokens("text", model: "gpt-4", tokenizer: tokenizer) }
          .to raise_error(ArgumentError, /either model: or tokenizer:/)
      end
    end
  end
end
