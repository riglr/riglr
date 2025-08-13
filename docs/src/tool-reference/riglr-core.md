# riglr-core

*This page is auto-generated from the source code.*

Core utilities and abstractions used across all riglr crates.

## Available Tools

### SignerContext Management

#### `set_signer_context`
**Description:**  
Sets the current signer context for blockchain operations.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `signer` | `Arc<dyn TransactionSigner>` | Yes | The signer to use for transactions |

**Returns:** `Result<(), ToolError>`

**Example:**
```rust
let signer = Arc::new(LocalSolanaSigner::from_env()?);
set_signer_context(signer).await?;
```

---

#### `get_current_signer`
**Description:**  
Retrieves the current signer from the context.

**Parameters:** None

**Returns:** `Result<Arc<dyn TransactionSigner>, ToolError>`

**Example:**
```rust
let signer = get_current_signer().await?;
let pubkey = signer.pubkey();
```

---

### Job Processing

#### `enqueue_job`
**Description:**  
Adds a job to the processing queue.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `job_type` | `String` | Yes | Type of job to enqueue |
| `payload` | `serde_json::Value` | Yes | Job payload data |
| `priority` | `u8` | No | Job priority (0-255, default: 100) |

**Returns:** `Result<String, ToolError>` (Job ID)

**Example:**
```rust
let job_id = enqueue_job(
    "swap_tokens".to_string(),
    json!({ "from": "SOL", "to": "USDC", "amount": 1.0 }),
    Some(150)
).await?;
```

---

#### `get_job_status`
**Description:**  
Checks the status of a queued job.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `job_id` | `String` | Yes | ID of the job to check |

**Returns:** `Result<JobStatus, ToolError>`

**Example:**
```rust
let status = get_job_status(job_id).await?;
match status {
    JobStatus::Completed(result) => println!("Done: {:?}", result),
    JobStatus::Failed(error) => println!("Failed: {}", error),
    JobStatus::Pending => println!("Still processing..."),
}
```

---

### Error Utilities

#### `classify_error`
**Description:**  
Classifies an error as retriable or permanent.

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `error` | `Box<dyn Error>` | Yes | The error to classify |

**Returns:** `ErrorClassification`

**Example:**
```rust
match classify_error(error) {
    ErrorClassification::Retriable => {
        // Retry with exponential backoff
    },
    ErrorClassification::Permanent => {
        // Fail immediately
    }
}
```

---

## Type Definitions

### `JobStatus`
```rust
enum JobStatus {
    Pending,
    Processing,
    Completed(serde_json::Value),
    Failed(String),
}
```

### `ErrorClassification`
```rust
enum ErrorClassification {
    Retriable,
    Permanent,
}
```

## Error Handling

All tools in this crate follow the standard error pattern:
- Network and timeout errors are marked as **retriable**
- Configuration and validation errors are marked as **permanent**

See [Error Handling Philosophy](../concepts/error-handling.md) for details.