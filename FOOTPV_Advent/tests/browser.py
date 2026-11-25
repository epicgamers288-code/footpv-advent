from common import *
from playwright.sync_api import sync_playwright
a=login(1);clock(a,11)
with sync_playwright() as pw:
    browser=pw.chromium.launch(executable_path=os.environ.get('CHROMIUM','/usr/bin/chromium'),args=['--no-sandbox'])
    page=browser.new_page(viewport={'width':390,'height':844},is_mobile=True,has_touch=True)
    errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
    page.route('https://telegram.org/**',lambda r:r.fulfill(body='',content_type='text/javascript'))
    page.goto(os.getenv('WEB_URL','http://127.0.0.1:5173'))
    page.get_by_text('Войти в тестовый сезон').click()
    page.locator('.door.today').click();page.get_by_text('К колесу подарков').click()
    page.get_by_role('button',name='Крутить',exact=True).click();page.get_by_text('Пропустить анимацию').click();page.locator('.reward').wait_for()
    page.reload();page.locator('.door.today').wait_for();assert page.locator('.door').count()==20
    assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
    assert page.locator('.door img').evaluate_all('(imgs)=>imgs.every(i=>i.complete && i.naturalWidth>0)')
    for g in range(1,11):
        clock(a,9+2*g);page.reload();page.locator('.matchlink').click();page.get_by_text('Начать зачётную попытку').click()
        field=page.locator('.pitch canvas');field.wait_for(timeout=30000);page.wait_for_timeout(400)
        opts=page.locator('.choices button')
        if opts.count():opts.first.click()
        else:field.click(position={'x':110,'y':115})
        page.get_by_text('Свернуть игру').click()
        assert not errors,errors
    page.get_by_role('button',name='Профиль').click();page.get_by_text('Панель организатора').click()
    page.get_by_text('Активность по дням').wait_for();page.get_by_role('button',name='Закрыть',exact=True).click()
    page.set_viewport_size({'width':1280,'height':800});page.get_by_role('button',name='Календарь').click()
    page.locator('.grid').wait_for();assert not errors,errors
    print('PASS Chromium: вход, 20 коробок с графикой, колесо, перезагрузка, все 10 игр, профиль, админка; 390×844 и 1280×800')
    browser.close()
