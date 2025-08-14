package io.getunleash.engine;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import org.openjdk.jmh.annotations.Benchmark;
import org.openjdk.jmh.annotations.BenchmarkMode;
import org.openjdk.jmh.annotations.Fork;
import org.openjdk.jmh.annotations.Level;
import org.openjdk.jmh.annotations.Measurement;
import org.openjdk.jmh.annotations.Mode;
import org.openjdk.jmh.annotations.OutputTimeUnit;
import org.openjdk.jmh.annotations.Scope;
import org.openjdk.jmh.annotations.Setup;
import org.openjdk.jmh.annotations.State;
import org.openjdk.jmh.annotations.Warmup;
import org.openjdk.jmh.infra.Blackhole;

/**
 * Microbenchmarks for UnleashEngine public APIs. These are intentionally small and synthetic,
 * focusing on steady-state performance of core operations.
 */
@OutputTimeUnit(TimeUnit.NANOSECONDS)
@BenchmarkMode(Mode.AverageTime)
@Warmup(iterations = 1)
@Measurement(iterations = 1)
@Fork(1)
public class UnleashEngineBenchmark {

  @State(Scope.Benchmark)
  public static class EngineState {
    UnleashEngine engine;
    Context emptyCtx;
    Context user123Ctx;
    Context user999Ctx;

    String toggleDefault = "Feature.Default";
    String toggleUserWithId = "Feature.UserWithId";
    String toggleVariants = "Feature.Variants.A";
    String toggleCustom = "Feature.CustomStrategy";

    @Setup(Level.Trial)
    public void setup() throws Exception {
      // Optionally register a noop custom strategy to exercise evaluator overhead if desired.
      List<IStrategy> custom = new ArrayList<>();
      custom.add(new AlwaysTrueStrategy("alwaysTrue"));
      engine = new UnleashEngine(custom);

      emptyCtx = new Context();

      user123Ctx = new Context();
      user123Ctx.setUserId("123");

      user999Ctx = new Context();
      user999Ctx.setUserId("999");

      String features = buildFeaturesJson();
      engine.takeState(features);

      // Prime a tiny bit of state for metrics so the map isn't empty in getMetrics.
      engine.isEnabled(toggleDefault, emptyCtx);
      engine.isEnabled(toggleUserWithId, user123Ctx);
      engine.isEnabled(toggleCustom, emptyCtx);
      engine.getVariant(toggleVariants, emptyCtx);
    }

    private static String buildFeaturesJson() {
      // Single state JSON combining a few representative toggles
      // - Feature.Default: enabled with default strategy
      // - Feature.UserWithId: userIds strategy requiring userId=123
      // - Feature.CustomStrategy: enabled and uses custom strategy alwaysTrue
      // - Feature.Variants.A: one variant with weight 1
      return "{"
          + "\"version\":1,\"features\":["
          + "{\"name\":\"Feature.Default\",\"enabled\":true,\"strategies\":[{\"name\":\"default\"}]},"
          + "{\"name\":\"Feature.UserWithId\",\"enabled\":true,\"strategies\":[{\"name\":\"userWithId\",\"parameters\":{\"userIds\":\"123\"}}]},"
          + "{\"name\":\"Feature.CustomStrategy\",\"enabled\":true,\"strategies\":[{\"name\":\"alwaysTrue\"}]},"
          + "{\"name\":\"Feature.Variants.A\",\"enabled\":true,\"strategies\":[],\"variants\":[{\"name\":\"variant1\",\"weight\":1}]}"
          + "]}";
    }
  }

  /** A trivial custom strategy that always returns true. */
  static class AlwaysTrueStrategy implements IStrategy {
    private final String name;

    AlwaysTrueStrategy(String name) {
      this.name = name;
    }

    @Override
    public String getName() {
      return name;
    }

    @Override
    public boolean isEnabled(Map<String, String> parameters, Context context) {
      return true;
    }
  }

  @Benchmark
  public void isEnabled_default(EngineState s, Blackhole bh) throws Exception {
    bh.consume(s.engine.isEnabled(s.toggleDefault, s.emptyCtx));
  }

  @Benchmark
  public void isEnabled_userWithId_match(EngineState s, Blackhole bh) throws Exception {
    bh.consume(s.engine.isEnabled(s.toggleUserWithId, s.user123Ctx));
  }

  @Benchmark
  public void isEnabled_userWithId_noMatch(EngineState s, Blackhole bh) throws Exception {
    bh.consume(s.engine.isEnabled(s.toggleUserWithId, s.user999Ctx));
  }

  @Benchmark
  public void getVariant_present(EngineState s, Blackhole bh) throws Exception {
    bh.consume(s.engine.getVariant(s.toggleVariants, s.emptyCtx));
  }

  @Benchmark
  public void getMetrics_afterSomeTraffic(EngineState s, Blackhole bh) throws Exception {
    // Add a smidge of traffic per invocation, then read metrics
    s.engine.isEnabled(s.toggleDefault, s.emptyCtx);
    s.engine.getVariant(s.toggleVariants, s.emptyCtx);
    bh.consume(s.engine.getMetrics());
  }
}
