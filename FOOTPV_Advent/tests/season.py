from common import *
from verify_draw import verify
from concurrent.futures import ThreadPoolExecutor
import time,uuid
a=login(1);u=login(2);v=login(3);clock(a,10)
for prize in ['Мяч','Шарф']:req('prize',a,title=prize,request=str(uuid.uuid4()))
denied(lambda:req('admin',u));denied(lambda:req('state','bad'));denied(lambda:req('draw',a));clock(a,11,8,59);assert req('state',u)['games']==[]
total=0
for day in range(1,21):
    date=10+day;game=(day+1)//2
    if day%2==0:
        clock(a,date,0);s=req('state',u);assert s['game']==game and s['wheelDay'] is None
    clock(a,date,9);s=req('state',u);assert s['wheelDay']==day and len(s['games'])==game
    req('open',u,day=day)
    with ThreadPoolExecutor(4) as ex:rolls=list(ex.map(lambda _:req('spin',u,day=day)['reward'],range(4)))
    assert len(set(rolls))==1 and 1<=rolls[0]<=5;total+=rolls[0]
    if day==1:req('spin',v,day=day)
    if day%2:
        r=req('start',u,game=game);assert r['id']==req('start',u,game=game)['id'] and 'answer' not in r['scene']
        time.sleep(.19)
        retry=lambda:req('action',u,id=r['id'],step=0,x=50,y=72,choice=0)
        assert retry()['step']==retry()['step']==1
    else:
        assert len([r for r in s['attempts'] if r['game']==game])==1
        for k in (1,2):
            clock(a,date,11,k*2);r=req('start',u,game=game);time.sleep(.19);req('action',u,id=r['id'],step=0,x=50,y=80,choice=1)
        clock(a,date,11,8);denied(lambda:req('start',u,game=game))
    print('PASS день',day,flush=True)
clock(a,31,0);denied(lambda:req('draw',a));snap=req('freeze',a);assert snap['seed'] is None and snap['digest']
out=req('draw',a);verify(out);assert out==req('draw',a) and len(out['winners'])==2
s=req('state',login(2));assert len(s['opened'])==20 and len(s['attempts'])==30 and len(s['tickets'])==total+50<=150
assert len({t['id'] for t in s['tickets']})==len(s['tickets'])
for g in range(1,11):
    rank=req('rating',u,game=g);assert rank['closed'] and rank['rows'][0]['place']==1
print('PASS 30 попыток, лимиты, 10 финальных рейтингов, 50 билетов за игры, публичный протокол, уникальные победители')
