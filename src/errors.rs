#[derive(Debug)]
pub struct CurrentIPNotInWhitelistError;

impl std::fmt::Display for CurrentIPNotInWhitelistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Current IP is not in whitelist")
    }
}

impl std::error::Error for CurrentIPNotInWhitelistError {}