# Examples

This document describes the examples provided with AshEngine.

The `examples/` directory contains sample applications that demonstrate various features of the engine.

## Running Examples

To run an example, navigate to the `engine` directory in your terminal and use the `cargo run --example` command, followed by the example name. For instance:

```bash
cd engine
cargo run --example text_demo
```

## Available Examples

### `text_demo`

This example demonstrates basic text rendering using the engine's text rendering system. It initializes a window, loads a font, sets up text with a specified position and size, and renders it to the screen.

**Files:**

- `examples/text_demo.rs`: The main example code.
- `examples/text_blocks.ron` and `examples/text_blocks.toml`: Seem to be configuration or data files related to text rendering, but are not directly used in this specific example.
- `examples/fonts/NotoSans-Regular.ttf`: The font file used in the example.
- `examples/text_config_usage.rs`: This file is present but its purpose is not clear from the `text_demo.rs` code. It might be an incomplete or alternative example.

**Functionality:**

- Initializes a window using `winit`.
- Creates a Vulkan `Context`.
- Creates a `TextRenderer` (defined within the example itself, not part of the core engine).
- Loads a font (`NotoSans-Regular.ttf`).
- Sets up the text "Hello, World!" with a specified position and size.
- Enters the main event loop, handling window events and rendering the text in each frame.

This example showcases the basic steps involved in setting up a window, initializing the engine, loading resources (font), and rendering text.
