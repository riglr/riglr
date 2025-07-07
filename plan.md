INITIAL CONTEXT: WE ARE REFACTORING FROM THE GROUND UP WHAT IS IN THE listen.xml CONSOLIDATED FILE. YOU CAN USE IT AS REFERENCE WHEN NEEDED.


## **Project Plan: `riglr`**

### **1. Project Vision & Executive Summary**

**Vision:** To create the premier Rust ecosystem for building high-performance, resilient, and developer-friendly on-chain AI agents. `riglr` (pronounced "riggler") will provide a suite of modular, `rig`-compatible crates that abstract away the complexities of blockchain interaction and AI orchestration, enabling developers to build sophisticated crypto-native applications with unprecedented ease and safety.

**Executive Summary:** The `listen` project has demonstrated the potential for a full-stack, on-chain AI platform. However, its monolithic structure creates a high barrier to entry and limits the reusability of its powerful components. `riglr` will be a complete refactoring and reimagining of `listen`'s core functionalities. It will deconstruct the monolith into a set of focused, independent crates built on top of the `rig` framework. Key deliverables will include standardized toolsets for Solana and EVM chains, a resilient tool execution engine with job queueing, an advanced graph-based memory system, and a developer-first macro for creating custom tools with minimal boilerplate. This modular approach will foster a vibrant community, encourage adoption, and establish `riglr` as the foundational toolkit for the next generation of on-chain AI.

### **2. Project Goals & Guiding Principles**

* **Modularity:** Every core functionality (Solana tools, EVM tools, memory) will be an independent crate. Developers should only need to import what they use.
* **Developer Experience (DX):** The primary focus is on ease of use. This will be achieved through a `serde`-first API, powerful derive macros to eliminate boilerplate, and comprehensive documentation with practical examples.
* **Resilience & Performance:** The system must be production-ready. This means robust error handling, configurable timeouts, retries with exponential backoff, bounded concurrency, and idempotency for state-mutating operations.
* **`rig`-Native:** `riglr` is not a competitor to `rig`; it is an extension of it. All components will seamlessly integrate with `rig`'s core abstractions like `Agent`, `Tool`, and `VectorStore`.
* **Community-Driven:** The project will be structured to encourage community contributions, with clear guidelines, a starter template, and a focus on building a shared ecosystem.

### **3. Target Audience & User Stories**

#### **Personas:**

* **Devin, the DeFi Developer:** A Rust developer building a new DeFi protocol or trading bot. They are technically proficient but want to move fast without compromising on security or reliability.
* **Anna, the On-Chain Analyst:** A data scientist or analyst who is comfortable with scripting but not necessarily a systems programmer. They want to quickly build agents to query on-chain data and generate insights without getting bogged down in low-level details.

#### **User Stories:**

* **As Devin,** I want to add a `GetBalanceTool` to my `rig` agent by adding a single line to my `Cargo.toml` and one line to my agent builder, so that I can focus on my application's core logic.
* **As Devin,** I want to create a custom tool for interacting with my new protocol by simply defining a Rust function and adding a `#[tool]` macro, so that I can extend my agent's capabilities in minutes, not hours.
* **As Devin,** I want to execute a multi-step on-chain transaction (e.g., swap then stake) through an agent, knowing that if one step fails or times out, the system will handle it gracefully and not perform duplicate transactions.
* **As Anna,** I want to build a RAG agent that can answer questions about a wallet's transaction history by using a pre-built `riglr-graph-memory` crate, so that I can perform complex analysis with natural language.
* **As Anna,** I want to create an agent that monitors multiple tokens across different chains by easily composing tools from `riglr-solana-tools` and `riglr-evm-tools`, so that I can build cross-chain monitoring systems without writing chain-specific code from scratch.

### **4. System Requirements**

#### **Functional Requirements:**

1. The ecosystem must provide separate crates for Solana and EVM toolsets.
2. Tools must support reading on-chain data (balances, contract state, etc.).
3. Tools must support executing state-mutating transactions (transfers, swaps).
4. The system must provide a tool execution engine that supports job queueing.
5. The system must provide a derive macro (`#[tool]`) that automatically implements the `rig::Tool` trait from a function or struct.
6. The macro must generate a JSON schema from Rust types using `schemars`.
7. The system must provide a graph-based memory component that implements the `rig::VectorStore` trait.

#### **Non-Functional Requirements:**

1. **Performance:** Tool execution should be non-blocking. The system should handle high concurrency (e.g., 100+ parallel pipelines).
2. **Resilience:** All external API/RPC calls must have configurable timeouts and retry logic.
3. **Security:** Private keys and secrets must never be exposed to the agent's reasoning context. The `SignerContext` pattern should be enforced.
4. **Idempotency:** All state-mutating tool calls must support an optional idempotency key to prevent duplicate execution.
5. **Documentation:** Every public function and struct must be documented. Each crate must have a `README.md` with usage examples and a comprehensive `examples` directory.
6. **Test Coverage:** Core logic and all tools must have a high degree of unit and integration test coverage.

### **5. High-Level Architecture**

The `riglr` ecosystem will be a set of layered crates that build upon `rig-core`.

```mermaid
graph TD
    subgraph User Application
        A[Custom Agent]
    end

  subgraph riglr Ecosystem
    B[riglr-showcase]
    C[riglr-solana-tools]
    D[riglr-evm-tools]
    E[riglr-web-tools]
    F[riglr-graph-memory]
    G[riglr-core (Executor)]
    M[riglr-macros]
  end

    subgraph Third-Party Systems
        H[Solana RPC]
        I[EVM RPC]
        J[Web APIs / Indexers]
        K[Vector DBs e.g., Qdrant]
    end

    subgraph Core Framework
        RC[rig-core]
    end

    A --> B
    A --> C
    A --> D
    A --> E
    A --> F

    B --> C
    B --> D
    B --> E
    B --> F

  C --> G
  D --> G
  E --> G
  F --> G
  C --> M
  D --> M
  E --> M
  F --> M

    C --> H
    D --> I
    E --> J
    F --> K

    G --> RC

    style RC fill:#f9f,stroke:#333,stroke-width:2px
  style G fill:#ccf,stroke:#333,stroke-width:2px
  style M fill:#cfc,stroke:#333,stroke-width:2px
```

---

Of course. Here is the complete, deeply expanded implementation plan for all four phases of the `riglr` project (with refined crate naming: `riglr-macros`, `riglr-web-tools`, and `riglr-showcase`).

---

### **6. Detailed Implementation Plan: `riglr` (All Phases)**

#### **Phase 1: Foundation & Core Abstractions (Weeks 1-3)**

*Goal: Establish a professional project structure and build the core `riglr-core` and `riglr-macros` crates. These will serve as the bedrock for all subsequent development, ensuring resilience and a superior developer experience from day one.*

* **Epic 1: Project Setup & Governance**
  * **Subtask 1.1: Create Cargo Workspace**
    * **Action:** Initialize a new Rust project with `cargo new --lib riglr && cd riglr`.
    * **Action:** Convert it into a workspace by editing the top-level `Cargo.toml` to define the workspace members.
  * **Action:** Create initial directories for the first set of crates: `riglr-core`, `riglr-macros`, `riglr-solana-tools`, `riglr-evm-tools`.
    * **Deliverable:** A Git repository with a workspace structure that can be successfully built with `cargo build --workspace`.

  * **Subtask 1.2: Setup GitHub Repository**
    * **Action:** Create the `riglr` organization and repository on GitHub.
    * **Action:** Define a clear `CONTRIBUTING.md` outlining the process for PRs, code style, and communication (Discord/Discussions).
    * **Action:** Create issue templates for `bug_report`, `feature_request`, and `documentation_improvement`.
    * **Action:** Create a `PULL_REQUEST_TEMPLATE.md` that requires linking to an issue and a checklist for contributors.
    * **Deliverable:** A well-structured GitHub repository that is welcoming to new contributors.

  * **Subtask 1.3: Configure CI/CD Pipeline**
    * **Action:** Create a `.github/workflows/ci.yml` file.
    * **Action:** Configure a matrix build to test on multiple Rust versions (stable, beta, nightly) and platforms (Linux, macOS, Windows).
    * **Action:** Add a `cargo fmt --check` step to enforce code formatting.
    * **Action:** Add a `cargo clippy -- -D warnings` step to enforce strict linting rules.
    * **Action:** Add a `cargo test --workspace` step to run all tests.
    * **Action:** (Optional) Add a `cargo-deny` step to check for security vulnerabilities and license compliance.
    * **Deliverable:** A GitHub Actions workflow that automatically runs on every push and pull request.

  * **Subtask 1.4: Setup Documentation**
    * **Action:** Create a `docs` directory at the root of the workspace.
    * **Action:** Initialize an `mdbook` project within it.
    * **Action:** Create placeholder pages in `SUMMARY.md` for the planned architecture, concepts, and each crate's API.
    * **Action:** Add a GitHub Actions workflow to build and deploy the `mdbook` to GitHub Pages on every merge to `main`.
    * **Deliverable:** A live, albeit empty, documentation site hosted at `riglr-project.github.io/riglr`.

* **Epic 2: `riglr-core` - The Resilient Executor**
  * **Subtask 2.1: Design Core Data Structures**
    * **Action:** Define `Job` struct with fields: `job_id (Uuid)`, `tool_name (String)`, `params (serde_json::Value)`, `idempotency_key (Option<String>)`, `max_retries (u32)`.
    * **Action:** Define `JobResult` enum with variants: `Success { value: serde_json::Value, tx_hash: Option<String> }`, `Failure { error: String, retriable: bool }`.
    * **Action:** Ensure both structs derive `Serialize` and `Deserialize`.
    * **Deliverable:** A `jobs.rs` module with stable, well-documented data structures.

  * **Subtask 2.2: Implement `JobQueue` Trait**
    * **Action:** Define an `async_trait` `JobQueue` with methods: `enqueue(&self, job: Job) -> Result<()>` and `dequeue(&self) -> Result<Option<Job>>`.
    * **Action:** Implement `RedisJobQueue` using `redis-rs`. Use `LPUSH` for enqueueing and `BRPOP` for a blocking dequeue, ensuring workers don't spin needlessly.
    * **Deliverable:** A generic queue abstraction and a production-ready Redis implementation.

  * **Subtask 2.3: Create the `ToolWorker`**
  * **Action:** Create a `ToolWorker<Q: JobQueue>` struct that takes a `JobQueue` and a `rig::ToolSet` in its constructor. Keep `Q` generic to allow swapping queue backends.
    * **Action:** Implement a `run` method that enters an infinite loop, calling `dequeue` to get jobs.
    * **Action:** In the loop, look up the appropriate tool from the `ToolSet` using `job.tool_name`.
    * **Action:** Deserialize `job.params` into the tool's `Args` type.
    * **Action:** Call the tool's `execute` method.
    * **Deliverable:** A functioning worker that can process jobs from the queue.

  * **Subtask 2.4-2.8: Implement Resilience in the Worker**
  * **Action (Idempotency):** Before executing a mutating tool, check if `idempotency_key` exists. If so, check the idempotency store for a cached result. If found, return it. If not, proceed.
    * **Action (Concurrency):** Wrap the tool execution logic in a `tokio::sync::Semaphore` permit acquisition, with separate semaphores for different resource types (e.g., "solana_rpc", "alchemy_api").
    * **Action (Timeout):** Wrap the `tool.execute()` call in `tokio::time::timeout`.
  * **Action (Retries):** Wrap the entire execution block (timeout included) in a `tokio-retry` loop, checking the `retriable` flag on the `JobResult::Failure`.
  * **Action (Idempotency):** After a successful mutating tool call, store the result against the `idempotency_key` with an expiry. Document Redis as the default impl and include an in-memory map for tests/examples.
  * **Action (Alt Queue):** Provide both `RedisJobQueue` (production) and `InMemoryJobQueue` (tests/dev). Document how to swap via the generic `Q`.
  * **Action (Docs):** Add a docs page describing the idempotency store and queue backends, including a Docker Compose snippet for Redis for local development.
    * **Deliverable:** A `ToolWorker` that robustly handles network failures, slow responses, and prevents duplicate transactions.

* **Epic 3: `riglr-macros` - The DX Enhancer**
  * **Subtask 3.1: Design the `#[tool]` Macro**
    * **Action:** Define the macro to work on `async fn` and structs.
    * **Action:** For functions, arguments will be parsed to define the tool's parameters. For structs, it will assume an `execute` method.
    * **Deliverable:** A clear design document for the macro's capabilities and syntax.

  * **Subtask 3.2: Implement Doc Comment Parsing**
    * **Action:** Use `syn` and `quote` crates. Parse the `#[doc]` attributes on the function/struct for the main `description`.
    * **Action:** Parse `#[doc]` attributes on function arguments for parameter descriptions.
    * **Deliverable:** A macro that can extract documentation into variables.

  * **Subtask 3.3: Integrate `schemars`**
    * **Action:** The macro will require function arguments to derive `JsonSchema`.
    * **Action:** In the macro's expansion, call `schemars::schema_for!` to generate the schema.
    * **Action:** Serialize the generated schema to a `serde_json::Value`.
    * **Deliverable:** The macro can now generate a type-safe JSON schema for any tool.

  * **Subtask 3.4: Generate `impl rig::Tool`**
    * **Action:** Use the `quote!` macro to generate the full implementation block.
    * **Action:** The `definition` method will be populated with the parsed doc comments and the generated schema.
    * **Action:** The `call` method will simply delegate to the original annotated function.
    * **Deliverable:** A working `#[tool]` macro that drastically reduces boilerplate.

  * **Subtask 3.5: Testing and Documentation**
    * **Action:** Create a `tests` module within the macro crate.
    * **Action:** Use `trybuild` to create compile-time tests that ensure the macro expands correctly and fails on invalid input.
    * **Action:** Create an `examples` directory in the workspace showing how to use the macro on different function signatures.
    * **Deliverable:** A well-tested and documented macro ready for use in Phase 2.

---

#### **Phase 2: Core On-Chain Toolsets (Weeks 4-6)**

*Goal: Deliver the most valuable, reusable components for on-chain interaction. This phase focuses on creating two flagship crates that will immediately attract crypto-native developers.*

* **Epic 4: `riglr-solana-tools` Crate**
  * **Subtask 4.1: Setup Crate Structure**
  * **Action:** Create the `Cargo.toml` with necessary dependencies (`solana-sdk`, `solana-client`, `spl-token`, `riglr-core`, `riglr-macros`, etc.).
    * **Action:** Establish the module structure: `src/lib.rs`, `src/balance.rs`, `src/transaction.rs`, `src/swap.rs`.
    * **Deliverable:** A compilable, empty crate within the workspace.

  * **Subtask 4.2: Implement Read-Only Tools**
    * **Action:** In `balance.rs`, implement `get_sol_balance` and `get_spl_token_balance` as `async fn`s.
    * **Action:** Annotate them with the `#[tool]` macro, including clear doc comments for automatic description generation.
    * **Action:** Implement `get_latest_blockhash` and `get_transaction_details`.
    * **Deliverable:** A set of functional, documented, and tested read-only tools.

  * **Subtask 4.3: Implement State-Mutating Tools**
    * **Action:** In `transaction.rs`, implement `transfer_sol` and `transfer_spl_token`.
    * **Action:** These tools will not execute transactions directly. Instead, they will construct the transaction, serialize it, and enqueue it as a `Job` for the `ResilientExecutor`.
    * **Action:** The tool's return value will be the `job_id`.
    * **Deliverable:** Secure, non-blocking tools for basic on-chain mutations.

  * **Subtask 4.4: Implement Jupiter Swap Tool**
    * **Action:** In `swap.rs`, port the Jupiter swap logic from `listen-kit`.
    * **Action:** Create `get_jupiter_quote` as a read-only tool.
    * **Action:** Create `perform_jupiter_swap` as a state-mutating tool that enqueues a job for the executor.
    * **Deliverable:** A powerful DeFi tool for Solana agents.

  * **Subtask 4.5: Integration Testing**
    * **Action:** Set up an integration test suite in the `tests/` directory.
    * **Action:** Use `solana-test-validator` or a public devnet RPC for testing.
    * **Action:** Write tests that build a `rig::Agent`, equip it with the new tools, and prompt it to perform actions (e.g., "what is the SOL balance of address X?").
    * **Deliverable:** A CI-passing test suite that validates the end-to-end functionality of each tool within an agent context.

  * **Subtask 4.6: Create Examples**
    * **Action:** Create `examples/simple_balance_agent.rs` showing how to build an agent that only checks balances.
    * **Action:** Create `examples/trading_agent.rs` showing how to compose the quote and swap tools.
    * **Deliverable:** Clear, runnable examples that serve as practical documentation.

* **Epic 5: `riglr-evm-tools` Crate**
  * **Subtask 5.1: Setup Crate Structure**
    * **Action:** Create `Cargo.toml` with dependencies (`ethers` to start for maturity/examples; plan optional `alloy` feature later), `riglr-core`, `riglr-macros`, etc.
    * **Action:** Establish module structure: `src/lib.rs`, `src/balance.rs`, `src/transaction.rs`, `src/swap.rs`.
    * **Deliverable:** A compilable, empty crate.

  * **Subtask 5.2: Implement Read-Only Tools**
    * **Action:** Implement `get_eth_balance`, `get_erc20_balance`, and a generic `read_contract` tool using the `#[tool]` macro.
    * **Deliverable:** A set of functional, documented, and tested read-only EVM tools.

  * **Subtask 5.3: Implement State-Mutating Tools**
    * **Action:** Implement `transfer_eth`, `transfer_erc20`, and `approve_token` tools that enqueue jobs for the `ResilientExecutor`.
    * **Deliverable:** Secure, non-blocking tools for basic EVM mutations.

  * **Subtask 5.4: Implement Uniswap V3 Swap Tool**
    * **Action:** Port the Uniswap logic from `listen-kit`.
    * **Action:** Create `get_uniswap_quote` and `perform_uniswap_swap` tools.
    * **Deliverable:** A powerful DeFi tool for EVM agents.

  * **Subtask 5.5: Integration Testing**
    * **Action:** Set up an integration test suite using a local node like Anvil.
    * **Action:** Write tests that build a `rig::Agent` and prompt it to perform EVM actions.
    * **Deliverable:** A CI-passing test suite for all EVM tools.

  * **Subtask 5.6: Create Examples**
    * **Action:** Create `examples/cross_chain_balance_agent.rs` that uses tools from both `riglr-solana-tools` and `riglr-evm-tools`.
    * **Action:** Create `examples/uniswap_agent.rs`.
    * **Deliverable:** Clear, runnable examples demonstrating EVM and cross-chain capabilities.

---

#### **Phase 3: Advanced Features & Data Integrations (Weeks 7-9)**

*Goal: Build higher-level abstractions that provide unique value, focusing on the sophisticated graph memory system and off-chain data tools.*

* **Epic 6: `riglr-graph-memory` Crate**
  * **Subtask 6.1: Port Core Graph Logic**
    * **Action:** Move the Neo4j client and entity/relation logic from `listen-memory/src/graph` into the new crate.
    * **Action:** Abstract the database client behind a trait to allow for future backends (e.g., TigerGraph).
    * **Deliverable:** A standalone crate for creating and querying the on-chain knowledge graph.

  * **Subtask 6.2: Implement `rig::VectorStore` Trait**
    * **Action:** Create a `GraphRetriever` struct.
    * **Action:** Implement the `VectorStore` trait for this struct. The `top_n` method will execute the vector search Cypher query against the Neo4j index.
    * **Action:** The `add_documents` method will be responsible for the complex task of distilling entities/relations from text and inserting them into the graph.
    * **Deliverable:** A `rig`-compatible vector store that uses a knowledge graph as its backend.

  * **Subtask 6.3: Implement `rig::Embed` Trait**
    * **Action:** Define a `RawTextDocument` struct that holds unstructured text.
    * **Action:** Implement the `Embed` trait for this struct. The `embed` method will contain the logic to call the distiller agent (from `listen-memory`) to extract entities and relations.
    * **Deliverable:** An easy-to-use interface for adding unstructured data to the graph memory.

  * **Subtask 6.4: Create RAG Agent Example**
    * **Action:** Create `examples/wallet_analyst_rag.rs`.
    * **Action:** The example will first populate the graph with sample transaction data.
    * **Action:** It will then create a `rig::Agent` using the `GraphRetriever` as its dynamic context.
    * **Action:** The agent will be prompted with questions like "What protocols has this wallet interacted with?".
    * **Deliverable:** A powerful demonstration of the graph memory's capabilities.

  * **Subtask 6.5: Document Data Model**
    * **Action:** In the crate's documentation, clearly define the node and relationship schema (e.g., `(User)-[:PERFORMED]->(Transaction)-[:INVOLVED]->(Token)`).
    * **Action:** Provide Cypher queries and code examples for populating the graph.
    * **Deliverable:** Clear documentation enabling developers to use and extend the graph memory.

* **Epic 7: `riglr-web-tools` Crate**
  * **Subtask 7.1: Port Web Data Tools**
    * **Action:** Move the client and tool logic for DexScreener, Twitter, and the Exa web search API from `listen-kit` into separate modules within the new crate. Use `web-tools` name for clarity; if expanded to local DB/files later, reassess naming.
    * **Deliverable:** A crate containing clients for various off-chain data sources.

  * **Subtask 7.2: Refactor into `rig::Tool`s**
    * **Action:** Apply the `#[tool]` macro to all public functions (e.g., `search_tweets`, `search_web`).
    * **Action:** Ensure all argument and return types are `serde`-compatible and have `JsonSchema`.
    * **Deliverable:** A set of easy-to-use, `rig`-native tools for off-chain data.

  * **Subtask 7.3: Create "Market Analyst" Example**
    * **Action:** Create `examples/market_analyst.rs`.
  * **Action:** Build an agent that combines tools from `riglr-web-tools` (e.g., `search_tweets` for sentiment) and `riglr-solana-tools` (e.g., `get_token_balance` for volume).
    * **Action:** Prompt the agent with a complex query like "What's the current sentiment on Twitter for $WIF, and does its on-chain volume support this trend?".
    * **Deliverable:** A compelling example showcasing the power of composing tools from different `riglr` crates.

---

#### **Phase 4: Ecosystem, Documentation & Launch (Weeks 10-12)**

*Goal: Polish the entire ecosystem, create a frictionless onboarding experience for new developers, and execute a successful public launch.*

* **Epic 8: `create-riglr-app` Starter Template**
  * **Subtask 8.1: Create `cargo-generate` Template**
    * **Action:** Set up a new repository named `create-riglr-app`.
    * **Action:** Use `cargo-generate` conventions, including a `cargo-generate.toml` file for templating variables (e.g., project name, default chain).
    * **Deliverable:** A template that can be used via `cargo generate riglr-project/create-riglr-app`.

  * **Subtask 8.2: Include Example Agent**
    * **Action:** The template's `main.rs` will contain a simple, well-commented agent.
  * **Action:** The agent will be configured to use one tool from `riglr-solana-tools` and one from `riglr-web-tools` to demonstrate composition.
    * **Action:** Include a `.env.example` file with all necessary API keys.
    * **Deliverable:** A "hello world" agent that works out of the box.

  * **Subtask 8.3: Write Template `README.md`**
    * **Action:** Write a clear, concise README explaining how to configure the `.env` file and run the example.
    * **Action:** Include "Next Steps" section pointing to the main `riglr` documentation.
    * **Deliverable:** A self-contained, easy-to-follow starter project.

* **Epic 9: Documentation & Launch**
  * **Subtask 9.1: Finalize `mdbook` Documentation**
    * **Action:** Write the "Getting Started" tutorial using the `create-riglr-app` template.
    * **Action:** Write detailed "Concepts" pages explaining the Resilient Executor and Graph Memory.
    * **Action:** Write API reference pages for each tool in the toolset crates, with copy-pasteable examples.
    * **Action:** Review and polish all content for clarity and accuracy.
    * **Deliverable:** A complete, high-quality documentation site.

  * **Subtask 9.2: Publish Crates**
  * **Action:** Ensure all `Cargo.toml` files have correct metadata (authors, license, repository, documentation links) and include `readme`, `keywords`, and `categories` for crates.io discoverability.
    * **Action:** Perform a final check with `cargo-deny`.
  * **Action:** Publish all crates to `crates.io` in the correct dependency order, starting with `riglr-core` and `riglr-macros`.
    * **Deliverable:** All `riglr` crates are publicly available.

  * **Subtask 9.3: Build Showcase Application**
    * **Action:** Create a new repository `riglr-showcase-bot` (aka `riglr-showcase`).
    * **Action:** Build a Discord bot that allows users to interact with a powerful `riglr` agent. The bot should support commands like `/balance <address>`, `/analyze <token>`, and `/ask <question about wallet>`.
    * **Action:** Deploy the bot publicly so potential users can interact with it.
    * **Deliverable:** A live, interactive demonstration of the `riglr` ecosystem's capabilities.

  * **Subtask 9.4: Launch Campaign**
    * **Action:** Write a comprehensive blog post for the `riglr` website/Medium, explaining the vision, the problems it solves, and how to get started.
    * **Action:** Create a post for Twitter/X, announcing the launch and linking to the blog post, GitHub, and showcase bot.
    * **Action:** Post the announcement on relevant communities (e.g., `/r/rust`, `/r/solana`, crypto dev Discords).
    * **Deliverable:** A coordinated public launch to attract the initial wave of users and contributors.

---

### **7. Appendix: Example Requirements File (`Cargo.toml` for `riglr-solana-tools`)**

```toml
[package]
name = "riglr-solana-tools"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "A suite of rig-compatible tools for interacting with the Solana blockchain."
repository = "https://github.com/riglr-project/riglr"
readme = "README.md"
keywords = ["solana", "ai", "agent", "blockchain", "rig"]
categories = ["cryptography::cryptocurrencies", "web-programming::api-bindings"]
documentation = "https://docs.rs/riglr-solana-tools"

[dependencies]
# Core riglr and rig dependencies
rig-core = "0.2.0" # Or latest version
riglr-core = { path = "../riglr-core" }
riglr-macros = { path = "../riglr-macros" }

# Async and Error Handling
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
async-trait = "0.1"

# Serialization and Schema
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = "0.8"

# Solana Dependencies
solana-sdk = "1.18"
solana-client = "1.18"
spl-token = "4.0"

# Other utilities
# ...

[dev-dependencies]
# For testing
tokio-test = "0.4"
```
