from arnelify_server import Http3
from arnelify_server import Http3Ctx
from arnelify_server import Http3Opts
from arnelify_server import Http3Stream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  http3_opts: Http3Opts = {
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

  http3: Http3 = Http3(http3_opts)
  async def http3_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  http3.logger(http3_logger)
  async def http3_handler(ctx: Http3Ctx, stream: Http3Stream) -> Awaitable[None]:
    await stream.set_code(200)
    await stream.push_json(ctx)
    await stream.end()

  http3.on("/", http3_handler)
  await http3.start()

if __name__ == "__main__":
    asyncio.run(main())