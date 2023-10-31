require 'objspace'
require 'json'
require_relative '../lib/unleash_engine'
require 'get_process_mem'

unleash_engine = UnleashEngine.new

def run_memory_check(lambda, method_name_being_tested)
  puts "#{method_name_being_tested} Warming up"

  ## This should give us a warmup baseline to allow the runtime to assgin whatever it needs
  for i in 0..1000000
    lambda.call
  end
  GC.start

  ## Now we poke the memory - only memory we need for this call should be assigned now
  ## everything else should be done in the previous warm up
  pre_check_memory = GetProcessMem.new.mb
  for i in 0..1000000
    lambda.call
  end

  GC.start
  post_check_memory = GetProcessMem.new.mb

  puts "#{method_name_being_tested} Memory diff during enabled check: #{post_check_memory} #{pre_check_memory} MB"
  diff = post_check_memory - pre_check_memory
  if diff < 0.5
    puts "#{method_name_being_tested} Memory diff is within half a MB, this is an expected range"
  else
    STDERR.puts "WARNING: LIKELY MEMORY LEAK DETECTED. THIS REQUIRES HUMAN ATTENTION"
    STDERR.puts "#{method_name_being_tested} Memory diff is not within the expected half MB range, this likely indicates a leak within the engine"
  end

  puts "---"
end

suite_path = File.join('../client-specification/specifications', '01-simple-examples.json')
suite_data = JSON.parse(File.read(suite_path))
json_client_features = suite_data['state'].to_json

is_enabled = lambda { unleash_engine.enabled?('Feature.A', {}) }
get_variant = lambda { unleash_engine.get_variant('Feature.A', {}) }
get_metrics = lambda { unleash_engine.get_metrics() }
take_state = lambda { unleash_engine.take_state(json_client_features) }

run_memory_check(is_enabled, "IsEnabled:")
run_memory_check(get_variant, "GetVariant:")
run_memory_check(get_metrics, "GetMetrics:")
run_memory_check(take_state, "TakeState:")
