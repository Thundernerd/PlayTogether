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
            if (!/^https?:\/\//i.test(attr.value)) {
              element.removeAttribute(attr.name)
            }
            continue
          }

          if (tag === 'img' && (attr.name === 'src' || attr.name === 'alt')) {
            if (attr.name === 'src' && !/^https?:\/\//i.test(attr.value)) {
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

export function bbcodeToHtml(input: string): string {
  let html = escapeHtml(input)

  html = html.replace(/\[url=(.*?)\]([\s\S]*?)\[\/url\]/gi, (_, href, label) => {
    const safeHref = String(href).trim()
    const safeLabel = String(label).trim()
    if (!/^https?:\/\//i.test(safeHref)) {
      return safeLabel
    }
    return `<a href="${safeHref}" target="_blank" rel="noopener noreferrer">${safeLabel}</a>`
  })

  html = html.replace(/\[url\]([\s\S]*?)\[\/url\]/gi, (_, href) => {
    const safeHref = String(href).trim()
    if (!/^https?:\/\//i.test(safeHref)) {
      return safeHref
    }
    return `<a href="${safeHref}" target="_blank" rel="noopener noreferrer">${safeHref}</a>`
  })

  html = html.replace(/\[img\]([\s\S]*?)\[\/img\]/gi, (_, src) => {
    const safeSrc = String(src).trim()
    if (!/^https?:\/\//i.test(safeSrc)) {
      return ''
    }
    return `<img src="${safeSrc}" alt="" loading="lazy" />`
  })

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

export function renderGameDescription(description?: string | null): string {
  if (!description?.trim()) {
    return ''
  }

  if (description.includes('[') && description.includes(']')) {
    return bbcodeToHtml(description)
  }

  return sanitizeHtml(escapeHtml(description).replace(/\n/g, '<br />'))
}
