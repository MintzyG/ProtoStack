import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const HTML_PATH = path.join(__dirname, "index.html");
const LOG_DIR = path.resolve(__dirname, "../../research/usability-sessions");
const LOG_FILE = path.join(LOG_DIR, "hifi-session-02.jsonl");
const PORT = 4174;

fs.mkdirSync(LOG_DIR, { recursive: true });

const server = http.createServer((req, res) => {
  if (req.method === "POST" && req.url === "/api/track") {
    let body = "";
    req.on("data", (chunk) => (body += chunk));
    req.on("end", () => {
      if (body.trim()) {
        fs.appendFileSync(LOG_FILE, body.trim() + "\n");
      }
      res.writeHead(204);
      res.end();
    });
    return;
  }

  if (req.method === "GET" && (req.url === "/" || req.url.startsWith("/?"))) {
    res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
    fs.createReadStream(HTML_PATH).pipe(res);
    return;
  }

  res.writeHead(404);
  res.end("Not found");
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`ProtoStack Hi-Fi Session 02 running at http://127.0.0.1:${PORT}`);
  console.log(`Recording PostHog-style events to: ${LOG_FILE}`);
});
