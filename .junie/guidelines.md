# AWS TUI Toolkit Development Guidelines

This document provides essential information for developers working on the AWS TUI Toolkit project.

## Build/Configuration Instructions

### Prerequisites
- Rust toolchain (latest stable version recommended)
- AWS CLI configured with appropriate credentials
- AWS SDK for Rust

### AWS Configuration
The application uses the AWS SDK for Rust and requires proper AWS credentials configuration. You can configure AWS credentials in one of the following ways:

1. **Environment variables**:
   ```bash
   export AWS_ACCESS_KEY_ID=your_access_key
   export AWS_SECRET_ACCESS_KEY=your_secret_key
   export AWS_REGION=your_region
   ```

2. **AWS credentials file** (recommended):
   Create or update `~/.aws/credentials` with your AWS credentials:
   ```ini
   [default]
   aws_access_key_id = your_access_key
   aws_secret_access_key = your_secret_key
   region = your_region
   ```

   You can also create multiple profiles:
   ```ini
   [profile1]
   aws_access_key_id = profile1_access_key
   aws_secret_access_key = profile1_secret_key
   region = profile1_region
   ```

### Building the Project
To build the project, run:
```bash
cargo build
```

For a release build:
```bash
cargo build --release
```

### Running the Application
To run the application:
```bash
cargo run
```

## Testing Information

### Running Tests
To run all tests:
```bash
cargo test
```

To run a specific test:
```bash
cargo test test_name
```

To run tests for a specific module:
```bash
cargo test module_name
```

### Writing Tests
Tests should be placed in a module with the `#[cfg(test)]` attribute or in a separate file with the same name as the module being tested, but with a `_test` suffix.

#### Example: Testing a Module in the Same File
```rust
// In file.rs
pub fn function_to_test() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        assert_eq!(function_to_test(), true);
    }
}
```

#### Example: Testing a Module in a Separate File
For a module in `src/module.rs`, create a test file at `src/module_test.rs`:

```rust
// In src/module_test.rs
use crate::module::YourStruct;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_struct() {
        let instance = YourStruct::new();
        assert!(instance.some_method());
    }
}
```

Then, add the test module to `main.rs`:
```rust
mod module;
#[cfg(test)]
mod module_test;
```

### Test Example
Here's a working example of tests for the `SearchBar` struct:

```rust
// In src/search_test.rs
use crate::search::SearchBar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_bar_default() {
        let search_bar = SearchBar::default();
        assert_eq!(search_bar.query, "");
        assert_eq!(search_bar.active, false);
        assert_eq!(search_bar.cursor_position, 0);
    }

    #[test]
    fn test_search_bar_toggle() {
        let mut search_bar = SearchBar::default();
        search_bar.toggle();
        assert_eq!(search_bar.active, true);
        search_bar.toggle();
        assert_eq!(search_bar.active, false);
        assert_eq!(search_bar.query, "");
        assert_eq!(search_bar.cursor_position, 0);
    }
}
```

## Additional Development Information

### Project Structure
- `src/main.rs`: Entry point of the application
- `src/app.rs`: Main application logic and state
- `src/aws/`: AWS-related functionality
  - `src/aws/s3_client.rs`: S3 client implementation
- `src/ui.rs`: UI rendering logic
- `src/list.rs`: List widget implementation
- `src/search.rs`: Search functionality

### Code Style
- Follow the Rust standard style guide
- Use `rustfmt` to format code
- Use `clippy` for linting

### TUI Framework
This project uses the `ratatui` crate for terminal UI rendering. Key points:
- The application follows the Model-View-Controller pattern
- UI rendering is done in the `render` methods
- Event handling is done in the `handle_event` methods
- The main application loop is in the `run` method of the `App` struct

### AWS SDK Integration
- The project uses the AWS SDK for Rust
- AWS configuration is loaded from the environment or credentials file
- The `AWS` struct in `aws.rs` provides a wrapper around the AWS SDK
- The `S3Client` struct in `aws/s3_client.rs` provides S3-specific functionality

### Error Handling
- The project uses `color-eyre` for error handling
- Most functions return `Result<T, Error>` to propagate errors
- Use the `?` operator to propagate errors

### Debugging
To enable debug logs, set the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run
```
