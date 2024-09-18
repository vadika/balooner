# Balooner

[![Rust](https://github.com/vadika/balooner/workflows/Rust/badge.svg)](https://github.com/vadika/balooner/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Balooner is a Rust-based tool for dynamically balancing memory across multiple QEMU virtual machines using the QEMU Monitor Protocol (QMP).

## Features

- Monitors and adjusts memory allocation for multiple VMs
- Uses QMP to interact with QEMU instances
- Graceful shutdown on SIGINT and SIGTERM
- Logs memory metrics for each VM

## Prerequisites

- Rust (edition 2021)
- QEMU with QMP enabled and virtio-balloon support

## Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/vadika/balooner.git/
   cd balooner
   ```

2. Build the project:
   ```sh
   cargo build --release
   ```

## Usage

Run the balooner with the following command:

```sh
./target/release/balooner <vm_name> <qmp_socket_path> <target_memory_mb> ...
```

You can specify multiple VMs by repeating the `<vm_name> <qmp_socket_path> <target_memory_mb>` arguments.

Example:
```sh
./target/release/balooner vm1 /tmp/vm1.sock 1024 vm2 /tmp/vm2.sock 2048
```

This will start balancing memory for two VMs: vm1 with a target of 1024 MB and vm2 with a target of 2048 MB.

## Configuration

The tool uses environment variables for logging configuration. You can set the `RUST_LOG` environment variable to control the log level:

```sh
RUST_LOG=info ./target/release/balooner ...
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Troubleshooting

If you encounter issues pushing to the repository, make sure you're using the correct branch name:

```sh
git push origin main
```
5. Open a Pull Request
