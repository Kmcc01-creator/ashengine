# Getting Started with AshEngine

This guide will help you get AshEngine up and running on your system.

## Prerequisites

Before you begin, make sure you have the following installed:

- Rust: Install the latest stable version of Rust using rustup (https://rustup.rs/).
- Vulkan SDK: Download and install the Vulkan SDK for your platform (https://vulkan.lunarg.com/sdk/home).
- Git: Install Git for version control (https://git-scm.com/).

## Cloning the Repository

```bash
git clone https://github.com/your-username/ashengine  # Replace with the actual repository URL
cd ashengine
```

**Note:** The repository URL above is a placeholder. You'll need to replace it with the actual URL if the project is publicly hosted. If it is not public, remove this section.

## Building the Project

AshEngine uses the Cargo build system. To build the project, run the following command in the project's root directory (`ashengine`):

```bash
cargo build --release
```

This will compile the engine and its dependencies in release mode, creating an optimized executable.

## Running the Examples

The `examples/` directory contains sample applications that demonstrate how to use AshEngine. To run an example, navigate to the `examples/` directory and use Cargo:

```bash
cd examples
cargo run --example text_demo # Example: Run the text_demo
```

This will build and run the `text_demo` example. You can replace `text_demo` with the name of other examples.
