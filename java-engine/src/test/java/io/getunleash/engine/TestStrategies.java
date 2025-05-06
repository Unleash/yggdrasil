package io.getunleash.engine;

import java.util.Map;

class TestStrategies {
  static IStrategy alwaysTrue(String name) {
    return new IStrategy() {
      @Override
      public String getName() {
        return name;
      }

      @Override
      public boolean isEnabled(Map<String, String> parameters, Context context) {
        return true;
      }
    };
  }

  static IStrategy alwaysFails(String name) {
    return new IStrategy() {
      @Override
      public String getName() {
        return name;
      }

      @Override
      public boolean isEnabled(Map<String, String> parameters, Context context) {
        throw new RuntimeException("This strategy always fails");
      }
    };
  }

  static IStrategy onlyTrueIfAllParametersInContext(String name) {
    return new IStrategy() {
      @Override
      public String getName() {
        return name;
      }

      @Override
      public boolean isEnabled(Map<String, String> parameters, Context context) {
        for (String parameter : parameters.keySet()) {
          Map<String, String> properties = context.getProperties();

          if (properties == null
              || !(properties.containsKey(parameter)
                  && properties.get(parameter).equals(parameters.get(parameter)))) {
            return false;
          }
        }
        return true;
      }
    };
  }
}
