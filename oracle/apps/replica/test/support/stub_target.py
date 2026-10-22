"""Alvo falso para testar run_requests sem Docker: serve /health e respostas canônicas por porta e modo."""
import http.server
import os
import sys
import time

MODE = sys.argv[1] if len(sys.argv) > 1 else "ok"
PORT = int(os.environ["PORT"])


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        return

    def _reply(self, status, body):
        payload = body.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_GET(self):
        if self.path == "/health":
            return self._reply(200, '{"status":"ok"}')
        if MODE == "drift":
            return self._reply(200, '{"t":%d}' % time.time_ns())
        if MODE == "wrong-status":
            return self._reply(500, '{"x":1}')
        return self._reply(200, '{"x":1}')


http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
