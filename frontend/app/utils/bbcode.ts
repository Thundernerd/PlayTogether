import { steamAppImageBase } from '~/utils/steamImages'

const ALLOWED_TAGS = new Set([
  'p',
  'br',
  'strong',
  'b',
  'em',
  'i',
  'u',
  'ul',
  'ol',
  'li',
  'h1',
  'h2',
  'h3',
  'h4',
  'a',
  'img',
  'span',
])

function isHttpUrl(value: string): boolean {
  return /^https?:\/\//i.test(value)
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

function sanitizeHtml(html: string): string {
  if (typeof window === 'undefined') {
    return escapeHtml(html)
  }

  const template = document.createElement('template')
  template.innerHTML = html

  const walk = (node: Node) => {
    for (const child of [...node.childNodes]) {
      if (child.nodeType === Node.ELEMENT_NODE) {
        const element = child as HTMLElement
        const tag = element.tagName.toLowerCase()

        if (!ALLOWED_TAGS.has(tag)) {
          const text = document.createTextNode(element.textContent ?? '')
          element.replaceWith(text)
          continue
        }

        for (const attr of [...element.attributes]) {
          if (tag === 'a' && attr.name === 'href') {
            if (!isHttpUrl(attr.value)) {
              element.removeAttribute(attr.name)
            }
            continue
          }

          if (tag === 'img' && (attr.name === 'src' || attr.name === 'alt')) {
            if (attr.name === 'src' && !isHttpUrl(attr.value)) {
              element.removeAttribute(attr.name)
            }
            continue
          }

          if (attr.name === 'class' || attr.name === 'style') {
            continue
          }

          element.removeAttribute(attr.name)
        }
      }

      walk(child)
    }
  }

  walk(template.content)
  return template.innerHTML
}

// Matches `{STEAM_APP_IMAGE}` and the mangled `http://STEAM_APP_IMAGE}` Steam has also sent.
const STEAM_APP_IMAGE_PLACEHOLDER = /^(?:https?:\/\/)?\{?STEAM_APP_IMAGE\}/i

function resolveImageUrl(raw: string | null | undefined, appid?: number | null): string | null {
  let url = raw?.trim()
  if (!url) return null
  if (STEAM_APP_IMAGE_PLACEHOLDER.test(url)) {
    if (appid == null) return null
    url = url.replace(STEAM_APP_IMAGE_PLACEHOLDER, steamAppImageBase(appid))
  }
  return isHttpUrl(url) ? url : null
}

function imageTag(raw: string | null | undefined, appid?: number | null): string {
  const src = resolveImageUrl(raw, appid)
  return src ? `<img src="${src}" alt="" loading="lazy" />` : ''
}

/** Picks the still-image URL from an escaped `[img …]` tag body (after `img`). */
function imageAttributeUrl(tagBody: string): string | null {
  if (tagBody.startsWith('=')) {
    return tagBody.slice(1).trim().split(/\s+/)[0] ?? null
  }
  const attrs: Record<string, string> = {}
  for (const match of tagBody.matchAll(/(\w+)=(?:&quot;([\s\S]*?)&quot;|(\S+))/g)) {
    attrs[match[1]!.toLowerCase()] = match[2] ?? match[3] ?? ''
  }
  return attrs.src || attrs.poster || null
}

export function bbcodeToHtml(input: string, appid?: number | null): string {
  let html = escapeHtml(input)

  html = html.replace(/\[url=(.*?)\]([\s\S]*?)\[\/url\]/gi, (_, href, label) => {
    const safeHref = String(href).trim()
    const safeLabel = String(label).trim()
    if (!isHttpUrl(safeHref)) {
      return safeLabel
    }
    return `<a href="${safeHref}" target="_blank" rel="noopener noreferrer">${safeLabel}</a>`
  })

  html = html.replace(/\[url\]([\s\S]*?)\[\/url\]/gi, (_, href) => {
    const safeHref = String(href).trim()
    if (!isHttpUrl(safeHref)) {
      return safeHref
    }
    return `<a href="${safeHref}" target="_blank" rel="noopener noreferrer">${safeHref}</a>`
  })

  html = html.replace(/\[img\]([\s\S]*?)\[\/img\]/gi, (_, src) => imageTag(src, appid))

  // Steam's attribute forms: [img src="…" poster="…" …][/img] and [img=URL …].
  // The input is already escaped here, so quotes appear as &quot;.
  html = html.replace(/\[img([\s=][^\]]*)\](?:\s*\[\/img\])?/gi, (_, tagBody) =>
    imageTag(imageAttributeUrl(String(tagBody)), appid),
  )
  html = html.replace(/\[\/img\]/gi, '')

  html = html.replace(/\[\/?(h1|h2|h3|h4|p|b|i|u|strong|em|list|olist)\]/gi, (match, tag) => {
    const normalized = String(tag).toLowerCase()
    const closing = match.startsWith('[/')
    if (normalized === 'list') {
      return closing ? '</ul>' : '<ul>'
    }
    if (normalized === 'olist') {
      return closing ? '</ol>' : '<ol>'
    }
    return closing ? `</${normalized}>` : `<${normalized}>`
  })

  html = html.replace(/\[\*\]/g, '<li>')
  html = html.replace(/\n/g, '<br />')

  return sanitizeHtml(html)
}

export function renderGameDescription(description?: string | null, appid?: number | null): string {
  if (!description?.trim()) {
    return ''
  }

  if (description.includes('[') && description.includes(']')) {
    return bbcodeToHtml(description, appid)
  }

  return sanitizeHtml(escapeHtml(description).replace(/\n/g, '<br />'))
}
