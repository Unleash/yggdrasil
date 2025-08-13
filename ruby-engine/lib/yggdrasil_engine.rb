require 'ffi'
require 'json'
require 'custom_strategy'

TOGGLE_MISSING_RESPONSE = 'NotFound'.freeze
ERROR_RESPONSE = 'Error'.freeze
OK_RESPONSE = 'Ok'.freeze

def platform_specific_lib
  os = RbConfig::CONFIG['host_os']
  cpu = RbConfig::CONFIG['host_cpu']

  extension, prefix = case os
  when /darwin|mac os/
    ['dylib', 'lib']
  when /linux/
    ['so', 'lib']
  when /mswin|msys|mingw|cygwin|bccwin|wince|emc/
    ['dll', '']
  else
    raise "unsupported platform #{os}"
  end

  arch_suffix = case cpu
  when /x86_64|x64/
    'x86_64'
  when /arm|aarch64/
    'arm64'
  else
    raise "unsupported architecture #{cpu}"
  end

  lib_type_suffix = if os =~ /linux/
    musl = system("ldd /bin/sh | grep -q musl")
    musl ? "-musl" : ""
  else
    ""
  end

  "#{prefix}yggdrasilffi_#{arch_suffix}#{lib_type_suffix}.#{extension}"
end

def to_variant(raw_variant)
  payload = raw_variant[:payload] && raw_variant[:payload].transform_keys(&:to_s)
  {
    name: raw_variant[:name],
    enabled: raw_variant[:enabled],
    feature_enabled: raw_variant[:featureEnabled],
    payload: payload,
  }
end

class YggdrasilEngine
  extend FFI::Library
  ffi_lib File.expand_path(platform_specific_lib, __dir__)

  attach_function :new_engine, [], :pointer
  attach_function :free_engine, [:pointer], :void

  attach_function :take_state, %i[pointer string], :pointer
  attach_function :get_state, [:pointer], :pointer
  attach_function :check_enabled, %i[pointer string string string], :pointer
  attach_function :check_variant, %i[pointer string string string], :pointer
  attach_function :get_metrics, [:pointer], :pointer
  attach_function :free_response, [:pointer], :void

  attach_function :count_toggle, %i[pointer string bool], :pointer
  attach_function :count_variant, %i[pointer string string], :pointer

  attach_function :list_known_toggles, [:pointer], :pointer

  def initialize
    @engine = YggdrasilEngine.new_engine
    @custom_strategy_handler = CustomStrategyHandler.new
    ObjectSpace.define_finalizer(self, self.class.finalize(@engine))
  end

  def self.finalize(engine)
    proc { YggdrasilEngine.free_engine(engine) }
  end

  def take_state(toggles)
    @custom_strategy_handler.update_strategies(toggles)
    response_ptr = YggdrasilEngine.take_state(@engine, toggles)
    take_toggles_response = JSON.parse(response_ptr.read_string, symbolize_names: true)
    YggdrasilEngine.free_response(response_ptr)
  end

  def get_state
     response_ptr = YggdrasilEngine.get_state(@engine)
     state_json = response_ptr.read_string
     YggdrasilEngine.free_response(response_ptr)
     state_json
  end

  def get_variant(name, context)
    context_json = (context || {}).to_json
    custom_strategy_results = @custom_strategy_handler.evaluate_custom_strategies(name, context).to_json

    variant_def_json_ptr = YggdrasilEngine.check_variant(@engine, name, context_json, custom_strategy_results)
    variant_def_json = variant_def_json_ptr.read_string
    YggdrasilEngine.free_response(variant_def_json_ptr)
    variant_response = JSON.parse(variant_def_json, symbolize_names: true)

    return nil if variant_response[:status_code] == TOGGLE_MISSING_RESPONSE
    variant = variant_response[:value]

    return to_variant(variant) if variant_response[:status_code] == OK_RESPONSE
  end

  def enabled?(toggle_name, context)
    context_json = (context || {}).to_json
    custom_strategy_results = @custom_strategy_handler.evaluate_custom_strategies(toggle_name, context).to_json

    response_ptr = YggdrasilEngine.check_enabled(@engine, toggle_name, context_json, custom_strategy_results)
    response_json = response_ptr.read_string
    YggdrasilEngine.free_response(response_ptr)
    response = JSON.parse(response_json, symbolize_names: true)

    raise "Error: #{response[:error_message]}" if response[:status_code] == ERROR_RESPONSE
    return nil if response[:status_code] == TOGGLE_MISSING_RESPONSE

    return response[:value] == true
  end

  def count_toggle(toggle_name, enabled)
    response_ptr = YggdrasilEngine.count_toggle(@engine, toggle_name, enabled)
    YggdrasilEngine.free_response(response_ptr)
  end

  def count_variant(toggle_name, variant_name)
    response_ptr = YggdrasilEngine.count_variant(@engine, toggle_name, variant_name)
    YggdrasilEngine.free_response(response_ptr)
  end

  def get_metrics
    metrics_ptr = YggdrasilEngine.get_metrics(@engine)
    metrics = JSON.parse(metrics_ptr.read_string, symbolize_names: true)
    YggdrasilEngine.free_response(metrics_ptr)
    metrics[:value]
  end

  def list_known_toggles
    response_ptr = YggdrasilEngine.list_known_toggles(@engine)
    response_json = response_ptr.read_string
    YggdrasilEngine.free_response(response_ptr)
    JSON.parse(response_json, symbolize_names: true)
  end

  def register_custom_strategies(strategies)
    @custom_strategy_handler.register_custom_strategies(strategies)
  end
end
