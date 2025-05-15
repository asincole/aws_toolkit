# AWS TUI Toolkit

**This exists because I am intensely lazy to keep entering OTP to browse s3 buckets. Also, Rust lang is addictive and makes you find reasons to build any and everything. As they say "Find the right job for the tool" or whatever... There was also a bit of vibe coding involved in this; I just cared that this worked and not that it does things the best ways, so there are a few bugs, silly choices for key-bindings etc... If I ever get bored, I will get back to fixing some of these things.**

A terminal-based user interface for interacting with AWS S3 services. This toolkit provides a convenient way to browse, view, and download S3 buckets and objects directly from your terminal.

## Features

- Browse S3 buckets and objects with an intuitive terminal interface
- View object content with automatic formatting for different content types (JSON, images, binary data)
- Search functionality for buckets and objects
- Download objects to your local filesystem
- Pagination support for large bucket and object lists
- Keyboard-driven navigation

## Installation

### Prerequisites

- Rust toolchain (latest stable version recommended)
- AWS CLI configured with appropriate credentials
- AWS SDK for Rust

### Building from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/aws_tui_toolkit.git
   cd aws_tui_toolkit
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run --release
   ```

## AWS Configuration

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

## Usage

### Navigation

- Use arrow keys to navigate through buckets and objects
- Press `Enter` to select a bucket or view an object
- Press `Esc` to go back to the previous view
- Press `q` to quit the application

### Search

- Press `/` to activate search mode
- Type your search query and press `Enter`
- Press `Esc` to exit search mode

### Object Operations

- Press `d` to download the selected object
- Use arrow keys to scroll through object content in preview mode

## Todo/Bug Checklist

- [ ] Add support for uploading files to S3
- [ ] Implement object deletion functionality
- [ ] Add support for creating new buckets
- [ ] Improve error handling for network failures
- [ ] Add configuration options for customizing the UI
- [ ] Fix search functionality in preview mode (currently disabled)
- [ ] Add support for more AWS services beyond S3
- [ ] Implement multi-threaded downloads for large files
- [ ] Add progress indicators for long-running operations
- [ ] Improve handling of very large text files in preview mode

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details ("is the LICENSE file in the room with us?). 

**This is actually under the vibes license, I genuinely don't care what you do with this lol... also, this is a benevolent dictatorship**
