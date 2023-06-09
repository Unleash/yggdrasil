require 'rspec'
require 'json'

require_relative '../unleash_engine_ffi'

file_path = File.expand_path('../../../client-specification/specifications/01-simple-examples.json', __FILE__)
simple_spec = File.read(file_path)
json_data = JSON.parse(simple_spec)
features_values = json_data['state']
simple_features = features_values.to_json

RSpec.describe UnleashEngine do
  describe '.new_engine_state' do
    it 'returns a non-null pointer' do
      ptr = UnleashEngine.new_engine_state
      expect(ptr).not_to be_nil
      UnleashEngine.free_engine_state(ptr)
    end
  end

  describe '.take_state' do
    it 'takes features without error' do
      ptr = UnleashEngine.new_engine_state
      metric_bucket = UnleashEngine.take_state(ptr, simple_features)
      UnleashEngine.free_engine_state(ptr)
    end
  end

  describe '.is_enabled?' do
    it 'allows is enabled to complete without error' do
      ptr = UnleashEngine.new_engine_state
      metric_bucket = UnleashEngine.take_state(ptr, simple_features)
      result = UnleashEngine.is_enabled?(ptr, 'Feature.A', {})
      expect(result).to be true
      UnleashEngine.free_engine_state(ptr)
    end

    it 'allows metrics bucket to be correctly passed back over the ffi layer' do
      ptr = UnleashEngine.new_engine_state
      metric_bucket = UnleashEngine.take_state(ptr, simple_features)
      result = UnleashEngine.is_enabled?(ptr, 'Feature.A', {})
      metric_bucket = UnleashEngine.take_state(ptr, simple_features)
      expect(metric_bucket["toggles"]["Feature.A"]["yes"]).to be 1
      UnleashEngine.free_engine_state(ptr)
    end
  end

  describe '.get_variant' do
    it 'allows get variant to complete without error' do
      ptr = UnleashEngine.new_engine_state
      metric_bucket = UnleashEngine.take_state(ptr, simple_features)
      result = UnleashEngine.get_variant(ptr, 'Feature.A', {})
      expect(result).to be_a Hash
      expect(result[:name]).to eq 'disabled'
      expect(result[:enabled]).to be false
      expect(result[:payload]).to be_nil
      UnleashEngine.free_engine_state(ptr)
    end
  end
end