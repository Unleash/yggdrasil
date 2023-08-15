require 'rspec'
require 'json'
require_relative '../lib/unleash_engine'

index_file_path = '../client-specification/specifications/index.json'
test_suites = JSON.parse(File.read(index_file_path))

#tests to cover checking a toggle with a missing name

RSpec.describe UnleashEngine do
  let(:unleash_engine) { UnleashEngine.new }

  describe '#checking a toggle' do
    it 'that does not exist should yield a not found' do
      is_enabled = unleash_engine.enabled?('missing-toggle', {})
      expect(is_enabled).to be_nil
    end
  end

  describe '#metrics' do
    it 'should clear metrics when get_metrics is called' do
      feature_name = 'Feature.A'

      suite_path = File.join('../client-specification/specifications', '01-simple-examples.json')
      suite_data = JSON.parse(File.read(suite_path))

      unleash_engine.take_state(suite_data['state'].to_json)

      unleash_engine.count_toggle(feature_name, true)
      unleash_engine.count_toggle(feature_name, false)

      metrics =  unleash_engine.get_metrics() # This should clear the metrics buffer

      metric = metrics[:toggles][feature_name.to_sym]
      expect(metric[:yes]).to eq(1)
      expect(metric[:no]).to eq(1)

      metrics =  unleash_engine.get_metrics()
      expect(metrics).to be_nil
    end

    it 'should increment toggle count when it exists' do
      toggle_name = 'Feature.A'

      suite_path = File.join('../client-specification/specifications', '01-simple-examples.json')
      suite_data = JSON.parse(File.read(suite_path))

      unleash_engine.take_state(suite_data['state'].to_json)

      unleash_engine.count_toggle(toggle_name, true)
      unleash_engine.count_toggle(toggle_name, false)

      metrics =  unleash_engine.get_metrics()
      metric = metrics[:toggles][toggle_name.to_sym]

      expect(metric[:yes]).to eq(1)
      expect(metric[:no]).to eq(1)
    end

    it 'should increment toggle count when the toggle does not exist' do
      toggle_name = 'Feature.X'

      unleash_engine.count_toggle(toggle_name, true)
      unleash_engine.count_toggle(toggle_name, false)

      metrics =  unleash_engine.get_metrics()
      metric = metrics[:toggles][toggle_name.to_sym]

      expect(metric[:yes]).to eq(1)
      expect(metric[:no]).to eq(1)
    end

    it 'should increment variant' do
      toggle_name = 'Feature.Q'

      suite_path = File.join('../client-specification/specifications', '01-simple-examples.json')
      suite_data = JSON.parse(File.read(suite_path))

      unleash_engine.take_state(suite_data['state'].to_json)

      unleash_engine.count_variant(toggle_name, 'disabled')

      metrics =  unleash_engine.get_metrics()
      metric = metrics[:toggles][toggle_name.to_sym]

      expect(metric[:variants][:disabled]).to eq(1)
    end
  end
end

RSpec.describe 'Client Specification' do
  let(:unleash_engine) { UnleashEngine.new }

  test_suites.each do |suite|
    suite_path = File.join('../client-specification/specifications', suite)
    suite_data = JSON.parse(File.read(suite_path))

    describe "Suite '#{suite}'" do
      before(:each) do
        unleash_engine.take_state(suite_data['state'].to_json)
      end

      suite_data.fetch('tests', []).each do |test|
        describe "Test '#{test['description']}'" do
          let(:context) { test['context'] }
          let(:toggle_name) { test['toggleName'] }
          let(:expected_result) { test['expectedResult'] }

          it 'returns correct result for `is_enabled?` method' do
            result = unleash_engine.enabled?(toggle_name, context) || false

            expect(result).to eq(expected_result),
                              "Failed test '#{test['description']}': expected #{expected_result}, got #{result}"
          end
        end
      end

      suite_data.fetch('variantTests', []).each do |test|
        next unless test['expectedResult']

        describe "Variant Test '#{test['description']}'" do
          let(:context) { test['context'] }
          let(:toggle_name) { test['toggleName'] }
          let(:expected_result) { Variant.new(test['expectedResult']) }

          it 'returns correct result for `get_variant` method' do
            result = unleash_engine.get_variant(toggle_name, context).variant

            expect(result.name).to eq(expected_result.name),
                                   "Failed test '#{test['description']}': expected #{expected_result.name}, got #{result.name}"
            expect(result.enabled).to eq(expected_result.enabled),
                                      "Failed test '#{test['description']}': expected #{expected_result.enabled}, got #{result.enabled}"
            expect(result.payload).to eq(expected_result.payload),
                                      "Failed test '#{test['description']}': expected #{expected_result.payload}, got #{result.payload}"
          end
        end
      end
    end
  end
end
