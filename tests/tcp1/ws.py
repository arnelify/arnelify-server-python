from arnelify_server import WebSocket
from arnelify_server import WebSocketBytes
from arnelify_server import WebSocketCtx
from arnelify_server import WebSocketOpts
from arnelify_server import WebSocketStream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  ws_opts: WebSocketOpts = {
    "block_size_kb": 64,
    "compression": True,
    "max_message_size_kb": 64,
    "ping_timeout": 15,
    "port": 4433,
    "rate_limit": 5,
    "read_timeout": 30,
    "send_timeout": 30,
    "thread_limit": 4
  }

  ws: WebSocket = WebSocket(ws_opts)
  async def ws_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  ws.logger(ws_logger)
  async def ws_handler(ctx: WebSocketCtx, bytes: WebSocketBytes, stream: WebSocketStream) -> Awaitable[None]:
    await stream.push(ctx, bytes)
    await stream.close()

  ws.on("connect", ws_handler)
  await ws.start()

if __name__ == "__main__":
    asyncio.run(main())