require 'ffi'
require 'json'

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

  attach_function :engine_new, [], :pointer
  attach_function :engine_free, [:pointer], :void
  attach_function :engine_take_state, %i[pointer string], :string
  attach_function :engine_is_enabled, %i[pointer string string], :bool
  attach_function :engine_get_variant, %i[pointer string string], :pointer
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

    Variant.new(JSON.parse(variant_def_json))
  end

  def enabled?(toggle_name, context)
    context_json = context.to_json
    UnleashEngine.engine_is_enabled(@engine_state, toggle_name, context_json)
  end
end
