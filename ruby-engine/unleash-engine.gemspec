Gem::Specification.new do |s|
    s.name        = 'unleash-engine'
    s.version     = '0.0.1'
    s.date        = '2023-06-28'
    s.summary     = 'Unleash engine for evaluating feature toggles'
    s.description = '...'
    s.authors     = ['Your Name']
    s.email       = 'you@example.com'
    s.files       = Dir.glob("{lib,spec}/**/*") + ["README.md"] + Dir["lib/**/*", "ext/libyggdrasilffi.so"]
    s.homepage    =
      'http://github.com/username/my_gem'
    s.license     = 'MIT'

    s.add_dependency "ffi", "~> 1.15.5"
    s.add_dependency "fiddle", "~> 1.0.6"
  end