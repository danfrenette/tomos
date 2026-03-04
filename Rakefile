# frozen_string_literal: true

require "bundler/gem_tasks"
require "rspec/core/rake_task"
require "rake/extensiontask"

RSpec::Core::RakeTask.new(:spec)

gemspec = Bundler.load_gemspec("tomos.gemspec")
Rake::ExtensionTask.new("tomos", gemspec) do |ext|
  ext.lib_dir = "lib/tomos"
end

task default: :spec
