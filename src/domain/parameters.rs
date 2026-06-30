use serde::{Deserialize, Serialize};
use std::{fmt, ops::Deref};

#[derive(Deserialize)]
pub struct Parameters {
    pub subscriptions_token: SubscriptionToken,
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.subscriptions_token)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct SubscriptionToken(String);

impl SubscriptionToken {
    pub fn parse(s: String) -> Result<Self, String> {
        if s.len() == 25 && s.chars().all(|c| c.is_alphanumeric()) {
            Ok(Self(s))
        } else {
            Err("Invalid token format".to_string())
        }
    }
}

impl fmt::Display for SubscriptionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for SubscriptionToken {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl<'de> serde::Deserialize<'de> for SubscriptionToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SubscriptionToken::parse(s).map_err(serde::de::Error::custom)
    }
}

impl Deref for SubscriptionToken {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
