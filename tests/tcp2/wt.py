from arnelify_server import WebTransport
from arnelify_server import WebTransportBytes
from arnelify_server import WebTransportCtx
from arnelify_server import WebTransportOpts
from arnelify_server import WebTransportStream

wt_opts: WebTransportOpts = {
  "block_size_kb": 64,
  "cert_pem": "certs/cert.pem",
  "compression": True,
  "handshake_timeout": 30,
  "key_pem": "certs/key.pem",
  "max_message_size_kb": 64,
  "ping_timeout": 30,
  "port": 4433,
  "send_timeout": 30,
  "thread_limit": 4
}

wt = WebTransport(wt_opts)
def wt_logger(_level: str, message: str):
  print("[Arnelify Server]: " + message)

wt.logger(wt_logger)

def wt_handler(ctx: WebTransportCtx, bytes: WebTransportBytes, stream: WebTransportStream):
  stream.push(ctx, bytes)
  stream.close()

wt.on("connect", wt_handler)
wt.start()