# Rust Key Listener

A Rust library for global keyboard event listening and hotkey management on Windows.

## Features

- Global keyboard event capture
- Support for complex key chord combinations
- Customizable callback execution on key events
- Configurable event blocking and trigger intervals
- Thread-safe design for concurrent operations

## Usage

```rust
use std::time::Duration;
use std::sync::Arc;

fn main() {
    let listener = KeyListener::new();

    listener.listen(
        "Ctrl+Shift+A",
        true,
        Duration::from_millis(500),
        Arc::new(|| println!("Hotkey triggered!"))
    );

    // Keep the main thread alive
    std::thread::park();
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
key_listener = "0.1.1"
```

## Requirements

- Windows OS
- Rust 1.80+

## License

[MIT License](LICENSE)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
