use serde::de::{self, Deserialize, Deserializer, Visitor};
#[derive(Debug)]
pub struct Base64(pub String);
struct Base64Visitor;

impl<'de> Visitor<'de> for Base64Visitor {
    type Value = Base64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A base64 encoded buffer")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let b64_bytes = base64::decode(value).map_err(|_| E::custom("Not base64"))?;
        String::from_utf8(b64_bytes)
            .map_err(|_| E::custom("No UTF8"))
            .map(Base64)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}

impl<'de> Deserialize<'de> for Base64 {
    fn deserialize<D>(deserializer: D) -> Result<Base64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Base64Visitor)
    }
}
