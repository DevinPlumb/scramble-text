# scramble-text

A WebAssembly text scrambling animation library written in Rust.

## Features

- Text scrambling animation with configurable parameters
- WebAssembly-powered for high performance
- Works in any modern browser
- No JavaScript dependencies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
scramble-text = "0.1.0"
```

## Usage

### As a Rust library

```rust
use scramble_text::ScrambleText;

let element = // ... get DOM element ...;
let scramble = ScrambleText::new(element, UseScrambleProps {
    text: "Hello World".to_string(),
    speed: 0.8,
    ..Default::default()
})?;

scramble.start()?;
```

### In the browser

```html
<script type="module">
  import init, { ScrambleText } from './pkg/scramble_text.js';

  async function run() {
    await init();

    const element = document.querySelector('#my-text');
    const scramble = new ScrambleText(element, {
      text: 'Hello World',
      speed: 0.8,
      chance: 0.7,
    });

    scramble.start();
  }

  run();
</script>
```

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

### Building

```bash
# Build WebAssembly package
wasm-pack build --target web

# Build documentation
cargo doc --open
```

### Testing

```bash
# Run Rust tests
cargo test

# Run WebAssembly tests in Chrome
wasm-pack test --chrome

# Run WebAssembly tests in Firefox
wasm-pack test --firefox

# Run WebAssembly tests in headless Chrome
wasm-pack test --headless --chrome
```

### Example

To run the example:

```bash
# First build the wasm package
wasm-pack build --target web

# Then serve the examples directory
# Using Python's built-in server:
python3 -m http.server
# Or any other static file server
```

Then visit `http://localhost:8000/examples/` in your browser.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
