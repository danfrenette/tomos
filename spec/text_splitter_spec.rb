# frozen_string_literal: true

require "spec_helper"

RSpec.describe Tomos::Text do
  subject(:splitter) { described_class.new("gpt-4", capacity, overlap) }

  let(:capacity) { 100 }
  let(:overlap) { 10 }

  describe ".new" do
    it "creates a splitter with valid arguments" do
      expect(splitter).to be_a(described_class)
    end

    it "raises ArgumentError for an unknown model" do
      expect { described_class.new("nonexistent-model", 100, 0) }
        .to raise_error(ArgumentError, /unrecognized tiktoken model/)
    end

    it "raises ArgumentError when overlap exceeds capacity" do
      expect { described_class.new("gpt-4", 10, 20) }
        .to raise_error(ArgumentError, /invalid chunk config/)
    end
  end

  describe "#chunks" do
    it "returns Chunk objects" do
      chunks = splitter.chunks("Hello, world!")
      expect(chunks).to all(be_a(Tomos::Chunk))
    end

    it "returns a single chunk for short text" do
      chunks = splitter.chunks("Hello, world!")
      expect(chunks.size).to eq(1)
      expect(chunks.first.text).to eq("Hello, world!")
    end

    it "returns an empty array for empty input" do
      expect(splitter.chunks("")).to eq([])
    end

    it "splits long prose into multiple chunks" do
      text = "This is a sentence about natural language processing. " * 100
      chunks = splitter.chunks(text)

      expect(chunks.length).to be > 1
      chunks.each { |chunk| expect(chunk.text).not_to be_empty }
    end

    it "handles a wall of text with no paragraph breaks" do
      words = %w[the quick brown fox jumps over the lazy dog]
      text = Array.new(500) { words.sample }.join(" ")
      chunks = splitter.chunks(text)

      expect(chunks.length).to be > 1
      chunks.each { |chunk| expect(chunk.text).not_to be_empty }
    end

    context "with overlap" do
      let(:capacity) { 50 }
      let(:overlap) { 10 }

      it "produces chunks that share content at boundaries" do
        text = "First sentence here. Second sentence here. Third sentence here. Fourth sentence here. Fifth sentence here. Sixth sentence here. Seventh sentence here. Eighth sentence here."
        chunks = splitter.chunks(text)

        next if chunks.length < 2

        has_overlap = chunks.each_cons(2).any? do |a, b|
          tail = a.text.split.last(3).join(" ")
          b.text.include?(tail)
        end
        expect(has_overlap).to be true
      end
    end
  end

  describe "chunk metadata" do
    let(:text) { "Hello, world! This is a test of chunk metadata." }

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

    it "provides a deterministic chunk_id for the same content" do
      chunks_a = splitter.chunks(text)
      chunks_b = splitter.chunks(text)

      chunks_a.zip(chunks_b).each do |a, b|
        expect(a.chunk_id).to eq(b.chunk_id)
      end
    end

    it "produces different chunk_ids for different content" do
      text = (1..100).map { |i| "Sentence number #{i} with unique words. " }.join
      chunks = splitter.chunks(text)
      ids = chunks.map(&:chunk_id)

      expect(ids.uniq.size).to eq(ids.size)
    end

    it "returns a 64-character hex SHA256 digest as chunk_id" do
      chunks = splitter.chunks(text)

      chunks.each do |chunk|
        expect(chunk.chunk_id).to match(/\A[0-9a-f]{64}\z/)
      end
    end

    it "reports sequential byte offsets across chunks" do
      long_text = "This is a sentence about testing. " * 100
      chunks = splitter.chunks(long_text)

      next if chunks.length < 2

      chunks.each_cons(2) do |a, b|
        expect(b.byte_offset).to be >= a.byte_offset
      end
    end
  end
end
