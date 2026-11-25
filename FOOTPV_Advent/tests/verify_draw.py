import hashlib,json,sys
def h(s):return hashlib.sha256(s.encode()).hexdigest()
def verify(p):
    assert h(p['seed'])==p['commitment']
    pool=p['snapshot'].copy()
    assert len({ticket for ticket,_ in pool})==len(pool)
    assert h(json.dumps(pool,separators=(',',':')))==p['digest']
    winners=[];counter=0
    for prize in p['prizes']:
        if not pool:break
        maximum=2**64-1
        while True:
            prefix=f"{p['seed']}:{p['digest']}:{counter}"; counter+=1
            number=int(h(prefix)[:16],16)
            if number<maximum-maximum%len(pool):break
        ticket,owner=pool[number%len(pool)]
        winners.append(dict(ticket=ticket,owner=owner,prize=prize))
        pool=[x for x in pool if x[1]!=owner]
    assert winners==p['winners']
if __name__=='__main__':verify(json.load(open(sys.argv[1])));print('Протокол совпадает')
