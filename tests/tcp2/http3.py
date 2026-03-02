from arnelify_server import Http3
from arnelify_server import Http3Ctx
from arnelify_server import Http3Opts
from arnelify_server import Http3Stream

http3_opts: Http3Opts = {
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

http3 = Http3(http3_opts)
def http3_logger(_level: str, message: str):
  print("[Arnelify Server]: " + message)

http3.logger(http3_logger)

def http3_handler(ctx: Http3Ctx, stream: Http3Stream):
  stream.set_code(200)
  stream.push_json(ctx)
  stream.end()

http3.on("/", http3_handler)
http3.start()