Gem::Specification.new do |s|

  target_platform = -> { ENV['YGG_BUILD_PLATFORM'] || Gem::Platform::CURRENT }

  s.name = 'yggdrasil-engine'
  s.version = '0.0.7'
  s.date = '2023-06-28'
  s.summary = 'Unleash engine for evaluating feature toggles'
  s.description = '...'
  s.authors = ['Unleash']
  s.email = 'liquidwicked64@gmail.com'
  s.files = Dir.glob("{lib,spec}/**/*") + ["README.md"] + Dir["lib/**/*"]
  s.homepage = 'http://github.com/username/my_gem'
  s.license = 'MIT'
  s.add_dependency "ffi", "~> 1.16.3"
  s.platform = target_platform.call
end
