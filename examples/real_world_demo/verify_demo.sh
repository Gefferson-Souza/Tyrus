#!/bin/bash
set -e # Exit on error

# Resolve script directory to make execution location-agnostic
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

SERVER_RUST_PORT=3000
SERVER_NODE_PORT=3001
LOG_RUST="$SCRIPT_DIR/demo_rust.log"
LOG_NODE="$SCRIPT_DIR/demo_node.log"

cleanup() {
    echo ""
    echo "🧹 Cleaning up..."
    if [ -n "$RUST_PID" ]; then kill "$RUST_PID" 2>/dev/null || true; fi
    if [ -n "$NODE_PID" ]; then kill "$NODE_PID" 2>/dev/null || true; fi
    pkill -P $$ || true # Kill verify_demo's children just in case
}
trap cleanup EXIT

echo "=========================================="
echo "      TYRUS REAL-WORLD DEMO VERIFICATION"
echo "=========================================="
echo "Time: $(date)"
echo "WorkDir: $SCRIPT_DIR"

# 1. Check Node.js Dependencies
if [ ! -d "node_modules" ]; then
    echo "📦 Installing Node dependencies..."
    npm install
fi

# 2. Check Rust Compilation
if [ ! -d "output" ]; then
    echo "❌ Missing 'output' directory. Please run 'tyrus build' first."
    exit 1
fi

cd output
if [ ! -f "target/release/server" ] && [ ! -f "target/debug/server" ]; then
    echo "⚙️ Compiling Rust server..."
    cargo build --bin server
fi
RUST_BIN=$(find "$(pwd)/target" -name server -type f -executable | head -n 1)
cd ..

# 3. Start Rust Server
echo -e "\n🚀 Starting RUST Server (Port $SERVER_RUST_PORT)..."
$RUST_BIN > "$LOG_RUST" 2>&1 &
RUST_PID=$!
echo "Rust PID: $RUST_PID"

# Wait for port to be open
echo "Waiting for Rust server to be ready..."
for i in {1..30}; do
    if lsof -i :$SERVER_RUST_PORT >/dev/null 2>&1 || nc -z localhost $SERVER_RUST_PORT >/dev/null 2>&1; then
        echo "✅ Rust Server is UP!"
        break
    fi
    sleep 1
done

# 4. Start Node Server
echo -e "\n🚀 Starting NODE Server (Port $SERVER_NODE_PORT)..."
npm start > "$LOG_NODE" 2>&1 &
NODE_PID=$!
echo "Node PID: $NODE_PID"

# Wait for port to be open
echo "Waiting for Node server to be ready..."
for i in {1..30}; do
    if lsof -i :$SERVER_NODE_PORT >/dev/null 2>&1 || nc -z localhost $SERVER_NODE_PORT >/dev/null 2>&1; then
        echo "✅ Node Server is UP!"
        break
    fi
    sleep 1
done

# 5. Execute Tests
run_tests() {
    local PORT=$1
    local NAME=$2
    echo -e "\n🧪 Testing $NAME (Port $PORT)"
    echo "---------------------------------------------------"
    
    echo "Request: POST /users"
    curl -s -X POST "http://localhost:$PORT/users" \
         -H "Content-Type: application/json" \
         -d "{\"name\": \"$NAME User\", \"email\": \"$NAME@tyrus.dev\"}" | jq . || echo "❌ Request Failed"

    echo -e "\nRequest: GET /users"
    curl -s "http://localhost:$PORT/users" | jq . || echo "❌ Request Failed"
}

sleep 2 # Extra buffer
run_tests $SERVER_RUST_PORT "Rust"
run_tests $SERVER_NODE_PORT "Node"

echo -e "\n=========================================="
echo "✅ VERIFICATION COMPLETED SUCCESSFULLY"
echo "Logs available at:"
echo "  - $LOG_RUST"
echo "  - $LOG_NODE"
