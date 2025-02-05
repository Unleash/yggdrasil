STANDARD_STRATEGIES = [
  "default",
  "userWithId",
  "gradualRolloutUserId",
  "gradualRolloutSessionId",
  "gradualRolloutRandom",
  "flexibleRollout",
  "remoteAddress",
].freeze

class CustomStrategyHandler
  def initialize
    @custom_strategies_definitions = {}
    @custom_strategy_implementations = {}
  end

  def update_strategies(json_str)
    custom_strategies = {}
    parsed_json = JSON.parse(json_str)

    features = extract_features parsed_json

    features.each do |feature|
      toggle_name = feature["name"]
      strategies = feature["strategies"]

      custom_strategies_for_toggle = strategies.select do |strategy|
        !STANDARD_STRATEGIES.include?(strategy["name"])
      end

      unless custom_strategies_for_toggle.empty?
        custom_strategies[toggle_name] = custom_strategies_for_toggle
      end
    end

    @custom_strategies_definitions = custom_strategies
  end

  def register_custom_strategies(strategies)
    strategies.each do |strategy|
      if strategy.respond_to?(:name) && strategy.name.is_a?(String) &&
         strategy.respond_to?(:enabled?)
        @custom_strategy_implementations[strategy.name] = strategy
      else
        raise "Invalid strategy object. Must have a name method that returns a String and an enabled? method."
      end
    end
  end

  def evaluate_custom_strategies(toggle_name, context)
    results = {}

    @custom_strategies_definitions[toggle_name]&.each_with_index do |strategy, index|
      key = "customStrategy#{index + 1}"
      strategy_impl = @custom_strategy_implementations[strategy["name"]]
      result = strategy_impl&.enabled?(strategy["parameters"], context) || false
      results[key] = result
    end

    results
  end

  private

  def extract_features(data)
    return data["features"] if data.is_a?(Hash) && data.key?("features")

    if data.is_a?(Hash) && data.key?("events")
      features = []

      hydration_events = data["events"].select { |e| e["type"] == "hydration" }
      features.concat(hydration_events.flat_map { |e| e["features"] || [] })

      feature_updated_events = data["events"].select { |e| e["type"] == "feature-updated" }
      features.concat(feature_updated_events.map { |e| e["feature"] })

      return features
    end

    []
  end
end
