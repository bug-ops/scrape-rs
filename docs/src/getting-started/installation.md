# Installation

scrape-rs provides bindings for multiple platforms. Choose the installation method for your platform:

## Rust

Add scrape-core to your `Cargo.toml`:

```toml
[dependencies]
scrape-core = "0.2"
```

Or use `cargo add`:

```bash
cargo add scrape-core
```

### Feature Flags

scrape-core supports optional features:

```toml
[dependencies]
scrape-core = { version = "0.2", features = ["streaming", "parallel", "simd"] }
```

| Feature | Description | Default |
|---------|-------------|---------|
| `streaming` | Enable streaming parser with constant memory usage | No |
| `parallel` | Enable parallel batch processing with Rayon | No |
| `simd` | Enable SIMD-accelerated text processing | No |
| `serde` | Enable serialization support | No |

## Python

Install via pip:

```bash
pip install fast-scrape
```

Or with uv:

```bash
uv pip install fast-scrape
```

### Requirements

- Python 3.10 or later
- Supported platforms: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)

### Virtual Environment

We recommend using a virtual environment:

```bash
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install fast-scrape
```

## Node.js

Install via npm:

```bash
npm install @scrape-rs/scrape
```

Or with pnpm:

```bash
pnpm add @scrape-rs/scrape
```

Or with yarn:

```bash
yarn add @scrape-rs/scrape
```

### Requirements

- Node.js 18 or later
- Supported platforms: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)

### TypeScript Support

TypeScript types are included automatically. No additional `@types` package is needed.

## WASM (Browser)

Install via npm:

```bash
npm install @scrape-rs/wasm
```

Or with pnpm:

```bash
pnpm add @scrape-rs/wasm
```

### Usage in Browser

```typescript
import init, { Soup } from '@scrape-rs/wasm';

// Initialize WASM module (required once)
await init();

const soup = new Soup('<html>...</html>');
```

### Requirements

- Modern browser with WASM support (Chrome 57+, Firefox 52+, Safari 11+, Edge 16+)
- Bundle size: ~400KB (gzipped: ~120KB)

### Webpack Configuration

If using Webpack, add to your config:

```javascript
module.exports = {
  experiments: {
    asyncWebAssembly: true,
  },
};
```

### Vite Configuration

If using Vite, add `vite-plugin-wasm`:

```bash
npm install vite-plugin-wasm
```

```javascript
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [wasm()],
});
```

## Verifying Installation

After installation, verify it works:

### Rust

```bash
cargo run --example basic
```

Or create a test file:

```rust
use scrape_core::Soup;

fn main() {
    let soup = Soup::parse("<html><body><h1>Hello</h1></body></html>");
    println!("{:?}", soup.find("h1"));
}
```

### Python

```bash
python -c "from scrape_rs import Soup; print(Soup('<h1>Test</h1>').find('h1').text)"
```

### Node.js

```bash
node -e "const {Soup} = require('@scrape-rs/scrape'); console.log(new Soup('<h1>Test</h1>').find('h1').text)"
```

### WASM

Create a test HTML file:

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>scrape-rs WASM Test</title>
</head>
<body>
    <script type="module">
        import init, { Soup } from './node_modules/@scrape-rs/wasm/scrape_wasm.js';

        await init();
        const soup = new Soup('<h1>Hello WASM</h1>');
        const h1 = soup.find('h1');
        console.log('Success:', h1.text);
        document.body.innerHTML = `<p>Result: ${h1.text}</p>`;
    </script>
</body>
</html>
```

## Troubleshooting

### Rust: Compilation Errors

If you see compilation errors, ensure you're using Rust 1.75 or later:

```bash
rustc --version
rustup update
```

### Python: No Matching Distribution

If you get "no matching distribution found", ensure you're using Python 3.10+:

```bash
python --version
```

If on an unsupported platform, you can build from source:

```bash
pip install maturin
git clone https://github.com/bug-ops/scrape-rs.git
cd scrape-rs/crates/scrape-py
maturin develop --release
```

### Node.js: Binary Not Found

If the native module fails to load, ensure your platform is supported:

```bash
node -p "process.platform + '-' + process.arch"
```

Supported: `linux-x64`, `linux-arm64`, `darwin-x64`, `darwin-arm64`, `win32-x64`

### WASM: Module Not Found

Ensure your bundler is configured to handle WASM files. See platform-specific configuration above.

## Next Steps

Now that you have scrape-rs installed, proceed to the [Quick Start](quick-start.md) guide to learn the basics.
