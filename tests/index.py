from arnelify_server import ArnelifyServer
from arnelify_server.contracts.res import Res
from os import path
import platform
import json

def main():

  # Compile the dynamic libraries as described here:
  # https://github.com/arnelify/arnelify-server-python

  # libPath: str = ""
  # if platform.architecture()[0] != '64bit':
  #   print("CPU platform isn't 64bit.")
  #   exit(1)
    
  # if platform.machine() == 'aarch64':
  #   libPath = path.abspath('arnelify_server/bin/arnelify_server_arm64.so')

  # elif platform.machine() == 'x86_64':
  #   libPath = path.abspath('arnelify_server/bin/arnelify_server_amd64.so')
  
  # else:
  #   print("CPU platform isn't supported.")
  #   exit(1)

  server = ArnelifyServer({
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
      "SERVER_QUEUE_LIMIT": 1024,
      "SERVER_UPLOAD_DIR": "./storage/upload"
    })

  def handler(req: dict, res: Res):
    res.setCode(200)
    res.addBody(json.dumps(req, separators=(',', ':')))
    res.end()

  server.setHandler(handler)

  def callback(message: str, isError: bool):
    if isError:
      print("[Arnelify Server]: Error: " + message)
    print("[Arnelify Server]: " + message)

  server.start(callback)

if __name__ == "__main__":
    main()