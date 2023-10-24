class Strategy
    def initialize(name)
        @name = name
    end

    def enabled?(params, context)
        false
    end
end