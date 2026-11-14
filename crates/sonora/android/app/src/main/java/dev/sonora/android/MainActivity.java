// The activity `android-activity` starts: it loads the Rust library and lets the glue
// spawn `android_main(AndroidApp)` on its own thread. Everything after that is GPUI.
//
// The only host responsibility here, beyond loading the library, is
// forwarding `onActivityResult` into `dev.zed.gpui.NativeBridge` so
// `gpui_android`'s SAF / PhotoPicker round-trip can resolve. Everything
// else (clipboard, intents, IME, …) goes through JNI from
// `gpui_android` directly without per-host glue.
package dev.sonora.android;

import android.content.Intent;

import com.google.androidgamesdk.GameActivity;

import dev.zed.gpui.NativeBridge;

public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("sonora");
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        // Forward every result into the Rust-side picker registry. Codes
        // gpui_android didn't issue are dropped server-side with a log
        // line, so it's safe to forward unconditionally.
        NativeBridge.onActivityResult(requestCode, resultCode, data);
    }
}
