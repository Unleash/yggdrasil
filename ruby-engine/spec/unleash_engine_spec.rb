require 'rspec'
require 'json'

require_relative '../unleash_engine_ffi'

file_path = File.expand_path('../../../client-specification/specifications/01-simple-examples.json', __FILE__)
simple_spec = File.read(file_path)
json_data = JSON.parse(simple_spec)
features_values = json_data['state']
simple_features = features_values.to_json

RSpec.describe 'Client Specification' do
  let(:unleash_engine) { UnleashEngine.new }
  let(:index_file_path) { "../client-specification/specifications/index.json" }
  let(:test_suites) { JSON.parse(File.read(index_file_path)) }

  it 'passes the test suites' do
    test_suites.each do |suite|
      suite_path = File.join("../client-specification/specifications", suite)
      suite_data = JSON.parse(File.read(suite_path))

      unleash_engine.take_state(suite_data["state"].to_json)

      suite_data.fetch('tests', []).each do |test|
        context = test["context"]
        toggle_name = test["toggleName"]
        expected_result = test["expectedResult"]

        result = unleash_engine.is_enabled?(toggle_name, context)

        expect(result).to eq(expected_result),
          "Failed test '#{test['description']}': expected #{expected_result}, got #{result}"
      end

      suite_data.fetch('variantTests', []).each do |test|
        context = test["context"]
        toggle_name = test["toggleName"]
        expected_result = JSON.parse(test["expectedResult"].to_json, symbolize_names: true)

        result = unleash_engine.get_variant(toggle_name, context)

        expect(result).to eq(expected_result),
          "Failed test '#{test['description']}': expected #{expected_result}, got #{result}"
      end
    end
  end
end