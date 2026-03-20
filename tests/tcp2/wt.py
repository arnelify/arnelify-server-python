from arnelify_server import WebTransport
from arnelify_server import WebTransportBytes
from arnelify_server import WebTransportCtx
from arnelify_server import WebTransportOpts
from arnelify_server import WebTransportStream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  wt_opts: WebTransportOpts = {
    "block_size_kb": 64,
    "cert_pem": "certs/cert.pem",
    "compression": True,
    "handshake_timeout": 30,
    "key_pem": "certs/key.pem",
    "max_message_size_kb": 64,
    "ping_timeout": 15,
    "port": 4433,
    "send_timeout": 30,
    "thread_limit": 4
  }

  wt: WebTransport = WebTransport(wt_opts)
  async def wt_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  wt.logger(wt_logger)
  async def wt_handler(ctx: WebTransportCtx, bytes: WebTransportBytes, stream: WebTransportStream) -> Awaitable[None]:
    await stream.push(ctx, bytes)
    await stream.close()

  wt.on("connect", wt_handler)
  await wt.start()

if __name__ == "__main__":
    asyncio.run(main())