# frozen_string_literal: true

require_relative "lib/tomos/version"

Gem::Specification.new do |spec|
  spec.name = "tomos"
  spec.version = Tomos::VERSION
  spec.authors = ["Dan Frenette"]
  spec.email = ["dan.r.frenette@gmail.com"]
  spec.summary = "Token-aware text chunking for RAG pipelines, powered by Rust"
  spec.homepage = "https://github.com/danfrenette/tomos"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 4.0.1"

  spec.files = Dir["*.{md,txt}", "{ext,lib}/**/*", "Cargo.*"]
  spec.require_path = "lib"
  spec.extensions = ["ext/tomos/extconf.rb"]

  spec.add_dependency "rb_sys"
end
