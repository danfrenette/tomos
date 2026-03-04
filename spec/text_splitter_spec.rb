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
    it "returns a single chunk for short text" do
      chunks = splitter.chunks("Hello, world!")
      expect(chunks).to eq(["Hello, world!"])
    end

    it "returns an empty array for empty input" do
      expect(splitter.chunks("")).to eq([])
    end

    it "splits long prose into multiple chunks" do
      text = "This is a sentence about natural language processing. " * 100
      chunks = splitter.chunks(text)

      expect(chunks.length).to be > 1
      chunks.each { |chunk| expect(chunk).not_to be_empty }
    end

    it "handles a wall of text with no paragraph breaks" do
      words = %w[the quick brown fox jumps over the lazy dog]
      text = Array.new(500) { words.sample }.join(" ")
      chunks = splitter.chunks(text)

      expect(chunks.length).to be > 1
      chunks.each { |chunk| expect(chunk).not_to be_empty }
    end

    context "with overlap" do
      let(:capacity) { 50 }
      let(:overlap) { 10 }

      it "produces chunks that share content at boundaries" do
        text = "First sentence here. Second sentence here. Third sentence here. Fourth sentence here. Fifth sentence here. Sixth sentence here. Seventh sentence here. Eighth sentence here."
        chunks = splitter.chunks(text)

        next if chunks.length < 2

        has_overlap = chunks.each_cons(2).any? do |a, b|
          tail = a.split.last(3).join(" ")
          b.include?(tail)
        end
        expect(has_overlap).to be true
      end
    end
  end
end
