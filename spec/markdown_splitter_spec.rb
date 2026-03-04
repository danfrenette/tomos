# frozen_string_literal: true

require "spec_helper"

RSpec.describe Tomos::Markdown do
  subject(:splitter) { described_class.new("gpt-4", capacity, overlap) }

  let(:capacity) { 100 }
  let(:overlap) { 0 }

  describe "#chunks" do
    it "returns a single chunk for short text" do
      chunks = splitter.chunks("# Hello\n\nWorld")
      expect(chunks).to eq(["# Hello\n\nWorld"])
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

      section_one_chunks = chunks.select { |c| c.include?("section one") }
      section_two_chunks = chunks.select { |c| c.include?("section two") }

      expect(section_one_chunks).not_to be_empty
      expect(section_two_chunks).not_to be_empty

      section_one_chunks.each do |chunk|
        expect(chunk).not_to include("section two")
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
      code_chunk = chunks.find { |c| c.include?("def hello") }

      expect(code_chunk).to include("```ruby")
      expect(code_chunk).to include("```")
    end

    it "handles plain text gracefully (no markdown structure)" do
      text = "Just a plain paragraph with no markdown formatting at all. " * 50
      chunks = splitter.chunks(text)

      expect(chunks.length).to be > 1
      chunks.each { |chunk| expect(chunk).not_to be_empty }
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
      list_chunk = chunks.find { |c| c.include?("Apples") }
      expect(list_chunk).to include("Bananas")
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
end
