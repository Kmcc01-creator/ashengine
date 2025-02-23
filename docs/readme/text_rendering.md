# Text Rendering

This document describes text rendering in AshEngine.

AshEngine uses a combination of techniques for efficient and high-quality text rendering:

- **Signed Distance Fields (SDF):** Glyphs are stored as SDFs in a texture atlas, allowing for sharp rendering at various scales.
- **Fontdue:** The `fontdue` crate is used for font loading and rasterization.
- **Custom Memory Allocator:** Efficient memory management for buffers and textures.
- **Batched Rendering:** Text is rendered in batches to minimize draw calls.

## Core Components

The text rendering system is composed of several key components:

- **`FontManager` (`text/font.rs`):**

  - Loads and manages fonts.
  - Uses the `fontdue` crate for font parsing and rasterization.
  - Generates SDF metrics for glyphs.
  - Provides access to loaded fonts by name and a default font.

- **`FontAtlas` (`text/atlas.rs`):**

  - Manages the texture atlas that stores glyph SDF data.
  - Creates the Vulkan texture, image view, and sampler.
  - Handles glyph generation and updates the texture with glyph data.
  - Provides UV coordinates for each glyph within the atlas.
  - Manages descriptor sets for accessing the atlas in shaders.

- **`TextLayout` (`text/layout.rs`):**

  - Takes a collection of `TextElement` structs and generates the necessary vertex and index data for rendering.
  - Calculates bounding boxes for text elements.
  - Uses the `FontAtlas` to retrieve glyph information.

- **`TextElement` (`text/layout.rs`):**

  - Represents a string of text to be rendered, along with its position, color, scale, and an ID.

- **`TextVertex` (`text/vertex.rs`):**

  - Defines the vertex format used for text rendering, including position, texture coordinates, color, and an element ID.

- **`BoundingBox` and `Rect` (`text/layout.rs`):**

  - Helper structs for managing bounding boxes and rectangles.

- **`TextConfig` (`text/mod.rs`):**
  - Configuration struct for setting text rendering parameters (font size, line height, alignment, color, etc.).

## Rendering Process

1.  **Font Loading:** Fonts are loaded using the `FontManager`.
2.  **Glyph Generation:** For each required glyph, the `FontAtlas` generates SDF data using the `FontManager`.
3.  **Texture Update:** The generated SDF data is copied to the `FontAtlas` texture.
4.  **Text Layout:** The `TextLayout` takes a collection of `TextElement` structs and generates vertex and index data. It uses the `FontAtlas` to get glyph metrics and UV coordinates.
5.  **Rendering:** The generated vertex and index data is used to render the text using a graphics pipeline. The pipeline uses shaders that sample the font atlas texture and apply SDF-based rendering techniques.

## Usage

1.  **Load Fonts:** Use `FontManager::load_font` to load fonts.
2.  **Create a `FontAtlas`:** Instantiate a `FontAtlas` with the desired dimensions.
3.  **Generate Glyphs:** Use `FontAtlas::generate_glyph` for each character you need to render.
4.  **Create `TextElement`s:** Define the text you want to render using `TextElement` structs.
5.  **Layout Text:** Use `TextLayout::layout_text` to generate vertex and index data.
6.  **Render:** Use the generated vertex and index data with a suitable graphics pipeline and shaders to render the text. The shaders should sample the `FontAtlas` texture using the provided UV coordinates.

## Configuration

The `TextConfig` struct (in `text/mod.rs`) allows you to configure various text rendering parameters:

```rust
pub struct TextConfig {
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub alignment: TextAlignment,
    pub color: [f32; 4],
}
```

You can use this struct to customize the appearance of your text.
