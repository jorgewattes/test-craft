"""Generate Test Craft's original vector mark and Windows icon sizes."""
from pathlib import Path
from PIL import Image, ImageDraw

assets = Path(__file__).resolve().parents[1] / 'assets'
assets.mkdir(exist_ok=True)
svg = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
<rect x="8" y="8" width="240" height="240" rx="58" fill="#173531"/>
<rect x="51" y="58" width="127" height="153" rx="18" fill="#6bb9a5" transform="rotate(-10 115 134)"/>
<rect x="70" y="46" width="127" height="153" rx="18" fill="#fff9ed"/>
<path d="M154 46h18v48l-9-7-9 7z" fill="#ad94ce"/>
<path d="M94 88h39M94 105h43" stroke="#6bb9a5" stroke-width="8" stroke-linecap="round"/>
<path d="m100 147 17 17 42-43" fill="none" stroke="#e58461" stroke-width="13" stroke-linecap="round" stroke-linejoin="round"/>
<path d="m209 28 5 14 14 5-14 5-5 14-5-14-14-5 14-5z" fill="#edc676"/>
</svg>'''
(assets / 'test-craft.svg').write_text(svg, encoding='utf-8')
scale = 4
im = Image.new('RGBA', (1024, 1024))
d = ImageDraw.Draw(im)
def box(coords, radius, fill):
    d.rounded_rectangle(tuple(int(c*scale) for c in coords), radius=radius*scale, fill=fill)
box((8,8,248,248),58,'#173531')
paper = Image.new('RGBA', im.size)
pd = ImageDraw.Draw(paper)
pd.rounded_rectangle((51*scale,58*scale,178*scale,211*scale), radius=18*scale,fill='#6bb9a5')
paper = paper.rotate(10, Image.Resampling.BICUBIC, center=(115*scale,134*scale))
im = Image.alpha_composite(im, paper)
d = ImageDraw.Draw(im)
box((70,46,197,199),18,'#fff9ed')
d.polygon([(154*scale,46*scale),(172*scale,46*scale),(172*scale,94*scale),(163*scale,87*scale),(154*scale,94*scale)],fill='#ad94ce')
for y,end in [(88,133),(105,137)]:
    d.line([(94*scale,y*scale),(end*scale,y*scale)], fill='#6bb9a5',width=8*scale)
    for x in [94,end]:d.ellipse(((x-4)*scale,(y-4)*scale,(x+4)*scale,(y+4)*scale), fill='#6bb9a5')
pts=[(100,147),(117,164),(159,121)]
d.line([(x*scale,y*scale) for x,y in pts],fill='#e58461',width=13*scale,joint='curve')
for x,y in pts:d.ellipse(((x-6.5)*scale,(y-6.5)*scale,(x+6.5)*scale,(y+6.5)*scale),fill='#e58461')
d.polygon([(x*scale,y*scale) for x,y in [(209,28),(214,42),(228,47),(214,52),(209,66),(204,52),(190,47),(204,42)]],fill='#edc676')
im.resize((256,256),Image.Resampling.LANCZOS).save(assets/'test-craft.png')
im.save(assets/'test-craft.ico', sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
