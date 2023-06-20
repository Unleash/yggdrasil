require 'ffi'
require 'json'

class UnleashEngine
  extend FFI::Library

  lib_path = ENV['YGGDRASIL_LIB_PATH']
  raise 'YGGDRASIL_LIB_PATH env variable not set' if lib_path.nil?

  combined_path = File.join(lib_path, 'libyggdrasilffi.so')

  ffi_lib combined_path

  attach_function :engine_new, [], :pointer
  attach_function :engine_free, [:pointer], :void
  attach_function :engine_take_state, [:pointer, :string], :string
  attach_function :engine_is_enabled, [:pointer, :string, :string], :bool
  attach_function :engine_get_variant, [:pointer, :string, :string], :pointer
  attach_function :engine_free_variant_def, [:pointer], :void

  def initialize
    @engine_state = UnleashEngine.engine_new
    ObjectSpace.define_finalizer(self, self.class.finalize(@engine_state))
  end

  def self.finalize(engine_state)
    proc { UnleashEngine.engine_free(engine_state) }
  end

  def take_state(toggles)
    metric_bucket_json = UnleashEngine.engine_take_state(@engine_state, toggles)
    JSON.parse(metric_bucket_json)
  end

  def get_variant(name, context)
    context_json = context.to_json
    variant_def_json_ptr = UnleashEngine.engine_get_variant(@engine_state, name, context_json)

    variant_def_json = variant_def_json_ptr.read_string
    UnleashEngine.engine_free_variant_def(variant_def_json_ptr)

    JSON.parse(variant_def_json, symbolize_names: true)
  end

  def is_enabled?(toggle_name, context)
    context_json = context.to_json
    UnleashEngine.engine_is_enabled(@engine_state, toggle_name, context_json)
  end
end