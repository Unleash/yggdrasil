package io.getunleash.engine;

import java.util.Map;

public class WasmContext {
    public String userId;
    public String sessionId;
    public String appName;
    public String instanceId;
    public String environment;
    public String remoteAddress;
    public Map<String, String> properties;

    public Map<String, String> getProperties() {
        return properties;
    }
}
