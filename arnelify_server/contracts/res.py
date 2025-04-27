import json

class Http1Res:

  def __init__(self):
    def logger(message: str, isError: bool) -> None:
      if isError:
        print("Error: " + message)
        return
      print(message)

    self.logger: callable = logger
    self.res: dict = {
      "body": "",
      "code": 200,
      "filePath": "",
      "headers": {},
      "isStatic": False
    }

  def addBody(self, chunk: str) -> None:
    if len(self.res["filePath"]):
      self.logger("Can't add body to a Response that contains a file.", True)
      exit(1)

    self.res["body"] += chunk

  def setCode(self, code: int) -> None:
    self.res["code"] = code

  def setFile(self, filePath: str, isStatic: bool = False) -> None:
    if len(self.res["body"]):
      self.logger("Can't add an attachment to a Response that contains a body.", True)
      exit(1)

    self.res["filePath"] = filePath  
    self.res["isStatic"] = isStatic

  def setHeader(self, key: str, value: str) -> None:
    self.res["headers"][key] = value

  def end(self) -> None:
    if len(self.res["filePath"]):
      self.res["body"] = ""
      return

    if len(self.res["body"]):
      self.res["filePath"] = ""
      self.res["isStatic"] = False
      return

  def serialize(self) -> str:
    return json.dumps(self.res, separators=(',', ':'))