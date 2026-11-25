import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'deployment'))
from bot_module import advent_reply
def event(text,kind='private'):return {'message':{'chat':{'id':5,'type':kind},'text':text}}
for command in ('/advent','/start advent','/help','/wheel','/tickets','/games','/rules','/rating'):
    reply=advent_reply(event(command),'https://example.invalid');assert reply['chat_id']==5 and reply['reply_markup']['inline_keyboard'][0][0]['web_app']['url'].startswith('https://example.invalid/?tab=')
assert not advent_reply(event('/start'),'') and not advent_reply(event('/else'),'') and not advent_reply(event('/advent','group'),'')
print('PASS 8 входов бота и передача обычного /start, других команд и групповых сообщений существующему диспетчеру')
