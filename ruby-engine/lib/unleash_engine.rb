require 'ffi'
require 'json'
require 'custom_strategy'

TOGGLE_MISSING_RESPONSE = 'NotFound'.freeze
ERROR_RESPONSE = 'Error'.freeze
OK_RESPONSE = 'Ok'.freeze

def platform_specific_lib
  case RbConfig::CONFIG['host_os']
  when /darwin|mac os/
    'libyggdrasilffi.dylib'
  when /linux/
    'libyggdrasilffi.so'
  when /mswin|msys|mingw|cygwin|bccwin|wince|emc/
    'libyggdrasilffi.dll'
  else
    raise "unsupported platform #{RbConfig::CONFIG['host_os']}"
  end
end

class Variant
  attr_accessor :enabled, :name, :payload

  def initialize(attributes = {})
    self.enabled = attributes['enabled'] || false
    self.name = attributes['name'] || 'disabled'
    self.payload = attributes['payload']
  end

  def to_s
    "Variant: #{self.name} enabled: #{self.enabled} payload: #{self.payload}"
  end
end

class UnleashEngine
  extend FFI::Library
  ffi_lib File.expand_path(platform_specific_lib, __dir__)

  attach_function :new_engine, [], :pointer
  attach_function :free_engine, [:pointer], :void

  attach_function :take_state, %i[pointer string], :pointer
  attach_function :check_enabled, %i[pointer string string string], :pointer
  attach_function :check_variant, %i[pointer string string string], :pointer
  attach_function :get_metrics, [:pointer], :pointer
  attach_function :free_response, [:pointer], :void

  attach_function :count_toggle, %i[pointer string bool], :void
  attach_function :count_variant, %i[pointer string string], :void

  def initialize
    @engine = UnleashEngine.new_engine
    @custom_strategy_handler = CustomStrategyHandler.new
    ObjectSpace.define_finalizer(self, self.class.finalize(@engine))
  end

  def self.finalize(engine)
    proc { UnleashEngine.free_engine(engine) }
  end

  def take_state(toggles)
    @custom_strategy_handler.update_strategies(toggles)
    response_ptr = UnleashEngine.take_state(@engine, toggles)
    take_toggles_response = JSON.parse(response_ptr.read_string, symbolize_names: true)
    UnleashEngine.free_response(response_ptr)
  end

  def get_variant(name, context)
    context_json = (context || {}).to_json
    custom_strategy_results = @custom_strategy_handler.evaluate_custom_strategies(name, context).to_json

    variant_def_json_ptr = UnleashEngine.check_variant(@engine, name, context_json, custom_strategy_results)
    variant_def_json = variant_def_json_ptr.read_string
    UnleashEngine.free_response(variant_def_json_ptr)
    variant_response = JSON.parse(variant_def_json, symbolize_names: true)

    return nil if variant_response[:status_code] == TOGGLE_MISSING_RESPONSE
    return variant_response[:value] if variant_response[:status_code] == OK_RESPONSE
  end

  def enabled?(toggle_name, context)
    context_json = (context || {}).to_json
    custom_strategy_results = @custom_strategy_handler.evaluate_custom_strategies(toggle_name, context).to_json

    response_ptr = UnleashEngine.check_enabled(@engine, toggle_name, context_json, custom_strategy_results)
    response_json = response_ptr.read_string
    UnleashEngine.free_response(response_ptr)
    response = JSON.parse(response_json, symbolize_names: true)

    raise "Error: #{response[:error_message]}" if response[:status_code] == ERROR_RESPONSE
    return nil if response[:status_code] == TOGGLE_MISSING_RESPONSE

    return response[:value] == true
  end

  def count_toggle(toggle_name, enabled)
    response_ptr = UnleashEngine.count_toggle(@engine, toggle_name, enabled)
    UnleashEngine.free_response(response_ptr)
  end

  def count_variant(toggle_name, variant_name)
    response_ptr = UnleashEngine.count_variant(@engine, toggle_name, variant_name)
    UnleashEngine.free_response(response_ptr)
  end

  def get_metrics
    metrics_ptr = UnleashEngine.get_metrics(@engine)
    metrics = JSON.parse(metrics_ptr.read_string, symbolize_names: true)
    UnleashEngine.free_response(metrics_ptr)
    metrics[:value]
  end

  def register_custom_strategies(strategies)
    @custom_strategy_handler.register_custom_strategies(strategies)
  end
end
