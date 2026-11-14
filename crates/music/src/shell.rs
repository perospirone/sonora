//! Handing a URL to the user's browser.
//!
//! Every sign-in starts this way: the provider builds an authorization URL, the platform opens
//! it, and the callback lands back in the app. Desktop shells hand it to `xdg-open`-and-friends;
//! Android has neither, so it starts an `ACTION_VIEW` intent over JNI.

/// Opens `url` in whatever the platform treats as the user's browser.
pub fn open(url: impl AsRef<str>) {
    imp(url.as_ref());
}

#[cfg(not(target_os = "android"))]
fn imp(url: &str) {
    open::that_in_background(url);
}

#[cfg(target_os = "android")]
fn imp(url: &str) {
    if let Err(error) = intent(url) {
        log::warn!("shell: cannot open {url}: {error:#}");
    }
}

#[cfg(target_os = "android")]
fn intent(url: &str) -> anyhow::Result<()> {
    use jni::objects::JObject;

    let context = ndk_context::android_context();
    // SAFETY: `android-activity` initialised the process-wide `JavaVM` before it called
    // `android_main`, and the pointer it left in `ndk-context` is that same VM.
    let vm = unsafe { jni::JavaVM::from_raw(context.vm().cast()) };
    vm.attach_current_thread(|env| {
        // SAFETY: the activity reference is a global JNI ref held by `android-activity` for the
        // life of the process, so borrowing it for this call is valid.
        let activity = unsafe { JObject::from_raw(env, context.context().cast()) };
        start_view(env, &activity, url)
    })?;
    Ok(())
}

/// Runs `new Intent(Intent.ACTION_VIEW); intent.setData(Uri.parse(url)); activity.startActivity`.
#[cfg(target_os = "android")]
fn start_view(
    env: &mut jni::Env<'_>,
    activity: &jni::objects::JObject<'_>,
    url: &str,
) -> anyhow::Result<()> {
    use anyhow::Context as _;
    use jni::{jni_sig, jni_str, objects::JValue};

    let url = env.new_string(url).context("alloc the URL")?;
    let uri_class = env
        .find_class(jni_str!("android/net/Uri"))
        .context("find android.net.Uri")?;
    let uri = env
        .call_static_method(
            &uri_class,
            jni_str!("parse"),
            jni_sig!((text: "java.lang.String") -> "android.net.Uri"),
            &[JValue::Object(&url)],
        )
        .context("Uri.parse")?
        .l()
        .context("Uri.parse returned nothing")?;

    let action = env
        .new_string("android.intent.action.VIEW")
        .context("alloc the intent action")?;
    let intent_class = env
        .find_class(jni_str!("android/content/Intent"))
        .context("find android.content.Intent")?;
    let intent = env
        .new_object(
            &intent_class,
            jni_sig!((action: "java.lang.String") -> ()),
            &[JValue::Object(&action)],
        )
        .context("new Intent")?;
    env.call_method(
        &intent,
        jni_str!("setData"),
        jni_sig!((data: "android.net.Uri") -> "android.content.Intent"),
        &[JValue::Object(&uri)],
    )
    .context("Intent.setData")?;
    env.call_method(
        activity,
        jni_str!("startActivity"),
        jni_sig!((intent: "android.content.Intent") -> ()),
        &[JValue::Object(&intent)],
    )
    .context("Context.startActivity")?;
    Ok(())
}
