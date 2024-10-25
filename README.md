# <h1 align="center">Layerzero DVN Blueprint Template 🌐</h1>

**A template for creating Decentralized Verifier Network (DVN) Blueprints for LayerZero V2 on Tangle**

## 📚 Overview

This project provides a template for creating Layerzero Decentralized Verifier Network (DVN) Blueprints on the Tangle Network. DVNs are an essential component of the LayerZero protocol, responsible for verifying cross-chain messages and ensuring the security and reliability of inter-blockchain communications.

Blueprints in Tangle are specifications for Actively Validated Services (AVS) that run arbitrary computations for a user-specified period of time. This template allows developers to create reusable DVN infrastructures, enabling them to participate in the LayerZero ecosystem and potentially monetize their work.

## 📚 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Forge](https://getfoundry.sh)
- [Tangle](https://github.com/tangle-network/tangle?tab=readme-ov-file#-getting-started-)

You will also need to install `cargo-tangle`, our CLI tool for creating and deploying Tangle Blueprints:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tangle-network/gadget/releases/download/cargo-tangle-v0.1.2/cargo-tangle-installer.sh | sh
```

Or, if you prefer to install the CLI from crates.io:

```bash
cargo install cargo-tangle --force # to get the latest version
```

## 🛠️ Getting Started

1. Create a new project using the DVN blueprint template:

```sh
cargo tangle blueprint create --name my-dvn-blueprint --repo https://github.com/tangle-network/layerzero-dvn-blueprint-template
```

2. Navigate to your project directory:

```sh
cd my-dvn-blueprint
```

3. Implement your offchain DVN logic in `src/lib.rs` and onchain DVN in `contracts/src/`, building upon the provided template functions.

4. Build your project:

```sh
cargo build
```

5. Deploy your DVN blueprint to the Tangle network:

```sh
cargo tangle blueprint deploy
```

## 📖 Understanding the Template

The DVN has one off-chain workflow:

1. The DVN first listens for the `PacketSent` event:
2. After the `PacketSent` event, the `DVNFeePaid` is how you know your DVN has been assigned to verify the packet's `payloadHash`.
3. After receiving the fee, your DVN should query the address of the `MessageLib` on the destination chain:
4. After your DVN has retrieved the receive `MessageLib`, you should read the `MessageLib` configuration from it. In the configuration is the required block confirmations to wait before calling verify on the destination chain.
5. Your DVN should next do an idempotency check:
   1. If the state is `true`, then your idempotency check indicates that you already verified this packet. You can terminate your DVN workflow.
   2. If the state is `false`, then you must call `ULN.verify`:

## 📚 Resources

- [LayerZero V2 Documentation](https://layerzero.network/docs)
- [Tangle Network Documentation](https://docs.tangle.tools)

## 📬 Feedback

If you have any feedback or issues, please feel free to open an issue on our [GitHub repository](https://github.com/tangle-network/layerzero-dvn-blueprint-template/issues).

## 📜 License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.