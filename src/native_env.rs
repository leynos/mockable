//! Lossless access to native operating-system environment values.

use std::ffi::{OsStr, OsString};

/// Reads environment variables without converting their native values to Unicode.
///
/// An absent variable is distinct from a variable whose value is empty. The methods
/// return owned operating-system strings so callers can retain values without
/// borrowing process state. Implementations may instead read an injected snapshot.
///
/// The trait is object-safe, so a reader can be passed through a runtime-selected
/// boundary as `&dyn NativeEnv`.
///
/// ```
/// use mockable::{DefaultNativeEnv, NativeEnv};
/// use std::ffi::OsStr;
///
/// fn is_enabled(environment: &dyn NativeEnv, key: &OsStr) -> bool {
///     environment.raw_os(key).as_deref() == Some(OsStr::new("1"))
/// }
///
/// let environment = DefaultNativeEnv;
/// let _enabled = is_enabled(&environment, OsStr::new("FEATURE_FLAG"));
/// ```
pub trait NativeEnv: Send + Sync {
    /// Returns the native value for `key`, or `None` when the key is absent.
    ///
    /// A present empty value returns `Some(OsString::new())`, preserving the
    /// distinction from an absent key. Key requirements follow
    /// [`std::env::var_os`].
    fn raw_os(&self, key: &OsStr) -> Option<OsString>;

    /// Returns all entries available to this reader as owned native pairs.
    ///
    /// The order is unspecified, and duplicate pairs are retained if the source
    /// supplies them. Keys and values are returned without Unicode conversion or
    /// normalization. This snapshot does not promise atomic consistency with a
    /// separate [`Self::raw_os`] call.
    fn all_os(&self) -> Vec<(OsString, OsString)>;
}

/// Reads the current process environment through the operating system's native
/// string interface.
///
/// This explicit adapter leaves the existing Unicode-oriented [`crate::Env`]
/// implementation unchanged.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultNativeEnv;

impl NativeEnv for DefaultNativeEnv {
    fn raw_os(&self, key: &OsStr) -> Option<OsString> {
        std::env::var_os(key)
    }

    fn all_os(&self) -> Vec<(OsString, OsString)> {
        std::env::vars_os().collect()
    }
}

#[cfg(feature = "mock")]
mockall::mock! {
    /// `mockall` implementation of [`NativeEnv`].
    ///
    /// **This is supported on `feature=mock` only.**
    pub NativeEnv {}

    impl NativeEnv for NativeEnv {
        fn raw_os(&self, key: &OsStr) -> Option<OsString>;
        fn all_os(&self) -> Vec<(OsString, OsString)>;
    }
}
