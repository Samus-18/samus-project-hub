(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.AuroraIntent = api;
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';
  function detectImageRequest(text) {
    const original = String(text || '').trim();
    if (!original || original.length < 3) return null;
    const forced = /^\/(?:image|img)\b/i.test(original) || /^\[(?:image|img)\]\s*/i.test(original);
    if (/^(jak|how|co je|what is|návod|navod|tutorial|guide|vysvětli|vysvetli|explain|jak se|how to)\b/i.test(original) && !forced) return null;
    const value = original.replace(/^\/(?:image|img)\s*/i, '').replace(/^\[(?:image|img)\]\s*/i, '').trim();
    const directVerb = /\b(vygeneruj|nakresli|namaluj|ilustruj|renderuj|draw|paint|illustrate|render|skicuj|vytvoř vizuál|vytvor vizual)\b/i.test(value);
    const createVerb = /(?:^|[\s.,!?])(?:vytvoř|vytvor|udělej|udelej|generate|create|make)(?=$|[\s.,!?])/i.test(value);
    const wantVisual = /\b(?:chci|potřebuji|potrebuji|potřebuju|potrebuju|chtěl bych|chtel bych|chtěla bych|chtela bych|rád bych|rad bych|i want|i need|make me|give me)\b/i.test(value);
    const visualWord = /\b(obrázek|obrazek|obraz|fotku|fotka|fotografii|fotografie|ilustraci|ilustrace|obálku|obal|wallpaper|tapetu|thumbnail|náhled|cover|logo|ikonu|ikonku|poster|plakát|plakat|avatar|image|picture|photo|artwork|illustration|concept art|portrait|background|render|wallpaper|splash screen)\b/i.test(value);
    if (!forced && !directVerb && !(createVerb && visualWord) && !(wantVisual && visualWord)) return null;
    let prompt = value;
    prompt = prompt.replace(/^(?:prosím\s+)?(?:(?:můžeš|muzes|could you|please)\s+)?(?:(?:mi|me)\s+)?(?:vygeneruj|vytvoř|vytvor|udělej|udelej|nakresli|namaluj|ilustruj|renderuj|generate|create|make|draw|paint|illustrate|render|skicuj)\s+/i, '');
    prompt = prompt.replace(/^(?:(?:chci|potřebuji|potrebuji|potřebuju|potrebuju|chtěl bych|chtel bych|chtěla bych|chtela bych|rád bych|rad bych|i want|i need|make me|give me)\s+)?(?:(?:mi|pro mě|pro mne|me)\s+)?(?:(?:nový|novou|nové|an|a|the)\s+)?(?:obrázek|obrazek|obraz|fotku|fotka|fotografii|ilustraci|ilustrace|obálku|obal|wallpaper|tapetu|thumbnail|náhled|cover|logo|ikonu|ikonku|poster|plakát|plakat|avatar|image|picture|photo|artwork|illustration)\s*(?:z|na téma|of|about|s motivem|s tématem|pro|for)?\s*/i, '');
    prompt = prompt.replace(/^(?:mi|me)\s+/i, '').replace(/^(?:a|an|the)\s+/i, '').replace(/^[:\-,]\s*/, '').trim();
    if (/^(?:chci|potřebuji|potrebuji|potřebuju|potrebuju|chtěl bych|chtel bych|chtěla bych|chtela bych|rád bych|rad bych|i want|i need)\b/i.test(prompt)) {
      prompt = prompt.replace(/^(?:chci|potřebuji|potrebuji|potřebuju|potrebuju|chtěl bych|chtel bych|chtěla bych|chtela bych|rád bych|rad bych|i want|i need)\s+/i, '').trim();
    }
    prompt = prompt.replace(/^(?:obrázek|obrazek|fotku|ilustraci|wallpaper|image|photo)\s*[:\-]\s*/i, '').trim();
    return prompt.length >= 3 ? prompt : (forced ? value : null);
  }
  function detectWebResearchRequest(text) {
    const original = String(text || '').trim();
    if (/^\/(?:web|search|research)\b/i.test(original)) return original.replace(/^\/(?:web|search|research)\s*/i, '').trim() || null;
    const value = original;
    const explicit = /\b(?:na internetu|na webu|na stránkách|online|the web|online sources|internet search|web search|z webu|z internetu)\b/i.test(value);
    const verb = /\b(?:hledej|vyhledej|najdi|prohledej|prozkoumej|ověř|over|search|research|look up|find me|find out|find)\b/i.test(value);
    if (!explicit || !verb) return null;
    return (value.replace(/^(?:prosím\s+)?(?:(?:můžeš|muzes|could you|please)\s+)?(?:(?:hledej|vyhledej|najdi|prohledej|prozkoumej|ověř|over|search|research|look up|find me|find out|find)\s+)?(?:(?:mi|me)\s+)?(?:(?:na internetu|na webu|na stránkách|online|the web|online sources|internet|web)\s*)?[:\-,]?\s*/i, '').replace(/^(?:for|about|regarding|ohledně|k tématu)\s+/i, '').trim()) || value;
  }
  function detectProjectRequest(text) {
    const original = String(text || '').trim();
    if (!original) return null;
    const forced = /^\/(?:project|unity|repo)\b/i.test(original);
    if (forced) return original.replace(/^\/(?:project|unity|repo)\s*/i, '').trim() || 'Analyzuj vybraný projekt a nejdřív popiš, co v něm je.';
    // Explicit web research belongs to the web workflow, even if the query mentions a project.
    if (detectWebResearchRequest(original)) return null;
    const hasProject = /\b(?:můj projekt|muj projekt|moje repo|projekt|unity|repository|codebase|project folder|my project|my repository|workspace)\b/i.test(original) || /(?:můj repozitář|muj repozitar)(?=$|[\s.,!?])/i.test(original) || /(?:repozitář|repozitar)(?=$|[\s.,!?])/i.test(original);
    const hasAction = /\b(?:analyzuj|zkontroluj|prohledej|audituj|najdi chyby|oprav|debuguj|projdi|zreviduj|analyze|analyse|inspect|scan|audit|review|debug|fix|check)\b/i.test(original);
    if (!hasProject || !hasAction) return null;
    if (/^(?:jak|how)\s+(?:vytvořit|vytvorit|nastavit|začít|zacat|create|start|set up)\b/i.test(original)) return null;
    return original;
  }
  function detectWorkRequest(text) {
    const original = String(text || '').trim();
    if (!original) return null;
    if (/^\/(?:work|codex|code)\b/i.test(original)) return original.replace(/^\/(?:work|codex|code)\s*/i, '').trim() || 'Pokračuj v pracovním režimu.';
    if (detectImageRequest(original)) return null;
    if (detectWebResearchRequest(original) && !/\b(?:vytvoř|vytvor|oprav|uprav|naprogramuj|implementuj|build|create|fix|edit|modify|code)\b/i.test(original)) return null;
    const action = /(?<![\p{L}\p{N}_])(?:vytvoř|vytvor|udělej|udelej|napiš|napis|naprogramuj|programuj|implementuj|oprav|uprav|předělej|predelej|přestav|prestav|vylepši|vylepsi|přidej|pridej|odeber|odstraň|odstran|refaktoruj|sestav|vybuduj|zprovozni|vytvořit|upravit|opravit|create|build|write|develop|implement|fix|edit|modify|refactor|add|remove|generate|make|code|scaffold)(?![\p{L}\p{N}_])/iu.test(original);
    const artifact = /(?<![\p{L}\p{N}_])(?:aplikaci|aplikace|app|web|webovou stránku|stránku|stranka|website|site|program|software|hru|hra|game|mod|modifikaci|plugin|skript|script|funkci|feature|projekt|repozitář|repozitar|repository|codebase|soubor|souboru|file|ui|rozhraní|rozhrani|komponentu|komponenty|tlačítko|tlacitko|button|desktop|electron|python|javascript|typescript|html|css|api|backend|frontend|workflow|dokument|dokumentu|dokumentaci|zprávu|zprava|report|prezentaci|prezentace|slidy|slides|tabulku|tabulka|excel|sešit|sesit|spreadsheet|manuál|manual|pdf|docx|xlsx|pptx|smlouvu|nabídku|nabidka|životopis|zivotopis|resume|portfolio|knihu|článek|clanek|article|email|e-mail|mail|dopis|letter)(?![\p{L}\p{N}_])/iu.test(original);
    const technical = /\b(?:\.py|\.js|\.ts|\.html|\.css|\.json|\.csv|\.tsv|\.java|\.cs|\.cpp|\.rs|\.go|package\.json|readme|build\.bat|\.bat|\.ps1)\b/i.test(original);
    if (!(action && (artifact || technical))) return null;
    if (/^(?:jak|how)\s+(?:vytvořit|vytvorit|nastavit|začít|zacat|create|start|set up)\b/i.test(original)) return null;
    return original;
  }
  return { detectImageRequest, detectWebResearchRequest, detectProjectRequest, detectWorkRequest };
});
