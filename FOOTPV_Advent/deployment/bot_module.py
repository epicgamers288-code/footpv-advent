"""Import into the existing private-chat dispatcher. No webhook or polling changes."""
from urllib.parse import urlencode
def advent_reply(update, public_url):
    msg=update.get("message",{})
    if msg.get("chat",{}).get("type")!="private": return None
    words=msg.get("text","").split()
    if not words:return None
    cmd=words[0].split("@")[0]
    if cmd=="/start":
        if words[1:]!=["advent"]:return None
        cmd="/advent"
    sections={"/advent":"calendar","/games":"games","/wheel":"wheel","/rating":"rating","/tickets":"tickets","/rules":"rules","/help":"rules"}
    if cmd not in sections:return None
    return {"chat_id":msg["chat"]["id"],"text":"До Нового гола! Зимний календарь футбольной академии FOOTPV.","reply_markup":{"inline_keyboard":[[{"text":"Открыть календарь","web_app":{"url":public_url.rstrip("/")+"/?"+urlencode({"tab":sections[cmd]})}}]]}}
