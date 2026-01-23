use std::{borrow::Cow, collections::HashMap};
use unleash_types::client_features::Context;

pub type PropertiesCow<'a> = HashMap<Cow<'a, str>, Cow<'a, str>>;
pub type ExternalResultsCow<'a> = HashMap<Cow<'a, str>, bool>;

#[derive(Copy, Clone)]
pub enum PropertiesRef<'a> {
    Strings(&'a HashMap<String, String>),
    Cows(&'a PropertiesCow<'a>),
}

#[derive(Copy, Clone)]
pub enum ExternalResultsRef<'a> {
    Strings(&'a HashMap<String, bool>),
    Cows(&'a ExternalResultsCow<'a>),
}

impl<'a> PropertiesRef<'a> {
    pub fn get(&self, key: &str) -> Option<&'a str> {
        match self {
            PropertiesRef::Strings(m) => m.get(key).map(|v| v.as_str()),
            PropertiesRef::Cows(m) => m.get(key).map(|v| v.as_ref()),
        }
    }
}

impl<'a> ExternalResultsRef<'a> {
    pub fn get(&self, key: &str) -> Option<bool> {
        match self {
            ExternalResultsRef::Strings(m) => m.get(key).copied(),
            ExternalResultsRef::Cows(m) => m.get(key).copied(),
        }
    }
}

pub struct EnrichedContext<'a> {
    pub user_id: Option<&'a str>,
    pub session_id: Option<&'a str>,
    pub environment: Option<&'a str>,
    pub app_name: Option<&'a str>,
    pub current_time: Option<&'a str>,
    pub remote_address: Option<&'a str>,
    pub properties: Option<PropertiesRef<'a>>,
    pub external_results: Option<ExternalResultsRef<'a>>,
    pub toggle_name: &'a str,
    pub runtime_hostname: Option<&'a str>,
}

impl<'a> EnrichedContext<'a> {
    pub fn from(
        context: &'a Context,
        toggle_name: &'a str,
        external_results: Option<&'a HashMap<String, bool>>,
    ) -> Self {
        EnrichedContext {
            user_id: context.user_id.as_deref(),
            session_id: context.session_id.as_deref(),
            environment: context.environment.as_deref(),
            app_name: context.app_name.as_deref(),
            current_time: context.current_time.as_deref(),
            remote_address: context.remote_address.as_deref(),
            properties: context.properties.as_ref().map(PropertiesRef::Strings),
            external_results: external_results.map(ExternalResultsRef::Strings),
            toggle_name,
            runtime_hostname: None,
        }
    }
}

impl<'a> EnrichedContext<'a> {
    pub fn with_toggle_name(&self, toggle_name: &'a str) -> EnrichedContext<'a> {
        EnrichedContext {
            user_id: self.user_id,
            session_id: self.session_id,
            environment: self.environment,
            app_name: self.app_name,
            current_time: self.current_time,
            remote_address: self.remote_address,
            properties: self.properties,
            external_results: self.external_results,
            toggle_name,
            runtime_hostname: self.runtime_hostname,
        }
    }
}

#[derive(Debug)]
pub enum SdkError {
    StrategyEvaluationError,
    StrategyParseError(String),
}
