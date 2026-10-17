//! Inject a native environment reader without converting paths through Unicode.

use std::{ffi::OsStr, path::PathBuf};

use mockable::{DefaultNativeEnv, NativeEnv};

fn configuration_path(env: &dyn NativeEnv) -> Option<PathBuf> {
    env.raw_os(OsStr::new("APP_CONFIG")).map(PathBuf::from)
}

fn main() {
    let path = configuration_path(&DefaultNativeEnv);
    println!("Configuration path supplied: {}", path.is_some());
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    //! Exercise the example's injection boundary with an owned native path.

    use super::*;

    #[test]
    fn loads_the_supplied_configuration_path() {
        let mut env = mockable::MockNativeEnv::new();
        env.expect_raw_os()
            .withf(|key| key == OsStr::new("APP_CONFIG"))
            .times(1)
            .return_once(|_| Some("config.toml".into()));
        assert_eq!(configuration_path(&env), Some(PathBuf::from("config.toml")));
    }
}
