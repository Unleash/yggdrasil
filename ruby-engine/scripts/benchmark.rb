require 'benchmark/ips'
require_relative '../lib/unleash_engine'

unleash_engine = UnleashEngine.new
suite_path = File.join('../client-specification/specifications', '01-simple-examples.json')
suite_data = JSON.parse(File.read(suite_path))
json_client_features = suite_data['state'].to_json

Benchmark.ips do |x|
  x.report("enabled") { unleash_engine.enabled?('Feature.A', {}) }
end