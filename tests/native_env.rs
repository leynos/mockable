//! Verify native environment readers without mutating the parent environment.

use std::{
    ffi::{OsStr, OsString},
    io,
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

use mockable::{DefaultNativeEnv, NativeEnv};

const CHILD_MARKER: &str = "MOCKABLE_NATIVE_ENV_CHILD";
const CHILD_VALUE: &str = "native-fixture-v1";
const CHILD_TIMEOUT: Duration = Duration::from_secs(20);

#[cfg(unix)]
fn non_unicode(prefix: &str) -> OsString {
    use std::os::unix::ffi::OsStringExt;

    let mut bytes = prefix.as_bytes().to_vec();
    bytes.push(0xff);
    OsString::from_vec(bytes)
}

#[cfg(windows)]
fn non_unicode(prefix: &str) -> OsString {
    use std::os::windows::ffi::OsStringExt;

    let mut units: Vec<_> = prefix.encode_utf16().collect();
    units.push(0xd800);
    OsString::from_wide(&units)
}

fn fixtures() -> Vec<(OsString, OsString)> {
    let entries = vec![
        (CHILD_MARKER.into(), CHILD_VALUE.into()),
        ("MOCKABLE_UNICODE".into(), "grüße🦀".into()),
        ("MOCKABLE_EMPTY".into(), OsString::new()),
    ];
    #[cfg(any(unix, windows))]
    {
        let mut entries = entries;
        entries.push(("MOCKABLE_NATIVE_VALUE".into(), non_unicode("value-")));
        entries.push((
            non_unicode("MOCKABLE_NATIVE_KEY_"),
            "native-key-value".into(),
        ));
        entries
    }
    #[cfg(not(any(unix, windows)))]
    entries
}

/// Keep cleanup ownership on timeout and on any fallible process operation.
struct FixtureChild(Option<Child>);

impl Drop for FixtureChild {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Wait only for this owned child, with a deadline independent of test threads.
fn wait_for_child(child: Child) -> io::Result<Output> {
    let mut owned = FixtureChild(Some(child));
    let started = Instant::now();
    loop {
        let process = owned
            .0
            .as_mut()
            .ok_or_else(|| io::Error::other("fixture child missing"))?;
        if process.try_wait()?.is_some() {
            return owned
                .0
                .take()
                .ok_or_else(|| io::Error::other("fixture child missing"))?
                .wait_with_output();
        }
        if started.elapsed() >= CHILD_TIMEOUT {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "native fixture child timed out",
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

/// Spawn the same ordinary test with a controlled, exclusively owned environment.
fn run_fixture_child() -> io::Result<Output> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args([
            "--exact",
            "native_adapter_preserves_seeded_environment",
            "--nocapture",
            "--test-threads=1",
        ])
        .env_clear()
        .envs(fixtures())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Windows may need its system directory to load the test executable.
    // Copy this one known, non-secret value rather than inheriting the host.
    #[cfg(windows)]
    if let Some(root) = DefaultNativeEnv.raw_os(OsStr::new("SystemRoot")) {
        command.env("SystemRoot", root);
    }
    wait_for_child(command.spawn()?)
}

#[test]
fn native_adapter_preserves_seeded_environment() {
    let reader: &dyn NativeEnv = &DefaultNativeEnv;
    // The controller is independent of the reader under test: a broken raw_os
    // must fail a fixture assertion instead of recursively spawning children.
    if std::env::var_os(CHILD_MARKER) != Some(CHILD_VALUE.into()) {
        let output = run_fixture_child().expect("native environment fixture should finish");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "fixture child failed: {stdout}\n{stderr}"
        );
        assert!(stdout.contains("native_adapter_preserves_seeded_environment"));
        assert!(stdout.contains("1 passed; 0 failed; 0 ignored"));
        return;
    }

    let mut actual = reader.all_os();
    #[cfg(any(unix, windows))]
    {
        assert!(non_unicode("value-").to_str().is_none());
        assert!(non_unicode("MOCKABLE_NATIVE_KEY_").to_str().is_none());
    }
    // This oracle runs only in the env_clear child, never over the host's values.
    let mut expected: Vec<_> = std::env::vars_os().collect();
    actual.sort();
    expected.sort();
    assert!(
        actual == expected,
        "enumeration must retain every native pair"
    );
    for (index, (key, value)) in fixtures().into_iter().enumerate() {
        assert!(
            actual.contains(&(key.clone(), value.clone())),
            "missing fixture {index}"
        );
        assert!(
            reader.raw_os(&key) == Some(value),
            "lookup changed fixture {index}"
        );
    }
    assert_eq!(reader.raw_os(OsStr::new("MOCKABLE_NOT_PRESENT")), None);
    assert_eq!(
        reader.raw_os(OsStr::new("MOCKABLE_EMPTY")),
        Some(OsString::new())
    );
    actual.clear();
    assert_eq!(
        reader.all_os().len(),
        expected.len(),
        "returned snapshots are owned"
    );
}

#[cfg(feature = "mock")]
#[test]
fn mock_lookup_distinguishes_absence_empty_and_unicode() {
    for expected in [None, Some(OsString::new()), Some("value".into())] {
        let mut reader = mockable::MockNativeEnv::new();
        let returned = expected.clone();
        reader
            .expect_raw_os()
            .withf(|key| key == OsStr::new("MOCKABLE_KEY"))
            .times(1)
            .return_once(move |_| returned);
        let reader: &dyn NativeEnv = &reader;
        assert_eq!(reader.raw_os(OsStr::new("MOCKABLE_KEY")), expected);
    }
}

#[cfg(all(feature = "mock", any(unix, windows)))]
#[test]
fn mock_lookup_preserves_non_unicode_keys_and_values() {
    let key = non_unicode("key-");
    let expected_key = key.clone();
    let value = non_unicode("value-");
    let expected_value = value.clone();
    let mut reader = mockable::MockNativeEnv::new();
    reader
        .expect_raw_os()
        .withf(move |key| key == expected_key.as_os_str())
        .times(1)
        .return_once(move |_| Some(value));
    assert_eq!(reader.raw_os(&key), Some(expected_value));
}

#[cfg(feature = "mock")]
#[test]
fn mock_enumeration_returns_independent_owned_pairs() {
    let mut expected = fixtures();
    expected.extend(fixtures());
    let returned = expected.clone();
    let mut reader = mockable::MockNativeEnv::new();
    reader
        .expect_all_os()
        .times(2)
        .returning(move || returned.clone());
    let mut first = reader.all_os();
    assert_eq!(
        first, expected,
        "mock enumeration preserves duplicate native pairs"
    );
    first.clear();
    assert_eq!(
        reader.all_os(),
        expected,
        "snapshots do not share mutable storage"
    );
}
