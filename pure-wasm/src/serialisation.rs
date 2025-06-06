use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
};

use flatbuffers::{FlatBufferBuilder, Follow, WIPOffset};
use unleash_types::client_metrics::MetricBucket;
use unleash_yggdrasil::{EvalWarning, ExtendedVariantDef, ToggleDefinition};

use crate::{
    messaging::messaging::{
        BuiltInStrategies, BuiltInStrategiesBuilder, CoreVersion, CoreVersionBuilder,
        FeatureDefBuilder, FeatureDefs, FeatureDefsBuilder, MetricsResponse,
        MetricsResponseBuilder, Response, ResponseBuilder, TakeStateResponse,
        TakeStateResponseBuilder, ToggleEntryBuilder, ToggleStatsBuilder, Variant, VariantBuilder,
        VariantEntryBuilder, VariantPayloadBuilder,
    },
};

thread_local! {
    static BUILDER: RefCell<FlatBufferBuilder<'static>> =
        RefCell::new(FlatBufferBuilder::with_capacity(128));
}

#[derive(Debug)]
pub enum WasmError {
    InvalidContext(String),
    InvalidState(String),
    InvalidPointer,
}

pub struct ResponseMessage<T> {
    pub message: Option<T>,
    pub impression_data: bool,
}

impl Display for WasmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmError::InvalidContext(msg) => write!(f, "Invalid context: {}", msg),
            WasmError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            WasmError::InvalidPointer => write!(f, "Invalid pointer encountered"),
        }
    }
}

pub extern "C" fn allocate(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

pub trait WasmMessage<TInput>: Follow<'static> + Sized {
    fn as_flat_buffer(builder: &mut FlatBufferBuilder<'static>, input: TInput) -> WIPOffset<Self>;

    fn build_response(input: TInput) -> u64 {
        let response_buffer = BUILDER.with(|cell| {
            let mut builder = cell.borrow_mut();
            builder.reset();

            let offset = Self::as_flat_buffer(&mut builder, input);

            builder.finish(offset, None);
            builder.finished_data().to_vec()
        });

        let result_len = response_buffer.len();
        let result_ptr = allocate(result_len);

        unsafe { std::ptr::copy_nonoverlapping(response_buffer.as_ptr(), result_ptr, result_len) };

        ((result_len as u64) << 32) | (result_ptr as u64)
    }
}

impl WasmMessage<Result<ResponseMessage<bool>, WasmError>> for Response<'static> {
    fn as_flat_buffer(
        builder: &mut FlatBufferBuilder<'static>,
        from: Result<ResponseMessage<bool>, WasmError>,
    ) -> WIPOffset<Response<'static>> {
        match from {
            Ok(response) => match response.message {
                Some(flag) => {
                    let mut response_builder = ResponseBuilder::new(builder);
                    response_builder.add_impression_data(response.impression_data);
                    response_builder.add_enabled(flag);
                    response_builder.add_has_enabled(true);
                    response_builder.finish()
                }
                None => {
                    let mut response_builder = ResponseBuilder::new(builder);
                    response_builder.add_has_enabled(false);
                    response_builder.finish()
                }
            },
            Err(err) => {
                let error_offset = builder.create_string(&err.to_string());
                let mut response_builder = ResponseBuilder::new(builder);
                response_builder.add_has_enabled(false);
                response_builder.add_error(error_offset);
                response_builder.finish()
            }
        }
    }
}

impl WasmMessage<Result<ResponseMessage<ExtendedVariantDef>, WasmError>> for Variant<'static> {
    fn as_flat_buffer(
        builder: &mut FlatBufferBuilder<'static>,
        from: Result<ResponseMessage<ExtendedVariantDef>, WasmError>,
    ) -> WIPOffset<Self> {
        match from {
            Ok(response) => match response.message {
                Some(variant) => {
                    let payload_offset = variant.payload.as_ref().map(|payload| {
                        let payload_type_offset = builder.create_string(&payload.payload_type);
                        let value_offset = builder.create_string(&payload.value);

                        let mut variant_payload = VariantPayloadBuilder::new(builder);
                        variant_payload.add_payload_type(payload_type_offset);
                        variant_payload.add_value(value_offset);

                        variant_payload.finish()
                    });

                    let variant_name_offset = builder.create_string(&variant.name);

                    let mut variant_builder = VariantBuilder::new(builder);
                    variant_builder.add_feature_enabled(variant.feature_enabled);
                    variant_builder.add_impression_data(response.impression_data);
                    variant_builder.add_enabled(variant.enabled);
                    variant_builder.add_name(variant_name_offset);
                    if let Some(payload_offset) = payload_offset {
                        variant_builder.add_payload(payload_offset);
                    }

                    variant_builder.finish()
                }
                None => {
                    let resp_builder = VariantBuilder::new(builder);
                    resp_builder.finish()
                }
            },
            Err(err) => {
                let error_offset = builder.create_string(&err.to_string());
                let mut response_builder = VariantBuilder::new(builder);
                response_builder.add_error(error_offset);
                response_builder.finish()
            }
        }
    }
}

impl WasmMessage<Option<MetricBucket>> for MetricsResponse<'static> {
    fn as_flat_buffer(
        builder: &mut FlatBufferBuilder<'static>,
        metrics: Option<MetricBucket>,
    ) -> WIPOffset<Self> {
        if let Some(metrics) = metrics {
            let items: Vec<_> = metrics
                .toggles
                .iter()
                .map(|(toggle_key, stats)| {
                    let variant_items: Vec<_> = stats
                        .variants
                        .iter()
                        .map(|(variant_key, count)| {
                            let variant_key = builder.create_string(variant_key);
                            let mut variant_builder = VariantEntryBuilder::new(builder);
                            variant_builder.add_key(variant_key);
                            variant_builder.add_value(*count);
                            variant_builder.finish()
                        })
                        .collect();
                    let variant_vector = builder.create_vector(&variant_items);

                    let toggle_key = builder.create_string(toggle_key);
                    let mut toggle_builder = ToggleStatsBuilder::new(builder);
                    toggle_builder.add_no(stats.no);
                    toggle_builder.add_yes(stats.yes);
                    toggle_builder.add_variants(variant_vector);
                    let toggle_value = toggle_builder.finish();
                    let mut toggle_entry_builder = ToggleEntryBuilder::new(builder);
                    toggle_entry_builder.add_value(toggle_value);
                    toggle_entry_builder.add_key(toggle_key);
                    toggle_entry_builder.finish()
                })
                .collect();
            let toggle_vector = builder.create_vector(&items);
            let mut resp_builder = MetricsResponseBuilder::new(builder);
            resp_builder.add_start(metrics.start.timestamp_millis());
            resp_builder.add_stop(metrics.stop.timestamp_millis());
            resp_builder.add_toggles(toggle_vector);
            resp_builder.finish()
        } else {
            let resp_builder = MetricsResponseBuilder::new(builder);
            resp_builder.finish()
        }
    }
}

impl WasmMessage<Vec<ToggleDefinition>> for FeatureDefs<'static> {
    fn as_flat_buffer(
        builder: &mut FlatBufferBuilder<'static>,
        known_toggles: Vec<ToggleDefinition>,
    ) -> WIPOffset<Self> {
        let items: Vec<_> = known_toggles
            .iter()
            .map(|toggle| {
                let toggle_name_offset = builder.create_string(&toggle.name);
                let project_offset = builder.create_string(&toggle.project);
                let feature_type_offset = toggle
                    .feature_type
                    .as_ref()
                    .map(|f| builder.create_string(f));

                let mut feature_def_builder = FeatureDefBuilder::new(builder);

                feature_def_builder.add_name(toggle_name_offset);
                feature_def_builder.add_project(project_offset);
                feature_def_builder.add_enabled(toggle.enabled);
                if feature_type_offset.is_some() {
                    feature_def_builder.add_type_(feature_type_offset.unwrap());
                }
                feature_def_builder.finish()
            })
            .collect();

        let toggle_vector = builder.create_vector(&items);

        let mut resp_builder = FeatureDefsBuilder::new(builder);
        resp_builder.add_items(toggle_vector);
        resp_builder.finish()
    }
}

impl WasmMessage<&str> for CoreVersion<'static> {
    fn as_flat_buffer(builder: &mut FlatBufferBuilder<'static>, version: &str) -> WIPOffset<Self> {
        let version_offset = builder.create_string(version);
        let mut resp_builder = CoreVersionBuilder::new(builder);
        resp_builder.add_version(version_offset);
        resp_builder.finish()
    }
}

impl WasmMessage<[&'static str; 8]> for BuiltInStrategies<'static> {
    fn as_flat_buffer(
        builder: &mut FlatBufferBuilder<'static>,
        strategies: [&'static str; 8],
    ) -> WIPOffset<Self> {
        let items: Vec<_> = strategies
            .iter()
            .map(|strategy| builder.create_string(strategy))
            .collect();

        let strategy_vector = builder.create_vector(&items);

        let mut resp_builder = BuiltInStrategiesBuilder::new(builder);
        resp_builder.add_values(strategy_vector);
        resp_builder.finish()
    }
}

impl WasmMessage<Result<Option<Vec<EvalWarning>>, WasmError>> for TakeStateResponse<'static> {
    fn as_flat_buffer(
        builder: &mut FlatBufferBuilder<'static>,
        response: Result<Option<Vec<EvalWarning>>, WasmError>,
    ) -> WIPOffset<Self> {
        match response {
            Ok(Some(state)) => {
                let warnings: Vec<_> = state
                    .iter()
                    .map(|warning| {
                        builder
                            .create_string(&format!("{}:{}", warning.toggle_name, warning.message))
                    })
                    .collect();

                let warning_vector = builder.create_vector(&warnings);
                let mut resp_builder = TakeStateResponseBuilder::new(builder);
                resp_builder.add_warnings(warning_vector);
                resp_builder.finish()
            }
            Ok(None) => {
                let resp_builder = TakeStateResponseBuilder::new(builder);
                resp_builder.finish()
            }
            Err(err) => {
                let error_offset = builder.create_string(&err.to_string());
                let mut resp_builder = TakeStateResponseBuilder::new(builder);
                resp_builder.add_error(error_offset);
                resp_builder.finish()
            }
        }
    }
}