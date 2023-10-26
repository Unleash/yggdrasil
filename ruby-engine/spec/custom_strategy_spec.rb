require_relative '../lib/custom_strategy'

RSpec.describe "custom strategies" do

  let(:raw_state) do
    {
      "version": 1,
      "features": [
        {
          "name": "Feature.A",
          "strategies": [
            {
              "name": "default",
              "parameters": {}
            },
            {
              "name": "custom",
              "parameters": {
                "gerkhins": "yes"
              }
            },
            {
              "name": "some-other-custom",
              "parameters": {
                "gerkhins": "yes"
              }
            },
          ]
        }
      ]
    }
  end

  let(:handler) { CustomStrategyHandler.new }

  before do
    handler.update_strategies(raw_state.to_json)
  end

  describe 'computing custom strategies' do
    it 'respects the logic contained in the enabled function' do

      class TestStrategy
        attr_reader :name

        def initialize(name)
          @name = name
        end

        def enabled?(params, context)
          params["gerkhins"] == "yes"
        end
      end

      handler.register_custom_strategies([TestStrategy.new("custom")])
      strategy_results = handler.evaluate_custom_strategies("Feature.A", {})
      expect(strategy_results.length).to eq(2)
      expect(strategy_results["customStrategy1"]).to eq(true)
      expect(strategy_results["customStrategy2"]).to eq(false)
    end

    it 'creates a strategy result for every custom strategy thats implemented and defined' do

      class TestStrategy
        attr_reader :name

        def initialize(name)
          @name = name
        end

        def enabled?(params, context)
          params["gerkhins"] == "yes"
        end
      end

      handler.register_custom_strategies([TestStrategy.new("custom"), TestStrategy.new("some-other-custom")])
      strategy_results = handler.evaluate_custom_strategies("Feature.A", {})
      expect(strategy_results.length).to eq(2)
      expect(strategy_results["customStrategy1"]).to eq(true)
      expect(strategy_results["customStrategy2"]).to eq(true)
    end

    it 'returns false for missing implementations' do
      handler.register_custom_strategies([])
      strategy_results = handler.evaluate_custom_strategies("Feature.A", {})
      expect(strategy_results.length).to eq(2)
      expect(strategy_results["customStrategy1"]).to eq(false)
      expect(strategy_results["customStrategy1"]).to eq(false)
    end
  end
end
