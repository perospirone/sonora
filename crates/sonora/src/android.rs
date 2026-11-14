//! The Android entry point.
//!
//! `android-activity` calls [`android_main`] on the thread it has already attached to the JVM,
//! once the `GameActivity` is up. Everything after that is the same startup a desktop runs:
//! [`crate::boot`] builds the state, then the shared window opens with the same `views::Root`.
//! The desktop-only shells (tray, dock, single instance, CLI hand-off, MPRIS) are never
//! reached here.

use std::sync::Arc;

use anyhow::Context as _;
use gpui::App;
use gpui_android::AndroidApp;

use crate::{actions, assets, boot, http, memory, open_window};

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    if let Err(error) = start(app) {
        log::error!("sonora: android startup failed: {error:#}");
    }
}

/// Points the process at the directories Android gives the app, then runs Sonora in them.
///
/// The environment comes first: `dirs` reads `XDG_DATA_HOME`, `XDG_CONFIG_HOME` and
/// `XDG_CACHE_HOME` on unix, and a freshly started Android process has none of them, so without
/// this every path in the app resolves to `/`.
fn start(app: AndroidApp) -> anyhow::Result<()> {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let files = gpui_android::paths::files_dir(&app)
        .context("cannot find the app's files directory")?;
    let cache =
        gpui_android::paths::cache_dir(&app).context("cannot find the app's cache directory")?;
    std::fs::create_dir_all(&files).ok();
    std::fs::create_dir_all(&cache).ok();
    // SAFETY: this runs on the one thread android-activity hands the app, before GPUI or the
    // runtime has spawned any worker, so no other thread can read the environment yet.
    unsafe {
        std::env::set_var("HOME", &files);
        std::env::set_var("XDG_DATA_HOME", files.join("data"));
        std::env::set_var("XDG_CONFIG_HOME", files.join("config"));
        std::env::set_var("XDG_CACHE_HOME", &cache);
        // Every HTTPS client in the app reads the platform's trust store through
        // `openssl-probe`, which knows the desktop locations and nothing on Android. Android
        // keeps the same PEM bundle in one of these two places.
        for candidates in [
            "/apex/com.android.conscrypt/cacerts",
            "/system/etc/security/cacerts",
        ] {
            if std::path::Path::new(candidates).is_dir() {
                std::env::set_var("SSL_CERT_DIR", candidates);
                break;
            }
        }
    }

    trust_android_certificates(&app)?;
    gpui_android::set_android_app(app);
    memory::tune();

    let io = state::Io::new().context("cannot start the runtime")?;
    let app = gpui_platform::application()
        .with_assets(assets::Assets)
        .with_http_client(Arc::new(http::Client::new(io.handle())));

    app.run(move |cx: &mut App| {
        boot(cx, io, storage::Database::standard());
        actions::register(false, cx);
        memory::watch(cx);
        open_window(cx);
        let session = state::Sonora::global(cx).session.clone();
        session.update(cx, |session, cx| session.restore(cx));
        cx.activate(true);
    });

    Ok(())
}

/// Hands `rustls-platform-verifier` the JVM and the activity, so certificate checks go through
/// Android's own trust store.
///
/// reqwest validates TLS through this crate on every platform, and on Android it panics on the
/// first handshake until it has been initialized, which takes down the thread that asked.
fn trust_android_certificates(app: &AndroidApp) -> anyhow::Result<()> {
    use jni::objects::JObject;

    // SAFETY: `android-activity` keeps the VM and the activity alive for the whole process, and
    // this runs on its own main thread before the runtime has spawned any worker.
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) };
    vm.attach_current_thread(|env| {
        let context = unsafe { JObject::from_raw(env, app.activity_as_ptr().cast()) };
        rustls_platform_verifier::android::init_with_env(env, context)
    })
    .context("cannot initialize Android certificate verification")?;
    Ok(())
}
