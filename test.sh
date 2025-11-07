#!/data/data/com.termux/files/usr/bin/bash
# Simple CubeMouse server test script for Termux
# Usage: bash test_server.sh <server-ip>

SERVER_IP=${1:-127.0.0.1}
PORT=9000

echo "🔍 Testing CubeMouse server at ${SERVER_IP}:${PORT}..."

# Check if netcat (nc) is installed
if ! command -v nc >/dev/null 2>&1; then
    echo "⚠️  Installing netcat..."
    pkg install -y netcat
fi

# Try connecting
nc -vz $SERVER_IP $PORT
RESULT=$?

if [ $RESULT -eq 0 ]; then
    echo "✅ Server is reachable on port $PORT!"
else
    echo "❌ Unable to connect. Server may be down or blocked by firewall."
fi

# Optional: send a mock MOVE packet (opcode=0x01 + dx=1 + dy=2)
# Uncomment to test binary packet sending
# printf '\x01\x01\x00\x02\x00' | nc $SERVER_IP $PORT

exit $RESULT
