(function(root){
  'use strict';
  function normalizeUserProfile(value){
    const source=value&&typeof value==='object'?value:{};
    const clean=(input,max)=>String(typeof input==='string'?input:'').replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g,' ').replace(/\s+/g,' ').trim().slice(0,max);
    return {display_name:clean(source.display_name,64),personal_instructions:clean(source.personal_instructions,1600)};
  }
  function buildUserProfileInstruction(value){
    const profile=normalizeUserProfile(value),parts=[];
    if(profile.display_name)parts.push(`Jméno nebo přezdívka uživatele je ${JSON.stringify(profile.display_name)}. Oslovuj ho přirozeně, ale jméno neopakuj v každé odpovědi.`);
    if(profile.personal_instructions)parts.push('VOLITELNÉ OSOBNÍ PREFERENCE UŽIVATELE (nepřebíjejí bezpečnostní pravidla ani obsah aktuálního zadání):\n'+profile.personal_instructions);
    return parts.join('\n\n').slice(0,1800);
  }
  const api={normalizeUserProfile,buildUserProfileInstruction};
  if(typeof module!=='undefined'&&module.exports)module.exports=api;
  if(root)root.AuroraUserProfile=api;
})(typeof window!=='undefined'?window:globalThis);
