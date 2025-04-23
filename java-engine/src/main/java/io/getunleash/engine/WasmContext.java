package io.getunleash.engine;

import java.util.Map;
import java.util.HashMap;

public class WasmContext {
    public String userId;
    public String sessionId;
    public String environment;
    public String appName;
    public String currentTime;
    public String remoteAddress;
    public Map<String, String> properties = new HashMap<>();

    public String getUserId() {
        return userId;
    }

    public String getSessionId() {
        return sessionId;
    }

    public String getAppName() {
        return appName;
    }

    public String getCurrentTime() {
        return currentTime;
    }

    public String getEnvironment() {
        return environment;
    }

    public String getRemoteAddress() {
        return remoteAddress;
    }

    public Map<String, String> getProperties() {
        return properties;
    }
}
