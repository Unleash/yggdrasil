package io.getunleash.engine;

import org.openjdk.jmh.annotations.Benchmark;
import org.openjdk.jmh.annotations.Setup;
import org.openjdk.jmh.annotations.State;
import org.openjdk.jmh.annotations.Scope;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.nio.file.Paths;
import java.io.IOException;


@State(Scope.Thread)
public class UnleashEngineBenchmark {

    private UnleashEngine engine;
    private final String featureFilePath = "../client-specification/specifications/01-simple-examples.json";

    private static String loadFeaturesFromFile(String filePath) throws IOException {
        ObjectMapper mapper = new ObjectMapper();
        JsonNode jsonNode = mapper.readTree(Paths.get(filePath).toFile());
        JsonNode state = jsonNode.get("state");
        return state.toString();
    }

    @Setup
    public void setUp() {
        engine = new UnleashEngine(new YggdrasilFFI("../target/release"));
        try {
            engine.takeState(loadFeaturesFromFile(featureFilePath));
        } catch (Exception e) {
            System.out.println("Failed to setup benchmarks");
            e.printStackTrace();
            System.exit(1);
        }
    }

    @Benchmark
    public void benchmarkFeatureToggle() {
        Context context = new Context();
        try {
            Boolean result = engine.isEnabled("Feature.A", context);
        } catch (Exception e) {
            System.out.println("Exception caught during benchmark, this is no longer a valid benchmark so early exiting");
            e.printStackTrace();
            System.exit(1);
        }
    }
}