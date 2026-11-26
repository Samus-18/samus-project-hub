const decodeSearchText = (value = '') => String(value)
  .replace(/<[^>]*>/g, '')
  .replace(/&nbsp;|&#160;/gi, ' ')
  .replace(/&amp;/gi, '&')
  .replace(/&quot;|&#34;/gi, '"')
  .replace(/&#x27;|&#39;|&apos;/gi, "'")
  .replace(/&lt;/gi, '<')
  .replace(/&gt;/gi, '>')
  .replace(/&#(\d+);/g, (_, n) => String.fromCodePoint(Math.min(0x10ffff, Number(n) || 32)))
  .replace(/&#x([0-9a-f]+);/gi, (_, n) => String.fromCodePoint(Math.min(0x10ffff, parseInt(n, 16) || 32)))
  .replace(/\s+/g, ' ')
  .trim();

function safeSearchUrl(raw) {
  try {
    // DDG often returns protocol-relative click-tracking URLs.
    let url = new URL(String(raw), 'https://duckduckgo.com/');
    if ((url.hostname === 'duckduckgo.com' || url.hostname.endsWith('.duckduckgo.com')) && url.searchParams.has('uddg')) {
      const target = url.searchParams.get('uddg');
      if (!target) return null;
      url = new URL(target);
    }
    if (url.protocol !== 'https:' || !url.hostname || url.username || url.password) return null;
    const host = url.hostname.toLowerCase().replace(/^\[|\]$/g, '');
    if (host === 'localhost' || host.endsWith('.localhost') || host.endsWith('.local') || host.endsWith('.internal') ||
        host === '::1' || host === '0.0.0.0' || host.startsWith('127.') || host.startsWith('10.') ||
        host.startsWith('192.168.') || /^169\.254\./.test(host) || /^172\.(1[6-9]|2[0-9]|3[01])\./.test(host) ||
        /^\d{1,3}(\.\d{1,3}){3}$/.test(host) || host.includes(':')) return null;
    return url.toString();
  } catch {
    return null;
  }
}

function parseDuckDuckGoHtml(html, source = 'DuckDuckGo', limit = 10) {
  const results = [];
  const seen = new Set();
  const anchors = [...String(html || '').matchAll(/<a\b([^>]*)>([\s\S]*?)<\/a>/gi)];
  for (let i = 0; i < anchors.length && results.length < limit; i++) {
    const match = anchors[i];
    const attrs = match[1];
    const className = (attrs.match(/\bclass\s*=\s*["']([^"']*)["']/i) || [])[1] || '';
    if (!/(?:^|\s)(?:result__a|result-link)(?:\s|$)/i.test(className)) continue;
    const rawHref = (attrs.match(/\bhref\s*=\s*["']([^"']+)["']/i) || [])[1];
    if (!rawHref) continue;
    const url = safeSearchUrl(decodeSearchText(rawHref));
    if (!url || seen.has(url)) continue;
    const title = decodeSearchText(match[2]);
    if (!title) continue;
    const offset = match.index + match[0].length;
    const nearby = String(html).slice(offset, Math.min(String(html).length, offset + 3200));
    const snippetMatch = nearby.match(/<(?:a|div|td|span)\b([^>]*)class\s*=\s*["'][^"']*(?:result__snippet|result-snippet)[^"']*["'][^>]*>([\s\S]*?)<\/(?:a|div|td|span)>/i);
    const snippet = decodeSearchText(snippetMatch?.[2] || '');
    results.push({ title, url, snippet, domain: new URL(url).hostname, source });
    seen.add(url);
  }
  return results;
}

module.exports = { decodeSearchText, safeSearchUrl, parseDuckDuckGoHtml };
