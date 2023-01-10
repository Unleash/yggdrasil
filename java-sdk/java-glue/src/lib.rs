use jni::{
    objects::{JObject, JString},
    sys::{jboolean, jlong},
    JNIEnv,
};

use sdk_core::{state::InnerContext, EngineState};
use unleash_types::client_features::ClientFeatures;

#[no_mangle]
pub extern "system" fn Java_io_getunleash_javasdk_UnleashEngine_createEngine(
    _env: JNIEnv,
    _obj: JObject,
) -> jlong {
    Box::into_raw(Box::new(EngineState::new())) as jlong
}

#[no_mangle]
pub extern "system" fn Java_io_getunleash_javasdk_UnleashEngine_takeState(
    env: JNIEnv,
    _obj: JObject,
    ptr: jlong,
    state: JString,
) {
    let toggle_state: String = env
        .get_string(state)
        .expect("Couldn't get java string!")
        .into();

    let state: ClientFeatures =
        serde_json::from_str(&toggle_state).expect("Failed to parse unleash state");

    unsafe {
        let engine = &mut *(ptr as *mut EngineState);
        engine.take_state(state);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_getunleash_javasdk_UnleashEngine_destroyEngine(
    _env: JNIEnv,
    _obj: JObject,
    ptr: jlong,
) {
    unsafe { Box::from_raw(ptr as *mut EngineState) };
}

#[no_mangle]
pub extern "system" fn Java_io_getunleash_javasdk_UnleashEngine_enabled(
    env: JNIEnv,
    _obj: JObject,
    ptr: jlong,
    toggle_name: JString,
    context: JString,
) -> jboolean {
    let toggle_name: String = env
        .get_string(toggle_name)
        .expect("Couldn't get the toggle name from Java")
        .into();

    let context = env.get_string(context).ok().map(|context| {
        serde_json::from_str::<InnerContext>(
            context.to_str().expect("Could not get context from Java"),
        )
        .expect("Cannot deserialize context")
    });

    unsafe {
        let engine = &*(ptr as *mut EngineState);

        engine.is_enabled(toggle_name, &context.expect("This should work but doesn't")) as jboolean
    }
}
