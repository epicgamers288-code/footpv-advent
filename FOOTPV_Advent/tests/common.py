import os,json,urllib.request,urllib.error
base=os.getenv('API_URL','http://127.0.0.1:8080/api')
def req(op,token='',**data):
    q=urllib.request.Request(base+'/'+op,data=json.dumps(data).encode(),headers={'Content-Type':'application/json','Authorization':'Bearer '+token})
    try:return json.load(urllib.request.urlopen(q,timeout=8))
    except urllib.error.HTTPError as e:raise ValueError(e.code,json.load(e)) from e
def login(i):return req('login',testUser=i)['token']
def clock(t,day,hour=10,minute=0):req('clock',t,utc=f'2026-12-{day:02}T{hour:02}:{minute:02}:00+03:00')
def denied(fn):
    try:fn()
    except ValueError as e:assert e.args[0] in (400,403),e;return
    raise AssertionError('Request should be denied')
