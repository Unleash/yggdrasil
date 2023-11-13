package io.getunleash.engine;

import com.sun.management.HotSpotDiagnosticMXBean;
import org.openjdk.jmh.annotations.*;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import javax.management.MBeanServer;
import java.io.File;
import java.lang.management.ManagementFactory;
import java.nio.file.Paths;
import java.io.IOException;
import java.util.Random;
import java.util.concurrent.atomic.AtomicLong;


@State(Scope.Benchmark)
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
            dumpHeap("pre-take-state", true);
            System.out.println("Loading features from " + featureFilePath);
            String toggles = loadFeaturesFromFile(featureFilePath);
            System.out.println("Taking state "+toggles);
            engine.takeState(toggles);
            dumpHeap("after-take-state", true);
        } catch (Exception e) {
            System.out.println("Failed to setup benchmarks");
            e.printStackTrace();
            System.exit(1);
        }
    }

    long counter = 0;
    @Benchmark
    @Fork(jvmArgsAppend =
{
  //"-XX:StartFlightRecording:settings=/home/gaston/.sdkman/candidates/java/20.0.2-oracle/lib/jfr/profile.jfc"
    "-Xmx32m",
    "-XX:+UseStringDeduplication",
    "-XX:StartFlightRecording:filename=myrecording.jfr,settings=profile"
})
    public void benchmarkFeatureToggle() {
        Context context = new Context();
        try {
            engine.isEnabled("Feature.A", context);
            counter ++;
            if (counter % (1500000+teardownCounter) == 0) {
                UnleashEngineBenchmark.dumpHeap("inprocess-"+counter, true);
            }
        } catch (Exception e) {
            System.out.println("Exception caught during benchmark, this is no longer a valid benchmark so early exiting");
            e.printStackTrace();
            System.exit(1);
        }
    }

    long teardownCounter = 0;
    @TearDown
    public void tearDown() throws Exception {
        teardownCounter++;
        System.out.println("After testing "+counter+" isEnabled calls");
        UnleashEngineBenchmark.dumpHeap("teardown-"+teardownCounter, true);
    }

    static Random random = new Random();
    static Integer randomFamily = Double.valueOf(random.nextInt(100)).intValue();
    static AtomicLong dumpCounter = new AtomicLong(0);
    public static void dumpHeap(String ref, boolean live) throws Exception {
        String filepath = "heapdump."+randomFamily+"."+dumpCounter.incrementAndGet()+"-"+ref+".hprof";
        // check if file exists, if so, increment counter
        while (new File(filepath).exists()) {
            filepath = "heapdump."+randomFamily+"."+dumpCounter.incrementAndGet()+"-"+ref+".hprof";
        }
        try {

            MBeanServer server = ManagementFactory.getPlatformMBeanServer();
            HotSpotDiagnosticMXBean mxBean = ManagementFactory.newPlatformMXBeanProxy(
                    server, "com.sun.management:type=HotSpotDiagnostic", HotSpotDiagnosticMXBean.class);
            mxBean.dumpHeap(filepath, live);
        } catch (IOException e) {
            System.out.println("Failed to dump heap to "+filepath);
            throw new Exception(e);
        }
    }
}