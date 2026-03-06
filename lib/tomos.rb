# frozen_string_literal: true

require "tomos/version"
require "tomos/tomos"

module Tomos
  class Text
    def self.new(model:, capacity:, overlap: 0)
      _new(model, capacity, overlap)
    end
  end

  class Markdown
    def self.new(model:, capacity:, overlap: 0)
      _new(model, capacity, overlap)
    end
  end
end
