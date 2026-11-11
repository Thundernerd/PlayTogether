import { describe, expect, it } from 'vitest'
import { renderGameDescription } from '~/utils/bbcode'

const APPID = 570
/** Where `{STEAM_APP_IMAGE}` must resolve to for APPID, as seen on live Steam store pages. */
const STEAM_APP_IMAGE_570 = 'https://shared.fastly.steamstatic.com/store_item_assets/steam/apps/570'

function parseFragment(html: string): DocumentFragment {
  const template = document.createElement('template')
  template.innerHTML = html
  return template.content
}

/** Asserts the rendered HTML holds exactly these image sources and this visible text. */
function expectRendered(html: string, sources: string[], text: string) {
  const fragment = parseFragment(html)
  expect([...fragment.querySelectorAll('img')].map((img) => img.getAttribute('src'))).toEqual(sources)
  expect(fragment.textContent).toBe(text)
}

describe('renderGameDescription images', () => {
  it('renders [img]URL[/img] as an image', () => {
    const html = renderGameDescription('Intro [img]https://cdn.example.com/a.png[/img] outro', APPID)

    expectRendered(html, ['https://cdn.example.com/a.png'], 'Intro  outro')
  })

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

    expectRendered(html, [`${STEAM_APP_IMAGE_570}/extras/x.avif`], 'Before  after')
  })

  it('resolves the older [img=http://STEAM_APP_IMAGE}…] form to the Steam CDN', () => {
    const html = renderGameDescription('[img=http://STEAM_APP_IMAGE}/extras/x.gif fromclient=1]', APPID)

    expectRendered(html, [`${STEAM_APP_IMAGE_570}/extras/x.gif`], '')
  })

  it('renders a quoted [img="URL"] as an image', () => {
    const html = renderGameDescription('[img="https://cdn.example.com/b.png"][/img]', APPID)

    expectRendered(html, ['https://cdn.example.com/b.png'], '')
  })

  it.each([
    ['[img]…[/img]', '[img]{STEAM_APP_IMAGE}/extras/x.png[/img]'],
    ['[img src=…]', '[img src="{STEAM_APP_IMAGE}/extras/x.avif" fromclient=1][/img]'],
    ['[img=…]', '[img=http://STEAM_APP_IMAGE}/extras/x.gif fromclient=1]'],
  ])('silently drops a %s placeholder image when there is no appid', (_, tag) => {
    for (const appid of [undefined, null]) {
      const html = renderGameDescription(`Before ${tag} after`, appid)

      expectRendered(html, [], 'Before  after')
    }
  })

  it.each([
    ['[img]…[/img]', '[img]javascript:alert(1)[/img]'],
    ['[img src=…]', '[img src="javascript:alert(1)"][/img]'],
    ['[img=…]', '[img=javascript:alert(1)]'],
  ])('drops a non-http(s) %s image URL', (_, tag) => {
    const html = renderGameDescription(`Before ${tag} after`, APPID)

    expectRendered(html, [], 'Before  after')
    expect(html).not.toContain('javascript:')
  })

  it.each([
    ['an unpaired [img]', 'Before [img] after'],
    ['an unpaired [/img]', 'Before [/img] after'],
  ])('leaves no tag text for %s', (_, description) => {
    const html = renderGameDescription(description, APPID)

    expectRendered(html, [], 'Before  after')
  })
})

describe('renderGameDescription sanitising', () => {
  function eventHandlerAttributes(html: string): string[] {
    return [...parseFragment(html).querySelectorAll('*')].flatMap((element) =>
      element.getAttributeNames().filter((name) => name.startsWith('on')),
    )
  }

  it('keeps loading="lazy" on rendered images', () => {
    const html = renderGameDescription('[img]https://cdn.example.com/a.png[/img]', APPID)

    const img = parseFragment(html).querySelector('img')
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
