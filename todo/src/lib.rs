// HYPERWARE SKELETON APP
// This is a minimal, well-commented skeleton app for the Hyperware platform
// using the Hyperapp framework (macro-driven approach).

// CRITICAL IMPORTS - DO NOT MODIFY THESE
// The hyperprocess_macro provides everything you need including:
// - Async/await support (custom runtime)
// - Automatic WIT (WebAssembly Interface Types) generation
// - State persistence
// - HTTP/WebSocket bindings
use hyperprocess_macro::*;

use hyperware_process_lib::http::server::{send_ws_push, WsMessageType};
use hyperware_app_common::{get_server, source};
use hyperware_process_lib::{LazyLoadBlob, Address, homepage::add_to_homepage, our};
// you can use these imports when using P2P features from the hyperware_process_lib:
// Address,                // For P2P addressing
// ProcessId,              // Process identifiers
// Request,                // For making requests to other processes/nodes
use hyperware_process_lib::logging::{error, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid; 

// =============================================================================
// CORE TODO APPLICATION DATA STRUCTURES
// =============================================================================

/// Core todo item with unique ID, text content, and completion status
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct TodoItem {
    id: String,
    text: String,
    completed: bool,
}

/// Legacy response structure (kept for compatibility)
#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub data: NestedData,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct NestedData {
    pub items: Vec<Item>,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
}

fn ws_get_tasks(channel_id: u32, tasks: Vec<TodoItem>) {
    let response = serde_json::json!({
        "type": "tasks_overview",
        "tasks": tasks
    });

    let response_bytes = response.to_string().into_bytes();

    let response_blob = LazyLoadBlob {
        mime: Some("application/json".to_string()),
        bytes: response_bytes,
    };
    send_ws_push(channel_id, WsMessageType::Text, response_blob);
}

fn ws_add_task(channel_id: u32, task: TodoItem, tasks: Vec<TodoItem>) {

    let response = serde_json::json!({
        "type": "task_added",
        "task": task,
        "tasks": tasks
    });

    let response_bytes = response.to_string().into_bytes();

    let response_blob = LazyLoadBlob {
        mime: Some("application/json".to_string()),
        bytes: response_bytes,
    };
    send_ws_push(channel_id, WsMessageType::Text, response_blob);
}

fn ws_toggle_task(channel_id: u32, task: TodoItem, tasks: Vec<TodoItem>) {

    let response = serde_json::json!({
        "type": "task_toggled",
        "task": task,
        "tasks": tasks
    });

    let response_bytes = response.to_string().into_bytes();

    let response_blob = LazyLoadBlob {
        mime: Some("application/json".to_string()),
        bytes: response_bytes,
    };
    send_ws_push(channel_id, WsMessageType::Text, response_blob);
}

fn ws_ack(channel_id: u32) {
    let response = serde_json::json!({
        "type": "ack"
    });

    let response_bytes = response.to_string().into_bytes();

    let response_blob = LazyLoadBlob {
        mime: Some("application/json".to_string()),
        bytes: response_bytes,
    };
    send_ws_push(channel_id, WsMessageType::Text, response_blob);
}

// =============================================================================
// APPLICATION STATE
// =============================================================================

// STEP 1: DEFINE YOUR APP STATE
// This struct holds all persistent data for your app
// It MUST derive Default, Serialize, and Deserialize
// Add PartialEq if you use this type in WIT interfaces
#[derive(PartialEq, Clone, Default, Debug, Serialize, Deserialize)]
pub struct TodoState {
    /// List of todo tasks
    tasks: Vec<TodoItem>,
    /// Active WebSocket channel IDs (not serialized)
    #[serde(skip)]
    ws_channels: HashSet<u32>,
    // add clients
    clients: Vec<Address>,
}

// =============================================================================
// HYPERPROCESS CONFIGURATION
// =============================================================================

#[hyperprocess(
    name = "todo",
    ui = Some(HttpBindingConfig::default()),
    // HTTP API endpoints - MUST include /api for frontend communication
    // Any endpoint referenced in HTTP handlers must be first bound here
    // No dynamic paths allowed, only static ones
    endpoints = vec![
        Binding::Http {
            path: "/health",
            config: HttpBindingConfig::new(false, false, false, None),
        },
        Binding::Ws {
            path: "/ws",
            config: WsBindingConfig::new(false, false, false),
        },
        Binding::Http {
            path: "/api",
            config: HttpBindingConfig::new(false, false, false, None),
        },
    ],
    // State persistence options:
    // - EveryMessage: Save after each message (safest, slower)
    // - OnInterval(n): Save every n seconds
    // - Never: No automatic saves (manual only)
    save_config = SaveOptions::EveryMessage,
    wit_world = "todo-template-dot-os-v0"
)]

// =============================================================================
// CORE TODO APPLICATION IMPLEMENTATION
// =============================================================================

impl TodoState {
    /// Initialize the application state
    #[init]
    async fn initialize(&mut self) {
        debug!("Initializing todo list state");
        // Add your app to the Hyperware homepage
        // Parameters: name, icon (emoji), path, widget
        add_to_homepage("Todo App", Some("👀"), Some("/"), None);

        // Initialize your app state
        self.tasks = Vec::new();
        self.ws_channels = HashSet::new();
        self.clients = Vec::new();
        // You can use our() to get the address of the current process
        let our = our();
        debug!("Process has just started on here: {}", our);
    }

    #[local]
    #[remote]
    async fn share_tasks(&mut self, request: String) -> Vec<TodoItem> {
        let source = source();
        debug!("Sharing tasks with {}", source);
        let _value = request;
        self.tasks.clone()
    }

    #[local]
    #[remote]
    async fn merge_tasks(&mut self, tasks: Vec<TodoItem>) -> Result<(), String> {
        let source = source();
        debug!("Merging tasks with {}", source);
        self.tasks.extend(tasks);
        Ok(())
    }

    // HTTP ENDPOINT WITH PARAMETERS
    // Parameters are sent as either:
    // - Single value: { "MethodName": value }
    // - Multiple values as tuple: { "MethodName": [val1, val2] }
    #[http]
    async fn get_tasks(&self, request: String) -> Result<Vec<TodoItem>, String> {
        debug!("Request: {:?}", request);
        debug!("Fetching tasks");
        Ok(self.tasks.clone())
    }

    // WEBSOCKET ENDPOINT
    // WebSocket messages are sent as JSON blobs
    // The message type is specified in the WsMessageType enum
    // The blob contains the message data
    #[ws]
    fn websocket(&mut self, channel_id: u32, message_type: WsMessageType, blob: LazyLoadBlob) {
        match message_type {
            WsMessageType::Text => {
                // Get the message from the blob
                if let Ok(message) = String::from_utf8(blob.bytes.clone()) {
                    debug!("Received WebSocket text message: {}", message);
                    // Parse the message as JSON
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&message) {
                        // Handle different message types
                        if let Some(action) = json.get("action").and_then(|v| v.as_str()) {
                            match action {
                                "get_tasks" => {
                                    debug!("Getting tasks on channel {}", channel_id);
                                    ws_get_tasks(channel_id, self.tasks.clone());
                                }
                                "add_task" => {
                                    if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                                        if !text.trim().is_empty() {
                                            debug!("Adding task on channel {}", channel_id);
                                            let new_task = TodoItem {
                                                id: Uuid::new_v4().to_string(),
                                                text: text.to_string(),
                                                completed: false,
                                            };
                                            self.tasks.push(new_task.clone());
                                            ws_add_task(channel_id, new_task.clone(), self.tasks.clone());
                                        } else {
                                            error!("Task text cannot be empty");
                                        }
                                    }
                                }
                                "toggle_task" => {
                                    if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                        if let Some(task) =
                                            self.tasks.iter_mut().find(|t| t.id == id)
                                        {
                                            task.completed = !task.completed;
                                            ws_toggle_task(channel_id, task.clone(), self.tasks.clone());
                                        } else {
                                            error!("Task with id '{}' not found", id);
                                        }
                                    }
                                }
                                _ => {
                                    error!("Unknown WebSocket action: {}", action);
                                }
                            }
                        }
                    }
                }
            }
            WsMessageType::Binary => {
                error!("Received WebSocket binary message");
            }
            WsMessageType::Ping => {
                debug!("Received WebSocket ping message");
                ws_ack(channel_id);
            }
            WsMessageType::Pong => {
                debug!("Received WebSocket pong message");
                ws_ack(channel_id);
            }
            WsMessageType::Close => {
                debug!("Received WebSocket close message");
                let server = get_server().unwrap();
                server.handle_websocket_close(channel_id);
                self.ws_channels.remove(&channel_id);
            }
        }
    }
}

    // REMOTE ENDPOINT EXAMPLE
    // These are called by other nodes in the P2P network
    // Use #[remote] instead of #[http]
    // #[remote]
    // async fn handle_remote_message(&mut self, message: String) -> Result<String, String> {
    //     // Store the message
    //     // Note: In remote handlers, you can't easily get the sender's node ID
    //     // You would need to include it in the message payload
    //     self.messages.push(format!("Remote message: {}", message));
    //     
    //     Ok("Message received".to_string())
    // }
    
    // P2P COMMUNICATION EXAMPLE
    // Shows how to send messages to other nodes
    // #[http]
    // async fn send_to_node(&mut self, request_body: String) -> Result<String, String> {
    //     // Parse request containing target node and message
    //     #[derive(Deserialize)]
    //     struct SendRequest {
    //         target_node: String,
    //         message: String,
    //     }
    //     
    //     let req: SendRequest = serde_json::from_str(&request_body)
    //         .map_err(|e| format!("Invalid request: {}", e))?;
    //     
    //     // Construct the target address
    //     // Format: "process-name:package-name:publisher"
    //     let target_process_id = "skeleton-app:skeleton-app:skeleton.os"
    //         .parse::<ProcessId>()
    //         .map_err(|e| format!("Invalid process ID: {}", e))?;
    //     
    //     let target_address = Address::new(req.target_node, target_process_id);
    //     
    //     // Create request wrapper for remote method
    //     let request_wrapper = serde_json::json!({
    //         "HandleRemoteMessage": req.message
    //     });
    //     
    //     // Send the request
    //     // CRITICAL: Always set expects_response timeout for remote calls
    //     let result = Request::new()
    //         .target(target_address)
    //         .body(serde_json::to_vec(&request_wrapper).unwrap())
    //         .expects_response(30) // 30 second timeout
    //         .send_and_await_response(30);
    //     
    //     match result {
    //         Ok(_) => Ok("Message sent successfully".to_string()),
    //         Err(e) => Err(format!("Failed to send message: {:?}", e))
    //     }
    // }


// ICON FOR YOUR APP (base64 encoded PNG, 256x256 recommended)
// Generate your own icon and encode it, or use an emoji in add_to_homepage
// const ICON: &str = "";

// WIT TYPE COMPATIBILITY NOTES:
// The hyperprocess macro generates WebAssembly Interface Types from your code.
// Supported types:
// ✅ Primitives: bool, u8-u64, i8-i64, f32, f64, String
// ✅ Vec<T> where T is supported
// ✅ Option<T> where T is supported  
// ✅ Simple structs with public fields
// ❌ HashMap - use Vec<(K,V)> instead
// ❌ Fixed arrays [T; N] - use Vec<T>
// ❌ Complex enums with data
// 
// Workaround: Return complex data as JSON strings

// COMMON PATTERNS:

// 1. STATE MANAGEMENT
// Your AppState is automatically persisted based on save_config
// Access current state with &self (read) or &mut self (write)

// 2. ERROR HANDLING
// Return Result<T, String> for fallible operations
// The String error will be sent to the frontend

// 3. FRONTEND COMMUNICATION
// Frontend calls HTTP endpoints via POST to /api
// Body format: { "MethodName": parameters }

// 4. P2P PATTERNS
// - Use #[remote] for methods other nodes can call
// - Use Request API for calling other nodes
// - Always set timeouts for remote calls
// - Design for eventual consistency

// 5. SYSTEM INTEGRATION
// Common system processes you might interact with:
// - "vfs:distro:sys" - Virtual file system
// - "http-server:distro:sys" - HTTP server (automatic with macro)
// - "timer:distro:sys" - Timers and scheduling
// - "kv:distro:sys" - Key-value storage

// DEVELOPMENT WORKFLOW:
// 1. Define your AppState structure
// 2. Add HTTP endpoints for UI interaction
// 3. Add remote endpoints for P2P features
// 4. Build with: kit b --hyperapp
// 5. Start with: kit s
// 6. Access at: http://localhost:8080

// DEBUGGING TIPS:
// - Use the logging macros for backend logs (appears in terminal)
// - Check browser console for frontend errors
// - Common issues:
//   * Missing request_body parameter
//   * Wrong parameter format (object vs tuple)
//   * ProcessId parsing errors
//   * Missing /our.js in HTML
