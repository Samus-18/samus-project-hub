(()=>{"use strict";
 const btn=document.getElementById("lang");let language="cs";
 try{language=localStorage.getItem("samus-project-hub-lang")==="en"?"en":"cs"}catch(_){}
 function setLanguage(lang){language=lang;document.documentElement.lang=lang;for(const node of document.querySelectorAll("[data-en]")){if(!node.dataset.cs)node.dataset.cs=node.textContent;node.textContent=lang==="en"?node.dataset.en:node.dataset.cs;}btn.textContent=lang==="en"?"CZ":"EN";try{localStorage.setItem("samus-project-hub-lang",lang)}catch(_){}}
 btn.addEventListener("click",()=>setLanguage(language==="en"?"cs":"en"));setLanguage(language);
 const endpoint="https://api.github.com/repos/Samus-18/samus-project-hub/releases";
 const prefix={"aurora-ai":"aurora-ai-v","horizon-boost":"horizon-boost-v","thalvoryn":"thalvoryn-v"};
 fetch(endpoint,{headers:{"Accept":"application/vnd.github+json"}}).then(response=>{if(!response.ok)throw new Error("offline");return response.json()}).then(releases=>{if(!Array.isArray(releases))return;
   for(const [name,tag] of Object.entries(prefix)){const matched=releases.filter(r=>!r.draft&&typeof r.tag_name==="string"&&r.tag_name.startsWith(tag)).sort((a,b)=>Date.parse(b.published_at||b.created_at)-Date.parse(a.published_at||a.created_at));if(!matched.length)continue;const current=matched[0];const label=document.getElementById("rel-"+name);if(label)label.textContent=current.tag_name.replace(tag,"v")+(current.prerelease?" · TEST":"");const link=document.querySelector('[data-project="'+name+'"]');if(link && /^https:\/\/github\.com\/Samus-18\/samus-project-hub\/releases\/tag\//.test(current.html_url||""))link.href=current.html_url;}
 }).catch(()=>{});
})();