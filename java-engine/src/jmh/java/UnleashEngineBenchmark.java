package io.getunleash.engine;

import org.openjdk.jmh.annotations.*;
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
            System.out.println("Loading features from " + featureFilePath);
            String toggles = loadFeaturesFromFile(featureFilePath);
            System.out.println("Taking state "+toggles);
            engine.takeState(toggles);
        } catch (Exception e) {
            System.out.println("Failed to setup benchmarks");
            e.printStackTrace();
            System.exit(1);
        }
    }

    @Benchmark
    @Fork(jvmArgsAppend =
{
  //"-XX:StartFlightRecording:settings=/home/gaston/.sdkman/candidates/java/20.0.2-oracle/lib/jfr/profile.jfc"
  "-XX:StartFlightRecording:filename=myrecording.jfr,settings=profile"
})
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

    @TearDown
    public void tearDown() {
        System.out.println("Renaming recording");
       // new File("myrecording.jfr").renameTo(new File("myrecording"+ Instant.now() + ".jfr"));
    }
}