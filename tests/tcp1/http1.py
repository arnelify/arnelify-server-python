from arnelify_server import Http1
from arnelify_server import Http1Ctx
from arnelify_server import Http1Opts
from arnelify_server import Http1Stream

http1_opts: Http1Opts = {
    "allow_empty_files": False,
    "block_size_kb": 64,
    "charset": "utf-8",
    "compression": True,
    "keep_alive": 30,
    "keep_extensions": True,
    "max_fields": 10,
    "max_fields_size_total_mb": 1,
    "max_files": 10,
    "max_files_size_total_mb": 10,
    "max_file_size_mb": 10,
    "port": 4433,
    "storage_path": "/var/www/cpp/storage",
    "thread_limit": 4,
}

http1 = Http1(http1_opts)
def http1_logger(_level: str, message: str):
  print("[Arnelify Server]: " + message)

http1.logger(http1_logger)

def http1_handler(ctx: Http1Ctx, stream: Http1Stream):
  stream.set_code(200)
  stream.push_json(ctx)
  stream.end()

http1.on("/", http1_handler)
http1.start()