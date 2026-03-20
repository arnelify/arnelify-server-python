from arnelify_server import Http1
from arnelify_server import Http1Ctx
from arnelify_server import Http1Opts
from arnelify_server import Http1Stream

import asyncio
from typing import Awaitable

async def main() -> Awaitable[None]:

  http1_opts: Http1Opts = {
    "allow_empty_files": True,
    "block_size_kb": 64,
    "charset": "utf-8",
    "compression": True,
    "keep_alive": 30,
    "keep_extensions": True,
    "max_fields": 60,
    "max_fields_size_total_mb": 1,
    "max_files": 3,
    "max_files_size_total_mb": 60,
    "max_file_size_mb": 60,
    "port": 4433,
    "storage_path": "/var/www/cpp/storage",
    "thread_limit": 4,
  }

  http1: Http1 = Http1(http1_opts)
  async def http1_logger(_level: str, message: str) -> Awaitable[None]:
    print("[Arnelify Server]: " + message)

  http1.logger(http1_logger)
  async def http1_handler(ctx: Http1Ctx, stream: Http1Stream) -> Awaitable[None]:
    await stream.set_code(200)
    await stream.push_json(ctx)
    await stream.end()

  http1.on("/", http1_handler)
  await http1.start()
  
if __name__ == "__main__":
    asyncio.run(main())