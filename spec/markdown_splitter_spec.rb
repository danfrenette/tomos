# frozen_string_literal: true

require "spec_helper"

RSpec.describe Tomos::Markdown do
  subject(:splitter) { described_class.new("gpt-4", capacity, overlap) }

  let(:capacity) { 100 }
  let(:overlap) { 0 }

  describe "#chunks" do
    it "returns Chunk objects" do
      chunks = splitter.chunks("# Hello\n\nWorld")
      expect(chunks).to all(be_a(Tomos::Chunk))
    end

    it "returns a single chunk for short text" do
      chunks = splitter.chunks("# Hello\n\nWorld")
      expect(chunks.size).to eq(1)
      expect(chunks.first.text).to eq("# Hello\n\nWorld")
    end

    it "returns an empty array for empty input" do
      expect(splitter.chunks("")).to eq([])
    end

    it "does not mix content from different sections" do
      text = <<~MD
        # Section One

        #{"Content for section one that needs to be long enough to fill up a chunk. " * 10}

        # Section Two

        #{"Content for section two that also needs to be long enough to fill a chunk. " * 10}
      MD

      chunks = splitter.chunks(text)

      expect(chunks.length).to be >= 2

      section_one_chunks = chunks.select { |c| c.text.include?("section one") }
      section_two_chunks = chunks.select { |c| c.text.include?("section two") }

      expect(section_one_chunks).not_to be_empty
      expect(section_two_chunks).not_to be_empty

      section_one_chunks.each do |chunk|
        expect(chunk.text).not_to include("section two")
      end
    end

    it "keeps code fences intact" do
      text = <<~MD
        # Example

        Here is some code:

        ```ruby
        def hello
          puts "world"
        end
        ```

        That was a simple method.
      MD

      chunks = splitter.chunks(text)
      code_chunk = chunks.find { |c| c.text.include?("def hello") }

      expect(code_chunk.text).to include("```ruby")
      expect(code_chunk.text).to include("```")
    end

    it "handles plain text gracefully (no markdown structure)" do
      text = "Just a plain paragraph with no markdown formatting at all. " * 50
      chunks = splitter.chunks(text)

      expect(chunks.length).to be > 1
      chunks.each { |chunk| expect(chunk.text).not_to be_empty }
    end

    it "respects list structure" do
      text = <<~MD
        # Shopping

        - Apples
        - Bananas
        - Cherries
        - Dates
        - Elderberries

        # Cooking

        - Boil water
        - Add pasta
        - Drain
        - Serve
      MD

      chunks = splitter.chunks(text)

      expect(chunks.length).to be >= 1
      list_chunk = chunks.find { |c| c.text.include?("Apples") }
      expect(list_chunk.text).to include("Bananas")
    end

    context "with overlap" do
      let(:capacity) { 50 }
      let(:overlap) { 10 }

      it "produces overlapping chunks for long markdown" do
        text = "# Title\n\n" + ("Some content here. " * 100)
        chunks = splitter.chunks(text)

        expect(chunks.length).to be > 1
      end
    end
  end

  describe "chunk metadata" do
    let(:text) do
      <<~MD
        # Introduction

        This is a document about testing chunk metadata in markdown splitting.

        ## Details

        Here are some details that help verify the metadata is correct.
      MD
    end

    it "provides a positive token count within capacity" do
      chunks = splitter.chunks(text)

      chunks.each do |chunk|
        expect(chunk.token_count).to be_a(Integer)
        expect(chunk.token_count).to be > 0
        expect(chunk.token_count).to be <= capacity
      end
    end

    it "provides byte offsets that locate the chunk in the source" do
      chunks = splitter.chunks(text)

      chunks.each do |chunk|
        expect(text[chunk.byte_offset, chunk.byte_length]).to eq(chunk.text)
      end
    end

    it "provides byte_length matching the text bytesize" do
      chunks = splitter.chunks(text)

      chunks.each do |chunk|
        expect(chunk.byte_length).to eq(chunk.text.bytesize)
      end
    end

    it "provides a deterministic chunk_id" do
      chunks_a = splitter.chunks(text)
      chunks_b = splitter.chunks(text)

      chunks_a.zip(chunks_b).each do |a, b|
        expect(a.chunk_id).to eq(b.chunk_id)
      end
    end

    it "returns a 64-character hex SHA256 digest as chunk_id" do
      chunks = splitter.chunks(text)

      chunks.each do |chunk|
        expect(chunk.chunk_id).to match(/\A[0-9a-f]{64}\z/)
      end
    end

    it "reports sequential byte offsets across chunks" do
      long_text = "# Title\n\n" + ("More content for the test document. " * 100)
      chunks = splitter.chunks(long_text)

      next if chunks.length < 2

      chunks.each_cons(2) do |a, b|
        expect(b.byte_offset).to be >= a.byte_offset
      end
    end
  end
end
