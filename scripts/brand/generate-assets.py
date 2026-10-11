"""Rebuild SQLPage's vector lockups and raster exports from Outfit and its model render.

Run from the repository root after scripts/brand/export-sculpture.mjs.
Requires Python Pillow/fonttools/brotli and ImageMagick's convert.
"""
from pathlib import Path
from tempfile import TemporaryDirectory
import argparse
import shutil
import subprocess
from PIL import Image
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen

parser = argparse.ArgumentParser()
parser.add_argument('--render', default='/tmp/sqlpage-brand-render/mascot-transparent.png')
args = parser.parse_args()
root = Path.cwd()
brand = root/'examples/official-site/assets/brand'
docs = root/'docs/brand'
docs.mkdir(parents=True, exist_ok=True)
font = instantiateVariableFont(TTFont(brand/'fonts/Outfit.ttf'), {'wght':600})
glyphs = font.getGlyphSet()
cmap = font.getBestCmap()
units = font['head'].unitsPerEm

def lettering(text, size, x, baseline, color, tracking=0):
    scale = size/units
    paths=[]
    for char in text:
        name=cmap[ord(char)]
        pen=SVGPathPen(glyphs)
        glyphs[name].draw(TransformPen(pen,(scale,0,0,-scale,x,baseline)))
        paths.append(f'<path fill="{color}" d="{pen.getCommands()}"/>')
        x += glyphs[name].width*scale + tracking
    return ''.join(paths), x

# Preserve the broad database silhouette of SQLPage's original icon. The jewel
# remains shallow and close to the stack, as it is in the original 3D artwork.
def symbol(foreground, accent):
    return f'''<g fill="none" stroke-linejoin="round" stroke-linecap="round">
      <g stroke="{accent}" stroke-width="3">
        <path d="M24 9 33 3h30l9 6-24 24Z"/>
        <path d="m24 9 24 5 24-5M33 3l15 11L63 3M48 14v19" stroke-width="1.5"/>
      </g>
      <g transform="translate(-3.2 26) scale(.1)" stroke="{foreground}" stroke-width="60">
        <path d="M171 264v489a341 122 0 0 0 682 0V264"/>
        <ellipse cx="512" cy="264" rx="341" ry="122"/>
        <path d="M171 427a341 122 0 0 0 682 0M171 591a341 122 0 0 0 682 0" stroke="{accent}" stroke-width="70"/>
      </g>
    </g>'''

def svg(width,height,content):
    return f'<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 {width} {height}" width="{width}" height="{height}">{content}</svg>'

def save(name,content,copy=True):
    (brand/name).write_text(content+'\n')
    if copy: (docs/name).write_text(content+'\n')

for name,foreground,accent in [('light','#f4f7f8','#58cce0'),('dark','#080e11','#146575'),('mono-light','#f4f7f8','#f4f7f8'),('mono-dark','#080e11','#080e11')]:
    save(f'symbol-{name}.svg',svg(96,120,symbol(foreground,accent)))
    paths,end=lettering('SQLPage',78,108,86,foreground,tracking=-1)
    save(f'logo-horizontal-{name}.svg',svg(round(end+8),120,symbol(foreground,accent)+paths))
    paths,end=lettering('SQLPage',60,0,174,foreground,tracking=-.5)
    save(f'logo-stacked-{name}.svg',svg(round(end),190,f'<g transform="translate({end/2-48} 0)">{symbol(foreground,accent)}</g>'+paths))

# The original favicon's cylinder path and proportions are retained exactly.
# Small-size polish changes only the background and strokes; omitting the glow
# leaves the existing silhouette crisp at 16px. The jewel belongs to the full mark.
legacy_cylinder = 'M171 264a341 122 0 1 0 682 0 341 122 0 1 0-682 0m0 0v245a341 122 0 0 0 682 0V264M171 509v244a341 122 0 0 0 682 0V509'
favicon=svg(1024,1024,f'''<rect width="1024" height="1024" rx="64" fill="#080e11"/>
<path d="{legacy_cylinder}" fill="none" stroke="#f4f7f8" stroke-linecap="round" stroke-linejoin="round" stroke-width="75"/>
<path d="M171 509a341 122 0 0 0 682 0" fill="none" stroke="#58cce0" stroke-linecap="round" stroke-width="75"/>''')
save('favicon.svg',favicon)
for p in ['frontend/src/favicon.svg','docs/favicon.svg']: (root/p).write_text(favicon+'\n')

def raster(name, size=None):
    dest=(brand/name).with_suffix('.png')
    command=['convert','-density','300','-background','none',str(brand/name)]
    if size: command+=['-resize',size]
    subprocess.run(command+[str(dest)],check=True,capture_output=True)
    Image.open(dest).convert('RGBA').save(dest,optimize=True)
    if (docs/name).exists(): shutil.copyfile(dest,docs/dest.name)
    return dest

for variant in ['light','dark','mono-light','mono-dark']:
    for kind in ['logo-horizontal','logo-stacked','symbol']:
        raster(f'{kind}-{variant}.svg','1600x')
raster('favicon.svg','512x512')
icon=Image.open(brand/'favicon.png').convert('RGBA')
icon.save(root/'docs/favicon.png')
icon.save(root/'examples/official-site/favicon.ico',format='ICO',sizes=[(16,16),(32,32),(48,48),(64,64),(128,128),(256,256)])
icon.resize((180,180),Image.Resampling.LANCZOS).save(brand/'apple-touch-icon.png')

mascot=Image.open(args.render).convert('RGBA')
if mascot.getchannel('A').getextrema()[0] == 255:
    raise RuntimeError('The reference render must retain actual transparent coverage')
# Keep legacy logo URLs as shaded 3D artwork, rather than replacing their
# established character with a line drawing. Trim only export transparency.
alpha=mascot.getchannel('A').point(lambda value: 255 if value > 20 else 0)
trimmed=mascot.crop(alpha.getbbox())
trimmed.thumbnail((600,600),Image.Resampling.LANCZOS)
legacy_logo=Image.new('RGBA',(690,690))
legacy_logo.alpha_composite(trimmed,((690-trimmed.width)//2,(690-trimmed.height)//2))
legacy_logo.save(root/'docs/logo.png',optimize=True)
legacy_logo.save(root/'docs/logo.webp',lossless=True)
legacy_icon=Image.new('RGBA',(220,248))
small_logo=legacy_logo.resize((220,220),Image.Resampling.LANCZOS)
legacy_icon.alpha_composite(small_logo,(0,14))
legacy_icon.save(root/'examples/official-site/assets/icon.webp',lossless=True)
mascot.save(brand/'mascot-transparent.png')
mascot.save(brand/'mascot-transparent.webp',lossless=True)
for name,color in [('dark','#080e11'),('light','#f4f7f8')]:
    image=Image.new('RGBA',mascot.size,color)
    image.alpha_composite(mascot)
    image.convert('RGB').resize((1000,1000),Image.Resampling.LANCZOS).save(brand/f'mascot-{name}.webp',quality=95)

def composition(name,w,h,layout):
    # Sources refer to the canonical render by relative URL, so remain editable.
    if layout=='wide':
        text_x=w*.10; word_y=h*.27; headline_y=h*.52; size=h*.16; art_x=w*.52; art_size=h*1.22; art_y=-h*.10
    elif layout=='youtube':
        # All text and sculpture sit inside the central banner-safe strip.
        text_x=560; word_y=630; headline_y=748; size=68; art_x=1460; art_size=580; art_y=430
    else:
        text_x=w*.08; word_y=h*.26; headline_y=h*.50; size=h*.125; art_x=w*.46; art_size=h*1.20; art_y=-h*.07
    word, end=lettering('SQLPage',size*.48,text_x,word_y,'#f4f7f8',tracking=-size*.005)
    one,_=lettering('Turn your data',size,text_x,headline_y,'#f4f7f8',tracking=-size*.02)
    two,_=lettering('into an app.',size,text_x,headline_y+size*1.10,'#f4f7f8',tracking=-size*.02)
    content=f'<rect width="{w}" height="{h}" fill="#080e11"/>'
    content+=f'<image xlink:href="mascot-transparent.png" x="{art_x}" y="{art_y}" width="{art_size}" height="{art_size}"/>'
    content+=word+one+two
    save(f'{name}.svg',svg(w,h,content),copy=False)
    # Compose pixels independently of the system SVG renderer's external-image support.
    with TemporaryDirectory() as td:
        temp=Path(td)/'lettering.svg'
        temp.write_text(svg(w,h,word+one+two))
        overlay=Path(td)/'lettering.png'
        subprocess.run(['convert','-density','300','-background','none',str(temp),'-resize',f'{w}x{h}!',str(overlay)],check=True,capture_output=True)
        canvas=Image.new('RGBA',(w,h),'#080e11')
        artwork=mascot.resize((round(art_size),round(art_size)),Image.Resampling.LANCZOS)
        canvas.alpha_composite(artwork,(round(art_x),round(art_y)))
        canvas.alpha_composite(Image.open(overlay).convert('RGBA'))
        canvas.convert('RGB').save(brand/f'{name}.png')
    Image.open(brand/f'{name}.png').convert('RGB').save(brand/f'{name}.webp',quality=95)

for name,w,h,layout in [('social-og',1200,630,'standard'),('social-github',1280,640,'standard'),('social-x',1500,500,'wide'),('social-youtube',2560,1440,'youtube')]:
    composition(name,w,h,layout)
# Keep the site's authored legacy social composition; format exports are optional artwork.
avatar=Image.new('RGBA',(512,512),'#080e11')
with TemporaryDirectory() as td:
    temp=Path(td)/'symbol.png'
    subprocess.run(['convert','-density','300','-background','none',str(brand/'symbol-light.svg'),'-resize','300x380',str(temp)],check=True,capture_output=True)
    mark=Image.open(temp).convert('RGBA')
    avatar.alpha_composite(mark,((512-mark.width)//2,(512-mark.height)//2))
avatar.convert('RGB').save(brand/'avatar.png')
print('Generated sculpture renders, outlined lockups, favicons, and social exports')
