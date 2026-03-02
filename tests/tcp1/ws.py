from arnelify_server import WebSocket
from arnelify_server import WebSocketBytes
from arnelify_server import WebSocketCtx
from arnelify_server import WebSocketOpts
from arnelify_server import WebSocketStream

ws_opts: WebSocketOpts = {
  "block_size_kb": 64,
  "compression": True,
  "handshake_timeout": 30,
  "max_message_size_kb": 64,
  "ping_timeout": 30,
  "port": 4433,
  "send_timeout": 30,
  "thread_limit": 4
}

ws = WebSocket(ws_opts)
def ws_logger(_level: str, message: str):
  print("[Arnelify Server]: " + message)

ws.logger(ws_logger)

def ws_handler(ctx: WebSocketCtx, bytes: WebSocketBytes, stream: WebSocketStream):
  stream.push(ctx, bytes)
  stream.close()

ws.on("connect", ws_handler)
ws.start()