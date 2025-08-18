package io.getunleash.engine;

import com.dylibso.chicory.runtime.ByteBufferMemory;
import com.dylibso.chicory.runtime.ExportFunction;
import com.dylibso.chicory.runtime.HostFunction;
import com.dylibso.chicory.runtime.ImportValues;
import com.dylibso.chicory.runtime.Instance;
import com.dylibso.chicory.wasm.types.FunctionType;
import com.dylibso.chicory.wasm.types.ValType;
import io.getunleash.wasm.Yggdrasil;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.security.SecureRandom;
import java.time.ZonedDateTime;
import java.util.List;
import java.util.function.Function;
import messaging.BuiltInStrategies;
import messaging.CoreVersion;
import messaging.FeatureDefs;
import messaging.MetricsResponse;
import messaging.Response;
import messaging.Variant;

interface NativeInterface {
  int newEngine(long timestamp);

  void freeEngine(int ptr);

  void takeState(int ptr, byte[] messageBytes);

  String getState(int ptr);

  Response checkEnabled(int enginePtr, byte[] contextBytes);

  Variant checkVariant(int enginePtr, byte[] contextBytes);

  MetricsResponse getMetrics(int enginePtr, ZonedDateTime timestamp);

  int getLogBufferPtr();

  FeatureDefs listKnownToggles(int enginePtr);
}

public class WasmInterface implements NativeInterface {
  private static final Instance instance;
  private static final ExportFunction newEngine;
  private static final ExportFunction freeEngine;
  private static final ExportFunction alloc;
  private static final ExportFunction dealloc;
  private static final ExportFunction takeState;
  private static final ExportFunction getState;
  private static final ExportFunction checkEnabled;
  private static final ExportFunction checkVariant;
  private static final ExportFunction getMetrics;
  private static final ExportFunction deallocResponseBuffer;
  private static final ExportFunction getLogBufferPtr;
  private static final ExportFunction listKnownToggles;
  private static final ExportFunction getCoreVersion;
  private static final ExportFunction getBuiltInStrategies;
  private static final Object engineLock = new Object();

  static {
    FunctionType fillRandomFunctionType =
        FunctionType.of(List.of(ValType.I32, ValType.I32), List.of(ValType.I32));

    ImportValues imports =
        ImportValues.builder()
            .addFunction(
                new HostFunction(
                    "env",
                    "fill_random",
                    fillRandomFunctionType,
                    (Instance instance, long... args) -> {
                      int ptr = (int) args[0];
                      int len = (int) args[1];

                      if (len <= 0 || ptr < 0) return new long[] {1};

                      byte[] randomBytes = new byte[len];
                      new SecureRandom().nextBytes(randomBytes);

                      instance.memory().write(ptr, randomBytes);

                      return new long[] {0};
                    }))
            .build();

    instance =
        Instance.builder(Yggdrasil.load())
            .withMachineFactory(Yggdrasil::create)
            .withImportValues(imports)
            .withMemoryFactory(ByteBufferMemory::new)
            .build();

    alloc = instance.export("local_alloc");
    dealloc = instance.export("local_dealloc");
    takeState = instance.export("take_state");
    getState = instance.export("get_state");
    checkEnabled = instance.export("check_enabled");
    checkVariant = instance.export("check_variant");
    getMetrics = instance.export("get_metrics");
    deallocResponseBuffer = instance.export("dealloc_response_buffer");
    getLogBufferPtr = instance.export("get_log_buffer_ptr");
    listKnownToggles = instance.export("list_known_toggles");
    getCoreVersion = instance.export("get_core_version");
    getBuiltInStrategies = instance.export("get_built_in_strategies");
    newEngine = instance.export("new_engine");
    freeEngine = instance.export("free_engine");
  }

  @Override
  public int newEngine(long timestamp) {
    synchronized (engineLock) {
      return (int) newEngine.apply(timestamp)[0];
    }
  }

  @Override
  public void freeEngine(int ptr) {
    synchronized (engineLock) {
      freeEngine.apply(ptr);
    }
  }

  @Override
  public void takeState(int enginePtr, byte[] messageBytes) {
    synchronized (engineLock) {
      int len = messageBytes.length;
      int ptr = (int) alloc.apply(len)[0];

      instance.memory().write(ptr, messageBytes);
      takeState.apply(enginePtr, ptr, len);

      dealloc.apply(ptr, len);
    }
  }

  @Override
  public String getState(int enginePtr) {
    synchronized (engineLock) {
      int ptr = (int) getState.apply(enginePtr)[0];
      if (ptr == 0) {
        // Shouldn't happen anymore, but return empty state as fallback
        return "{\"version\":2,\"features\":[]}";
      }
      return instance.memory().readCString(ptr);
    }
  }

  @Override
  public Response checkEnabled(int enginePtr, byte[] contextBytes) {
    synchronized (engineLock) {
      int contextPtr = (int) alloc.apply(contextBytes.length)[0];
      instance.memory().write(contextPtr, contextBytes);

      long response = checkEnabled.apply(enginePtr, contextPtr, contextBytes.length)[0];
      Response responseObj = derefWasmPointer(response, Response::getRootAsResponse);
      dealloc.apply(contextPtr, contextBytes.length);
      return responseObj;
    }
  }

  @Override
  public Variant checkVariant(int enginePtr, byte[] contextBytes) {
    synchronized (engineLock) {
      int contextPtr = (int) alloc.apply(contextBytes.length)[0];
      instance.memory().write(contextPtr, contextBytes);

      long response = checkVariant.apply(enginePtr, contextPtr, contextBytes.length)[0];
      Variant variant = derefWasmPointer(response, Variant::getRootAsVariant);
      dealloc.apply(contextPtr, contextBytes.length);
      return variant;
    }
  }

  @Override
  public MetricsResponse getMetrics(int enginePtr, ZonedDateTime timestamp) {
    synchronized (engineLock) {
      long packed = getMetrics.apply(enginePtr, timestamp.toInstant().toEpochMilli())[0];
      return derefWasmPointer(packed, MetricsResponse::getRootAsMetricsResponse);
    }
  }

  @Override
  public int getLogBufferPtr() {
    long[] result = getLogBufferPtr.apply();
    return (int) result[0];
  }

  @Override
  public FeatureDefs listKnownToggles(int enginePtr) {
    synchronized (engineLock) {
      long packed = listKnownToggles.apply(enginePtr)[0];
      return derefWasmPointer(packed, FeatureDefs::getRootAsFeatureDefs);
    }
  }

  public static String getCoreVersion() {
    synchronized (engineLock) {
      long packed = WasmInterface.getCoreVersion.apply()[0];
      CoreVersion version = derefWasmPointer(packed, CoreVersion::getRootAsCoreVersion);
      return version.version();
    }
  }

  public static BuiltInStrategies getBuiltInStrategies() {
    synchronized (engineLock) {
      long packed = WasmInterface.getBuiltInStrategies.apply()[0];
      return derefWasmPointer(packed, BuiltInStrategies::getRootAsBuiltInStrategies);
    }
  }

  private static <T> T derefWasmPointer(long packed, Function<ByteBuffer, T> decoder) {
    // Warning: This is not thread safe, it should be called as part of a
    // synchronized block
    // This is not sane. To receive the response we need to two things:
    // 1) a pointer
    // 2) a length so we can read the pointer value to the end but not beyond
    //
    // However, we don't have a way to pass complex objects back to the host
    // function. We can use a pre-allocated shared buffer but we would need to have
    // that buffer size appropriately tuned for real workloads.
    // Which requires a bunch of experimentation sooooo...
    // instead we hack this. We're using 32 bit WASM here, which means
    // pointers are 32 bits and we need a second 32 bit number to represent the
    // length of the buffer. We can pass a 64 bit number across the WASM boundary,
    // which is really two 32 bit numbers wearing a silly hat

    int ptr = (int) (packed & 0xFFFFFFFFL);
    int len = (int) (packed >>> 32);

    byte[] bytes = instance.memory().readBytes(ptr, len);

    ByteBuffer buf = ByteBuffer.wrap(bytes);
    buf.order(ByteOrder.LITTLE_ENDIAN);

    T response = decoder.apply(buf);
    deallocResponseBuffer.apply(ptr, len);
    return response;
  }

  @FunctionalInterface
  public interface TriFunction<A, B, C, R> {
    R apply(A a, B b, C c);
  }

  private void readLog() {
    int start = getLogBufferPtr();
    String msg = instance.memory().readCString(start);
    if (msg != null && !msg.isEmpty()) {
      System.out.println("DebugLog: " + msg);
    }
  }
}
