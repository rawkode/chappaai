use k8s_openapi::api::core::v1::Secret;

#[derive(Debug)]
enum Decoded {
    Utf8(String),
    Bytes(Vec<u8>),
}

fn decode(secret: &Secret, key: &String) -> Result<Decoded, crate::Error> {
    let encoded_value = match &secret.data {
        Some(map) => match map.get(key) {
            Some(value) => value,
            None => {
                return Err(crate::Error::GenericError(format!(
                    "Secret has no key called {}",
                    key
                )))
            }
        },
        None => {
            return Err(crate::Error::GenericError(format!(
                "Secret has no data to contain {}",
                key
            )))
        }
    };

    if let Ok(b) = std::str::from_utf8(&encoded_value.0) {
        Ok(Decoded::Utf8(b.to_string()))
    } else {
        Ok(Decoded::Bytes(encoded_value.0.clone()))
    }
}

pub fn get_string_value(secret: &Secret, key: &String) -> Result<String, crate::Error> {
    match decode(secret, key) {
        Ok(Decoded::Utf8(value)) => Ok(value),
        Err(error) => Err(error),
        _ => Err(crate::Error::GenericError(format!(
            "Secret has {} but it is not a string",
            key
        ))),
    }
}
