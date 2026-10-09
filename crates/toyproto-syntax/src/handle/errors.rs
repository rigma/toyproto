#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ParseHandleError {
    #[error("provided handle is not a valid syntax")]
    InvalidSyntax,

    #[error("provided handle has a TLD not allowed")]
    InvalidTopLevelDomain,
}
