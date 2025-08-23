### From Brain to Blockchain: A Modern Architecture for AI Agents

This document outlines the end-to-end architectural flow of the `rig` and `riglr` ecosystem, detailing how a natural language command from a user is transformed into a verified transaction on the blockchain. The system is designed with a clean separation of concerns, distinguishing between the AI-driven decision-making "brain" and the blockchain-interaction "body."

#### **Core Concepts**

* **The `rig` Ecosystem (The Brain):** This layer is responsible for understanding natural language, making decisions, and selecting the appropriate tools for a given task. It interfaces with Large Language Models (LLMs) to translate user intent into structured, executable commands.
* **The `riglr` Ecosystem (The Body):** This layer acts as the execution engine. It takes the structured commands from the `rig` brain and securely interacts with the blockchain to perform operations. It manages infrastructure, cryptographic signers, and the actual execution of on-chain transactions.
* **Tools (The Hands):** These are specific, predefined functions that can be called to perform an action, such as `transfer_sol` or `get_sol_balance`. They are described to the "brain" and executed by the "body."

---

### **Architectural Blueprint**

The following diagram illustrates the complete lifecycle of a single request, from user input to a confirmed blockchain transaction.

```
========================================================================================================================
|   riglr-agents Crate (The Factory Supervisor & Worker)                                                               |
|                                                                                                                      |
|   [ User Prompt ]                                                                                                    |
|        |                                                                                                             |
|        | 1. Task (e.g., "Send 0.01 SOL to...")                                                                       |
|        v                                                                                                             |
|   +---------------------+                                                                                            |
|   | AgentDispatcher   |                                                                                            |
|   +---------------------+                                                                                            |
|        |                                                                                                             |
|        | 2. Finds capable agent                                                                                      |
|        v                                                                                                             |
|   +---------------------+      +----------------------------------------------------------------------------------+  |
|   | AgentRegistry     |----->| riglr_agents::Agent (LiveToolAgent)                                                |  |
|   +---------------------+      |                                                                                  |  |
|                              |   (The Orchestrator / The "Body")                                                |  |
|                              |                                                                                  |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |   |  rig-core Crate (The "Brain")                                              |   |  |
|                              |   |                                                                            |   |  |
|                              |   |   +---------------------+      4. Prompt      +------------------------+  |   |  |
|                              |   |   | rig::agent::Agent   |--------------------->|   LLM Provider (API)   |  |   |  |
|                              |   |   +---------------------+      (w/ Tool Schemas) +------------------------+  |   |  |
|                              |   |             ^                                              |               |   |  |
|                              |   |             | 5. JSON Tool Call ("transfer_sol", {..})     |               |   |  |
|                              |   |             +----------------------------------------------+               |   |  |
|                              |   |                                                                            |   |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |                                      |                                          |  |
|                              | 6. Receives Tool Call               | 7. Creates Job                           |  |
|                              |                                      v                                          |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |   |  riglr-core Crate (The Execution Engine / The "Foundation")                |   |  |
|                              |   |                                                                            |   |  |
|                              |   |   +---------------------+      8. process_job()      +-----------------+ |   |  |
|                              |   |   |    ToolWorker       |--------------------------->| riglr_core::Tool| |   |  |
|                              |   |   +---------------------+                            +-----------------+ |   |  |
|                              |   |             ^                                              |               |   |  |
|                              |   |             | 13. JobResult (w/ signature)                 | 9. execute()  |   |  |
|                              |   |             +----------------------------------------------+               |   |  |
|                              |   |                                                                            |   |  |
|                              |   +----------------------------------------------------------------------------+   |  |
|                              |                                                                                  |  |
|                              +----------------------------------------------------------------------------------+  |
|        ^                                                                                                             |
|        | 14. TaskResult                                                                                            |
|        +-----------------------------------------------------------------------------------------------------------+
|                                                                                                                      |
========================================================================================================================
                                                               |
                                                               | 10. Uses Contexts
                                                               v
+----------------------------------------------------------------------------------------------------------------------+
| riglr-core Contexts                                                                                                  |
|                                                                                                                      |
| +----------------------+     +-----------------------+     +-------------------------------------------------------+  |
| | ApplicationContext   |     |    SignerContext      |     | riglr-solana-tools Crate (The "Hands")                |  |
| |                      |     |                       |     |                                                       |  |
| | - RPC Client         |---->| - Secure Signer       |---->|   #[tool]                                             |  |
| | - Config             | 11. |                       | 12. |   async fn transfer_sol(...) { ... }                  |  |
| +----------------------+     +-----------------------+     |       |                                               |  |
|                                                            |       | 13. Sends REAL Transaction                    |  |
|                                                            |       v                                               |  |
|                                                            |   [ Blockchain ]                                      |  |
|                                                            +-------------------------------------------------------+  |
+----------------------------------------------------------------------------------------------------------------------+
```

---

### **Request Lifecycle: A Step-by-Step Analysis**

The following phases describe the journey of a single request through the system.

#### **Phase 1: Task Reception and Dispatch**

1. **Task Creation:** A user or an automated system initiates a `Task` containing a natural language `description`, such as "Send 0.01 SOL to...".
2. **Agent Dispatch:** The `AgentDispatcher` receives the `Task`. It queries the `AgentRegistry` to identify an agent that possesses the required capabilities (e.g., `live_solana_operations`).
3. **Agent Selection:** The `AgentRegistry` identifies and provides the appropriate agent, in this case, the `LiveToolAgent`, to the `Dispatcher`.

#### **Phase 2: Orchestration and AI-driven Decision Making**

4. **Prompting the Brain:** The selected `LiveToolAgent` (the "Body") forwards the natural language prompt to its internal `rig::Agent` instance (the "Brain"). Crucially, it also supplies the schemas of all available tools it can use, such as `get_sol_balance` and `transfer_sol`.
5. **AI Processing:** The `rig::Agent` makes an API call to a Large Language Model (LLM). The LLM analyzes the prompt and the provided tool schemas to determine the most suitable tool and its required parameters.
6. **Structured Response:** The LLM returns a structured JSON response, for example: `{"tool_name": "transfer_sol", "parameters": {"to_address": "...", "amount_sol": 0.01}}`. The `rig::Agent` parses this into a `ToolCall` object.

#### **Phase 3: Secure Execution Handoff**

7. **Job Creation:** The `LiveToolAgent` receives the `ToolCall` from its "brain." Instead of executing the tool directly, it translates the `ToolCall` into a generic `Job` object, which contains the tool's name and arguments.
8. **Delegation to Worker:** The agent delegates the `Job` to the `ToolWorker` by calling `self.tool_worker.process_job(job)`. This is a critical handoff to the `riglr-core` execution engine. The agent's orchestration role for this request is now complete.

#### **Phase 4: Blockchain Interaction**

9. **Tool Identification:** The `ToolWorker` inspects the `job.tool_name` (e.g., "transfer_sol") and locates the corresponding `riglr_core::Tool` that was registered during its initialization.
10. **Tool Execution:** The `ToolWorker` invokes the `execute` method on the identified tool, passing the JSON arguments from the job.
11. **Accessing Application Context:** The `transfer_sol` function accesses the `ApplicationContext` to retrieve shared resources, such as the read-only RPC client.
12. **Accessing Signer Context:** To authorize the transaction, the function securely retrieves the isolated signer by calling `SignerContext::current_as_solana()`.
13. **On-Chain Transaction:** The tool constructs the transaction, signs it using the secure signer, and broadcasts it to the Solana network. It then receives a transaction signature as confirmation.

#### **Phase 5: Result Propagation**

14. **Returning the Result:** The transaction signature is propagated back up the call stack. It flows from the tool to the `ToolWorker` as a `JobResult`, which is then converted into a `TaskResult` by the `LiveToolAgent` and finally returned to the original caller.

---

### **Component Responsibilities**

This architecture establishes a clear separation of concerns, ensuring modularity and scalability.

| Ecosystem | Component                | Primary Responsibility                                                              |
| :-------- | :----------------------- | :---------------------------------------------------------------------------------- |
| **`rig`** | `rig::agent::Agent`      | **Decides what to do** by interpreting natural language and selecting a tool via an LLM. |
| **`rig`** | `rig::tool::Tool`        | **Describes a capability** and its parameters to the AI brain.                      |
|           |                          |                                                                                     |
| **`riglr`** | `riglr_agents::Agent`    | **Orchestrates the workflow** by communicating with the brain and delegating to the worker. |
| **`riglr`** | `riglr_core::ToolWorker` | **Executes the requested work** by finding and running the appropriate concrete tool. |
| **`riglr`** | `riglr_core::Tool`       | **Defines how to perform the work** in a standardized way for the execution engine. |
| **`riglr`** | `riglr-solana-tools`     | **Provides the specific implementations** (the "hands") that interact with the blockchain. |

### The `#[tool]` Macro: Bridging the Ecosystems

The `#[tool]` macro is the key component that unifies the `rig` and `riglr` ecosystems. By applying this macro to a single function, it simultaneously implements both the `rig::tool::Tool` trait (for description and AI understanding) and the `riglr_core::Tool` trait (for execution). This dual implementation allows a function to be understood by the AI "brain" and executed by the "body" without code duplication, creating a seamless and powerful architecture.
