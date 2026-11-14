//! The Android shared library.
//!
//! `android-activity` starts the `GameActivity`, loads this `.so`, and calls the `android_main`
//! symbol it finds. All of that lives here rather than in the `sonora` package so a desktop
//! build produces exactly the same artefacts it always did: this crate is empty off Android.

#![cfg(target_os = "android")]

/// The symbol the GameActivity glue looks up, which boots the app on the thread it hands over.
#[unsafe(no_mangle)]
fn android_main(app: sonora::android::AndroidApp) {
    sonora::android::start(app);
}
