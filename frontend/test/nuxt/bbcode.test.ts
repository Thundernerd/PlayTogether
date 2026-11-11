import { describe, expect, it } from 'vitest'
import { renderGameDescription } from '~/utils/bbcode'

const APPID = 570

function parse(html: string): DocumentFragment {
  const template = document.createElement('template')
  template.innerHTML = html
  return template.content
}

function imageSources(html: string): (string | null)[] {
  return [...parse(html).querySelectorAll('img')].map((img) => img.getAttribute('src'))
}

describe('renderGameDescription images', () => {
  it('renders [img]URL[/img] as an image', () => {
    const html = renderGameDescription('Intro [img]https://cdn.example.com/a.png[/img] outro', APPID)

    expect(imageSources(html)).toEqual(['https://cdn.example.com/a.png'])
    expect(parse(html).textContent).toBe('Intro  outro')
  })

  const STEAM_EXTRAS_AVIF =
    'https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/570/extras/x.avif'

  it.each([
    [
      'src first, with [/img]',
      '[img src="{STEAM_APP_IMAGE}/extras/x.avif" poster="{STEAM_APP_IMAGE}/extras/x.png" mp4="{STEAM_APP_IMAGE}/extras/x.mp4" fromclient=1][/img]',
    ],
    [
      'src last, with [/img]',
      '[img fromclient=1 mp4="{STEAM_APP_IMAGE}/extras/x.mp4" poster="{STEAM_APP_IMAGE}/extras/x.png" src="{STEAM_APP_IMAGE}/extras/x.avif"][/img]',
    ],
    [
      'src in the middle, without [/img]',
      '[img poster="{STEAM_APP_IMAGE}/extras/x.png" src="{STEAM_APP_IMAGE}/extras/x.avif" fromclient=1]',
    ],
  ])('resolves the attribute form (%s) to the Steam CDN', (_, tag) => {
    const html = renderGameDescription(`Before ${tag} after`, APPID)

    expect(imageSources(html)).toEqual([STEAM_EXTRAS_AVIF])
    expect(parse(html).textContent).toBe('Before  after')
  })

  it('resolves the older [img=http://STEAM_APP_IMAGE}…] form to the Steam CDN', () => {
    const html = renderGameDescription('[img=http://STEAM_APP_IMAGE}/extras/x.gif fromclient=1]', APPID)

    expect(imageSources(html)).toEqual([
      'https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/570/extras/x.gif',
    ])
    expect(parse(html).textContent).toBe('')
  })

  it('renders a quoted [img="URL"] as an image', () => {
    const html = renderGameDescription('[img="https://cdn.example.com/b.png"][/img]', APPID)

    expect(imageSources(html)).toEqual(['https://cdn.example.com/b.png'])
    expect(parse(html).textContent).toBe('')
  })

  it.each([
    ['[img]…[/img]', '[img]{STEAM_APP_IMAGE}/extras/x.png[/img]'],
    ['[img src=…]', '[img src="{STEAM_APP_IMAGE}/extras/x.avif" fromclient=1][/img]'],
    ['[img=…]', '[img=http://STEAM_APP_IMAGE}/extras/x.gif fromclient=1]'],
  ])('silently drops a %s placeholder image when there is no appid', (_, tag) => {
    for (const appid of [undefined, null]) {
      const html = renderGameDescription(`Before ${tag} after`, appid)

      expect(imageSources(html)).toEqual([])
      expect(parse(html).textContent).toBe('Before  after')
    }
  })

  it.each([
    ['[img]…[/img]', '[img]javascript:alert(1)[/img]'],
    ['[img src=…]', '[img src="javascript:alert(1)"][/img]'],
    ['[img=…]', '[img=javascript:alert(1)]'],
  ])('drops a non-http(s) %s image URL', (_, tag) => {
    const html = renderGameDescription(`Before ${tag} after`, APPID)

    expect(imageSources(html)).toEqual([])
    expect(html).not.toContain('javascript:')
    expect(parse(html).textContent).toBe('Before  after')
  })

  it.each([
    ['an unpaired [img]', 'Before [img] after'],
    ['an unpaired [/img]', 'Before [/img] after'],
  ])('leaves no tag text for %s', (_, description) => {
    const html = renderGameDescription(description, APPID)

    expect(imageSources(html)).toEqual([])
    expect(parse(html).textContent).toBe('Before  after')
  })
})

describe('renderGameDescription sanitising', () => {
  function eventHandlerAttributes(html: string): string[] {
    return [...parse(html).querySelectorAll('*')].flatMap((element) =>
      element.getAttributeNames().filter((name) => name.startsWith('on')),
    )
  }

  it('keeps loading="lazy" on rendered images', () => {
    const html = renderGameDescription('[img]https://cdn.example.com/a.png[/img]', APPID)

    const img = parse(html).querySelector('img')
    expect(img?.getAttribute('loading')).toBe('lazy')
  })

  it.each([
    ['raw HTML', 'Hi <img src="https://cdn.example.com/a.png" onerror="alert(1)"> [b]there[/b]'],
    ['an unquoted [img src=…] value', '[img src=https://cdn.example.com/a.png"onerror="alert(1)][/img]'],
    ['an [img] URL', '[img]https://cdn.example.com/a.png" onerror="alert(1)[/img]'],
    ['a [url=…] href', '[url=https://example.com" onmouseover="alert(1)]link[/url]'],
    ['plain text', 'Hi <b onclick="alert(1)">there</b>'],
  ])('leaves no event-handler attributes when they are injected via %s', (_, description) => {
    const html = renderGameDescription(description, APPID)

    expect(eventHandlerAttributes(html)).toEqual([])
  })
})
