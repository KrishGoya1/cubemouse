// server.js
const path = require("path");
const http = require("http");
const express = require("express");
const WebSocket = require("ws");

const app = express();
const server = http.createServer(app);

// Serve index.html and any static files
app.use(express.static(path.join(__dirname)));

// HTTP server runs on port 8080
server.listen(8080, () => {
  console.log("Web server running at: http://0.0.0.0:8080");
});

// WebSocket server runs on port 9000
const wss = new WebSocket.Server({ port: 9000 }, () => {
  console.log("WebSocket server running on ws://0.0.0.0:9000");
});

wss.on("connection", (ws) => {
  console.log("Client connected");

  ws.on("message", (data) => {
    const dv = new DataView(data);
    const type = dv.getUint8(0);
    const dx = dv.getInt16(1, true);
    const dy = dv.getInt16(3, true);

    console.log(`Move event: ${dx}, ${dy} (type ${type})`);
  });

  ws.on("close", () => console.log("Client disconnected"));
});
