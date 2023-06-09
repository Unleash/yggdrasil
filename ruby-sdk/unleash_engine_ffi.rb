require 'ffi'

module UnleashEngine
  extend FFI::Library

  class Payload < FFI::Struct
    layout :payload_type, :pointer,
           :value, :pointer
  end

  class VariantDef < FFI::Struct
    layout :name, :pointer,
           :payload, :pointer,
           :enabled, :bool
  end

  class FFIContext < FFI::Struct
    layout :user_id, :pointer,
           :session_id, :pointer,
           :environment, :pointer,
           :app_name, :pointer,
           :current_time, :pointer,
           :remote_address, :pointer,
           :properties_keys, :pointer,
           :properties_values, :pointer,
           :properties_len, :size_t,
           :toggle_name, :pointer
  end

  ffi_lib '../target/release/libyggdrasilffi.so'

  attach_function :engine_new, [], :pointer
  attach_function :engine_free, [:pointer], :void
  attach_function :engine_take_state, [:pointer, :string], :string
  attach_function :engine_is_enabled, [:pointer, :string, FFIContext.by_ref], :bool
  attach_function :engine_get_variant, [:pointer, :pointer, FFIContext.by_ref], :pointer
  attach_function :engine_free_variant_def, [:pointer], :void


  def self.new_engine_state
    engine_new
  end

  def self.free_engine_state(ptr)
    engine_free(ptr)
  end

  def self.take_state(ptr, toggles)
    metric_bucket_json = engine_take_state(ptr, toggles)
    JSON.parse(metric_bucket_json)
  end

  def self.get_variant(ptr, name, context)
    name_ptr = FFI::MemoryPointer.from_string(name)
    context_struct = FFIContext.new
    context_pointers = []

    begin
      [:user_id, :session_id, :environment, :app_name, :current_time, :remote_address].each do |key|
        if context[key]
          pointer = FFI::MemoryPointer.from_string(context[key])
          context_struct[key] = pointer
          context_pointers << pointer
        end
      end

      if context[:properties]
        keys = context[:properties].keys
        values = context[:properties].values
        context_struct[:properties_keys] = keys_pointer = FFI::MemoryPointer.new(:pointer, keys.size)
        context_struct[:properties_values] = values_pointer = FFI::MemoryPointer.new(:pointer, values.size)
        context_struct[:properties_len] = keys.size
        keys.each_with_index do |key, i|
          pointer = FFI::MemoryPointer.from_string(key)
          context_struct[:properties_keys][i].write_pointer(pointer)
          context_pointers << pointer
        end
        values.each_with_index do |value, i|
          pointer = FFI::MemoryPointer.from_string(value)
          context_struct[:properties_values][i].write_pointer(pointer)
          context_pointers << pointer
        end
        context_pointers << keys_pointer
        context_pointers << values_pointer
      else
        context_struct[:properties_keys] = nil
        context_struct[:properties_values] = nil
        context_struct[:properties_len] = 0
      end

      variant_def_ptr = engine_get_variant(ptr, name_ptr.read_string, context_struct)

      variant_def = VariantDef.new(variant_def_ptr)
      name = variant_def[:name].read_string
      enabled = variant_def[:enabled]
      payload = if variant_def[:payload].null?
        nil
      else
        payload_struct = FFIPayload.new(variant_def[:payload])
        payload_type = payload_struct[:payload_type].read_string
        value = payload_struct[:value].read_string
        { payload_type: payload_type, value: value }
      end

      UnleashEngine.engine_free_variant_def(variant_def_ptr)

      { name: name, payload: payload, enabled: enabled }
    ensure
      context_pointers.each(&:free)
    end
  end

  def self.is_enabled?(ptr, input, context)
    context_struct = FFIContext.new
    context_pointers = []

    begin
      [:user_id, :session_id, :environment, :app_name, :current_time, :remote_address].each do |key|
        if context[key]
          pointer = FFI::MemoryPointer.from_string(context[key])
          context_struct[key] = pointer
          context_pointers << pointer
        end
      end

      if context[:properties]
        keys = context[:properties].keys
        values = context[:properties].values
        context_struct[:properties_keys] = keys_pointer = FFI::MemoryPointer.new(:pointer, keys.size)
        context_struct[:properties_values] = values_pointer = FFI::MemoryPointer.new(:pointer, values.size)
        context_struct[:properties_len] = keys.size
        keys.each_with_index do |key, i|
          pointer = FFI::MemoryPointer.from_string(key)
          context_struct[:properties_keys][i].write_pointer(pointer)
          context_pointers << pointer
        end
        values.each_with_index do |value, i|
          pointer = FFI::MemoryPointer.from_string(value)
          context_struct[:properties_values][i].write_pointer(pointer)
          context_pointers << pointer
        end
        context_pointers << keys_pointer
        context_pointers << values_pointer
      else
        context_struct[:properties_keys] = nil
        context_struct[:properties_values] = nil
        context_struct[:properties_len] = 0
      end

      engine_is_enabled(ptr, input, context_struct)
    ensure
      context_pointers.each(&:free)
    end
  end
end
