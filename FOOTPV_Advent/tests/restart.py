from common import *
import time,signal,subprocess
a=login(1);u=login(27);clock(a,11)
spin=req('spin',u,day=1);r=req('start',u,game=1);time.sleep(.22);req('action',u,id=r['id'],step=0,x=45)
os.kill(int(os.environ['TEST_SERVER_PID']),signal.SIGTERM);time.sleep(.3)
proc=subprocess.Popen(['server/target/debug/footpv-advent'])
try:
    for i in range(30):
        try:s=req('state',u);break
        except OSError:time.sleep(.1)
    assert req('spin',u,day=1)==spin and req('start',u,game=1)['id']==r['id']
    clock(a,13,0)
    for _ in range(18):
        s=req('state',u)
        if any(t['source']=='game' for t in s['tickets']):break
        time.sleep(.5)
    assert len(s['tickets'])==5+spin['reward']
    print('PASS Перезапуск процесса, сессия, активная попытка, награда колеса и закрытие пропущенного периода')
finally:proc.terminate();proc.wait()
