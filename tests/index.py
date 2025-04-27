from arnelify_server import Http1
from arnelify_server.contracts.res import Http1Res
import json

def main():

  http1 = Http1({
    "SERVER_ALLOW_EMPTY_FILES": True,
    "SERVER_BLOCK_SIZE_KB": 64,
    "SERVER_CHARSET": "UTF-8",
    "SERVER_GZIP": True,
    "SERVER_KEEP_EXTENSIONS": True,
    "SERVER_MAX_FIELDS": 1024,
    "SERVER_MAX_FIELDS_SIZE_TOTAL_MB": 20,
    "SERVER_MAX_FILES": 1,
    "SERVER_MAX_FILES_SIZE_TOTAL_MB": 60,
    "SERVER_MAX_FILE_SIZE_MB": 60,
    "SERVER_PORT": 3001,
    "SERVER_THREAD_LIMIT": 1,
    "SERVER_QUEUE_LIMIT": 1024,
    "SERVER_UPLOAD_DIR": "storage/upload"
  })

  def handler(req: dict, res: Http1Res):
    res.setCode(200)
    res.addBody(json.dumps(req, separators=(',', ':')))
    res.end()

  http1.setHandler(handler)

  def callback(message: str, isError: bool):
    if isError:
      print("[Arnelify Server]: Error: " + message)
    print("[Arnelify Server]: " + message)

  http1.start(callback)

if __name__ == "__main__":
    main()