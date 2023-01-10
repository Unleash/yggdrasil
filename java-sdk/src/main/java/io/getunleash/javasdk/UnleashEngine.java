package io.getunleash.javasdk;

import com.google.gson.Gson;

public class UnleashEngine {

    static {
        // Todo: This needs be replaced for loadLibrary, which will rather search all
        // the
        // places the lib should be
        // An easy work around for dev would be to copy the compiled lib to the root of
        // this sdk on build
        System.load("/home/simon/dev/experiments/yggdrasil/target/release/libjava_glue.so");
    }

    private long pointer;

    native long createEngine();

    native void destroyEngine(long pointer);

    native boolean enabled(long pointer, String toggleName, String context);

    native void takeState(long pointer, String state);

    private Gson gson;

    public UnleashEngine() {
        this.pointer = createEngine();
        this.gson = new Gson();
    }

    public boolean isEnabled(String toggleName, Context context) {
        return enabled(this.pointer, toggleName, this.gson.toJson(context));
    }

    public void takeState(String unleashState) {
        takeState(this.pointer, unleashState);
    }

    @Override
    public void finalize() {
        destroyEngine(this.pointer);
    }
}
