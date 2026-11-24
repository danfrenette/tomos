# frozen_string_literal: true

require "bundler/gem_tasks"
require "rspec/core/rake_task"
require "rake/extensiontask"

RSpec::Core::RakeTask.new(:spec)

namespace :verify do
  desc "Verify Voyage tokenization against the pinned local tokenizer artifact"
  RSpec::Core::RakeTask.new(:voyage) do |task|
    task.pattern = "spec/verification/voyage_tokenizer.rb"
  end
end

gemspec = Bundler.load_gemspec("tomos.gemspec")
Rake::ExtensionTask.new("tomos", gemspec) do |ext|
  ext.lib_dir = "lib/tomos"
end

task default: :spec
