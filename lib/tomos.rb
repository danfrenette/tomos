# frozen_string_literal: true

require "tomos/version"
require "tomos/tomos"

module Tomos
  class Voyage
    def self.new(path:)
      _load(File.path(path))
    end
  end

  class Text
    def self.new(capacity:, model: nil, tokenizer: nil, overlap: 0)
      _new(model, tokenizer, capacity, overlap)
    end

    def self.count_tokens(text, model: nil, tokenizer: nil)
      _count_tokens(text, model, tokenizer)
    end
  end

  class Markdown
    def self.new(capacity:, model: nil, tokenizer: nil, overlap: 0)
      _new(model, tokenizer, capacity, overlap)
    end

    def self.count_tokens(text, model: nil, tokenizer: nil)
      _count_tokens(text, model, tokenizer)
    end
  end
end
