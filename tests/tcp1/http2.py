from arnelify_server import Http2
from arnelify_server import Http2Ctx
from arnelify_server import Http2Opts
from arnelify_server import Http2Stream

http2_opts: Http2Opts = {
    "allow_empty_files": False,
    "block_size_kb": 64,
    "cert_pem": "certs/cert.pem",
    "charset": "utf-8",
    "compression": True,
    "keep_alive": 30,
    "keep_extensions": True,
    "key_pem": "certs/key.pem",
    "max_fields": 10,
    "max_fields_size_total_mb": 1,
    "max_files": 10,
    "max_files_size_total_mb": 10,
    "max_file_size_mb": 10,
    "port": 4433,
    "storage_path": "/var/www/cpp/storage",
    "thread_limit": 4,
}

http2 = Http2(http2_opts)
def http2_logger(_level: str, message: str):
  print("[Arnelify Server]: " + message)

http2.logger(http2_logger)

def http2_handler(ctx: Http2Ctx, stream: Http2Stream):
  stream.set_code(200)
  stream.push_json(ctx)
  stream.end()

http2.on("/", http2_handler)
http2.start()