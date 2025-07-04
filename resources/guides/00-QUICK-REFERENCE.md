# 🚀 Hyperware Quick Reference for AI Models

## Critical Rules - MUST FOLLOW

### 1. HTTP Endpoints can have `_request_body` parameter, but does not need it.
```rust
// Can cause HTTP requests to be routed here
#[http]
async fn get_data(&self) -> Vec<Data> { }

// ✅ CORRECT - Even if unused, parameter helps route the request properly
#[http]
async fn get_data(&self, _request_body: String) -> Vec<Data> { }
```

### 2. Frontend MUST Include `/our.js` Script
```html
<head>
    <!-- ⚠️ CRITICAL: This must be FIRST, before any other scripts -->
    <script src="/our.js"></script>
    <!-- Other scripts go here -->
</head>
```

### 3. Multi-Parameter Calls Use TUPLES
```typescript
// ❌ WRONG - Object format
{ "MethodName": { param1: "a", param2: "b" } }

// ✅ CORRECT - Tuple/array format
{ "MethodName": ["a", "b"] }
```

### 4. Remote Calls MUST Set Timeout
```rust
// ❌ WRONG - No timeout
Request::new()
    .target(address)
    .body(data)
    .send();

// ✅ CORRECT - Always set expects_response
Request::new()
    .target(address)
    .body(data)
    .expects_response(30)  // 30 second timeout
    .send_and_await_response(30);
```

### 5. WIT-Compatible Types Only
```rust
// ✅ ALLOWED
String, bool, u8-u64, i8-i64, f32, f64
Vec<T>, Option<T>
Simple structs with public fields

// ❌ NOT ALLOWED
HashMap → use Vec<(K,V)>
[T; N] → use Vec<T>
Complex enums → use simple enums + separate data

// 🔥 ESCAPE HATCH: Return JSON strings
#[http]
async fn get_complex(&self, _request_body: String) -> String {
    serde_json::to_string(&self.complex_data).unwrap()
}
```

## Build Commands

```bash
# First time build (installs dependencies)
kit bs --hyperapp

# Regular build
kit b --hyperapp

# Clean rebuild
rm -rf target/ ui/node_modules ui/dist pkg/
kit b --hyperapp
```

**Note**: `kit b --hyperapp` automatically generates `pkg/manifest.json`

## Project Structure
```
skeleton-app/
├── Cargo.toml           # Workspace config
├── metadata.json        # App metadata
├── skeleton-app/        # Rust backend
│   ├── Cargo.toml      # DO NOT add hyperware_process_lib here!
│   └── src/
│       └── lib.rs      # Main app logic
├── ui/                 # React frontend
│   ├── index.html      # MUST have <script src="/our.js">
│   └── src/
│       ├── App.tsx     # Main component
│       ├── store/      # Zustand state
│       └── utils/      # API calls
└── pkg/               # Build output (generated)
```

## Common Patterns

### Basic HTTP Endpoint
```rust
#[http]
async fn my_endpoint(&mut self, request_body: String) -> Result<String, String> {
    // Parse request if needed
    let req: MyRequest = serde_json::from_str(&request_body)?;
    
    // Update state
    self.data.push(req.value);
    
    // Return response
    Ok("Success".to_string())
}
```

### Remote P2P Endpoint
```rust
#[remote]
async fn handle_remote(&mut self, data: String) -> Result<String, String> {
    // Process remote request
    Ok("Acknowledged".to_string())
}
```

### Frontend API Call
```typescript
// utils/api.ts
export async function callEndpoint(data: any) {
    const response = await fetch('/api', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            MyEndpoint: data  // Single param
            // OR for multiple: ["param1", "param2"]
        })
    });
    return response.json();
}
```

### P2P Call to Another Node
```rust
// Construct address
let process_id = "app-name:package-name:publisher.os"
    .parse::<ProcessId>()?;
let target = Address::new(remote_node, process_id);

// Make request
let wrapper = json!({ "RemoteMethod": params });
let result = Request::new()
    .target(target)
    .body(serde_json::to_vec(&wrapper).unwrap())
    .expects_response(30)
    .send_and_await_response(30)?;
```

## Import Requirements

### Rust Backend
```rust
use hyperprocess_macro::*;
use hyperware_process_lib::{
    our, Address, ProcessId, Request,
    homepage::add_to_homepage
};
use serde::{Deserialize, Serialize};
```

### Cargo.toml Dependencies
```toml
[dependencies]
anyhow = "1.0"
process_macros = "0.1"  # Required for macro
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wit-bindgen = "0.36.0"

# ⚠️ DO NOT ADD hyperware_process_lib - provided by macro

[dependencies.hyperprocess_macro]
git = "https://github.com/hyperware-ai/hyperprocess-macro"
rev = "47400ab"

[dependencies.caller-utils]
path = "../target/caller-utils"
```

## Error Quick Fixes

| Error | Fix |
|-------|-----|
| "Failed to deserialize HTTP request" | Add `_request_body: String` parameter |
| "Node not connected" | Add `<script src="/our.js">` to HTML |
| "hyperware_process_lib is ambiguous" | Remove it from Cargo.toml |
| "Type not found in WIT" | Return as JSON string instead |
| "ProcessId parse error" | Check format: "name:package:publisher" |
| "failed to open manifest.json" | Run `kit b --hyperapp` to generate |
| "does not have capability" | Add to manifest.json request_capabilities |

## Testing P2P

```bash
# Terminal 1
kit s --fake-node alice.os

# Terminal 2
kit s --fake-node bob.os

# Install app on both, test communication
```

## Remember

1. **Always** include `_request_body` in HTTP endpoints
2. **Always** include `/our.js` in HTML
3. **Always** use tuples for multi-param calls
4. **Always** set timeout on remote requests
5. **Never** add hyperware_process_lib to Cargo.toml
6. **When in doubt**, return JSON strings
7. **Check** [Manifest Guide](./08-MANIFEST-AND-DEPLOYMENT.md) for deployment
8. **Review** [Capabilities Guide](./09-CAPABILITIES-GUIDE.md) for permissions