mod builtins;
mod method_name;

use super::errors::ParseDidError;
pub use builtins::{Plc, Universal, Web};
pub use method_name::MethodName;
use smol_str::SmolStr;

pub trait Method {
    fn name() -> impl Into<MethodName>;
    fn validate_identifier(identifier: impl AsRef<str>) -> Result<(), ParseDidError>;

    fn parse(did: impl AsRef<str>) -> Result<SmolStr, ParseDidError> {
        let did = did.as_ref();
        if !super::REGEX.is_match(did) {
            return Err(ParseDidError::InvalidSyntax);
        } else if did.len() > super::MAX_LENGTH {
            return Err(ParseDidError::TooLong);
        }

        match Self::name().into() {
            MethodName::Defined(method_name) => {
                let prefix = format!("did:{}:", method_name);
                if did.starts_with(&prefix) {
                    let identifier = did[prefix.len()..].to_lowercase();
                    Self::validate_identifier(&identifier)?;

                    Ok(SmolStr::new(identifier))
                } else {
                    Err(ParseDidError::BadMethod(
                        did.split(':')
                            .nth(1)
                            .map(|method| method.into())
                            .unwrap_or_default(),
                    ))
                }
            }
            MethodName::Undefined => Ok(SmolStr::new(did)),
        }
    }
}
