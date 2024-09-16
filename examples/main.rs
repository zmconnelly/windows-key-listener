use std::{sync::Arc, time::Duration};

use windows_key_listener::KeyListener;

fn main() {
    let key_listener = KeyListener::new();

    let callback_interval = Duration::from_millis(200);

    key_listener.listen(
        "Ctrl + Shift + Z",
        false,
        callback_interval,
        Arc::new(|| {
            println!("Ctrl + Shift + Z pressed!");
        }),
    );

    key_listener.listen(
        "VolumeUp",
        true,
        callback_interval,
        Arc::new(|| {
            println!("VolumeUp pressed!");
        }),
    );

    key_listener.listen(
        "VolumeDown",
        false,
        callback_interval,
        Arc::new(|| {
            println!("VolumeDown pressed!");
        }),
    );

    key_listener.listen(
        "VolumeMute",
        true,
        callback_interval,
        Arc::new(|| {
            println!("VolumeMute pressed!");
        }),
    );

    // Keep the main thread alive to keep listening for key presses
    loop {
        std::thread::sleep(std::time::Duration::from_secs(15));
        println!("Unlistening...");
        key_listener.unlisten();
    }
}
