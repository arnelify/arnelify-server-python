import json

class Res:

  def __init__(self):
    def callback(message: str, isError: bool) -> None:
      if isError:
        print("Error: " + message)
        return
      print(message)

    self.callback: callable = callback
    self.res: dict = {
      "body": "",
      "code": 200,
      "filePath": "",
      "isStatic": False,
      "headers": {}
    }
    
  def addBody(self, chunk: str) -> None:
    if len(self.res["filePath"]):
      self.callback("Can't add body to a Response that contains a file.", True)
      exit(1)

    self.res["body"] += chunk

  def setCode(self, code: int) -> None:
    self.res["code"] = code

  def setFile(self, filePath: str, isStatic: bool = False) -> None:
    if len(self.res["body"]):
      self.callback("Can't add an attachment to a Response that contains a body.", True)
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
    
    self.callback("Add the body or set the file.", True)
    exit(1)

  def serialize(self) -> str:
    return json.dumps(self.res, separators=(',', ':'))