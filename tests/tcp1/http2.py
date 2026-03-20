from arnelify_server import Http2
from arnelify_server import Http2Ctx
from arnelify_server import Http2Opts
from arnelify_server import Http2Stream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:
   
  http2_opts: Http2Opts = {
      "allow_empty_files": True,
      "block_size_kb": 64,
      "cert_pem": "certs/cert.pem",
      "charset": "utf-8",
      "compression": True,
      "keep_alive": 30,
      "keep_extensions": True,
      "key_pem": "certs/key.pem",
      "max_fields": 60,
      "max_fields_size_total_mb": 1,
      "max_files": 3,
      "max_files_size_total_mb": 60,
      "max_file_size_mb": 60,
      "port": 4433,
      "storage_path": "/var/www/cpp/storage",
      "thread_limit": 4,
  }

  http2: Http2 = Http2(http2_opts)
  async def http2_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  http2.logger(http2_logger)
  async def http2_handler(ctx: Http2Ctx, stream: Http2Stream) -> Awaitable[None]:
    await stream.set_code(200)
    await stream.push_json(ctx)
    await stream.end()

  http2.on("/", http2_handler)
  await http2.start()

if __name__ == "__main__":
    asyncio.run(main())