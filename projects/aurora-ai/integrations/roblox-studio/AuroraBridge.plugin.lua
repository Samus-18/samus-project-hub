-- Aurora AI Bridge for Roblox Studio
-- User-driven script context export and explicitly confirmed single-script edits.
-- No HTTP, telemetry, remote code execution, or automatic edits to multiple scripts.
local Selection = game:GetService("Selection")
local ChangeHistoryService = game:GetService("ChangeHistoryService")
local ScriptEditorService = game:GetService("ScriptEditorService")

local toolbar = plugin:CreateToolbar("Aurora AI")
local toggle = toolbar:CreateButton("AuroraBridge", "Open Aurora AI script bridge", "")
toggle.ClickableWhenViewportHidden = true
local widgetInfo = DockWidgetPluginGuiInfo.new(Enum.InitialDockState.Right, false, false, 540, 620, 380, 360)
local widget = plugin:CreateDockWidgetPluginGui("AuroraAIBridge", widgetInfo)
widget.Title = "Aurora AI · Roblox Bridge"

local root = Instance.new("Frame")
root.Size = UDim2.fromScale(1, 1)
root.BackgroundColor3 = Color3.fromRGB(18, 24, 39)
root.Parent = widget
local layout = Instance.new("UIListLayout")
layout.Padding = UDim.new(0, 8)
layout.SortOrder = Enum.SortOrder.LayoutOrder
layout.Parent = root
local padding = Instance.new("UIPadding")
padding.PaddingTop = UDim.new(0, 10)
padding.PaddingBottom = UDim.new(0, 10)
padding.PaddingLeft = UDim.new(0, 8)
padding.PaddingRight = UDim.new(0, 8)
padding.Parent = root

local function label(text, height, order)
    local item = Instance.new("TextLabel")
    item.Size = UDim2.new(1, -16, 0, height or 32)
    item.BackgroundTransparency = 1
    item.TextColor3 = Color3.fromRGB(226, 237, 255)
    item.TextWrapped = true
    item.Font = Enum.Font.Gotham
    item.TextSize = 13
    item.TextXAlignment = Enum.TextXAlignment.Left
    item.TextYAlignment = Enum.TextYAlignment.Center
    item.Text = text
    item.LayoutOrder = order or 1
    item.Parent = root
    return item
end

local function button(text, order)
    local item = Instance.new("TextButton")
    item.Size = UDim2.new(1, -16, 0, 36)
    item.BackgroundColor3 = Color3.fromRGB(35, 65, 96)
    item.TextColor3 = Color3.new(1, 1, 1)
    item.Font = Enum.Font.GothamMedium
    item.TextSize = 13
    item.Text = text
    item.LayoutOrder = order or 1
    item.Parent = root
    return item
end

label("Vyber jeden nebo více Scriptů v Exploreru a načti jejich kód do kontextu Aurory. Nic se neposílá automaticky.", 48, 1)
local scan = button("Načíst vybrané skripty", 2)
local source = Instance.new("TextBox")
source.Size = UDim2.new(1, -16, 0, 270)
source.BackgroundColor3 = Color3.fromRGB(9, 14, 25)
source.TextColor3 = Color3.fromRGB(220, 238, 255)
source.Font = Enum.Font.Code
source.TextSize = 12
source.TextXAlignment = Enum.TextXAlignment.Left
source.TextYAlignment = Enum.TextYAlignment.Top
source.TextWrapped = false
source.ClearTextOnFocus = false
source.MultiLine = true
source.Text = "-- Vyber script a stiskni Načíst. Kód můžeš zkopírovat do Aurory nebo upravit zde pro náhled.\n"
source.LayoutOrder = 3
source.Parent = root
local apply = button("Připravit náhled změny…", 4)
local status = label("Zápis je standardně ruční. Změna se nikdy neprovede bez druhého kliknutí a potvrzení.", 62, 5)

local loadedTarget = nil
local loadedOriginal = nil
local pendingApply = nil
local applyBusy = false
local MAX_SOURCE_BYTES = 300000

local function isScript(obj)
    return obj and (obj:IsA("Script") or obj:IsA("LocalScript") or obj:IsA("ModuleScript"))
end

local function readSource(target)
    local ok, text = pcall(function()
        return ScriptEditorService:GetEditorSource(target)
    end)
    if ok and type(text) == "string" then return text end
    local okFallback, fallback = pcall(function() return target.Source end)
    if okFallback and type(fallback) == "string" then return fallback end
    error("Studio nedovolilo bezpečně přečíst zdroj tohoto skriptu. Otevři jej v editoru a zkus to znovu.")
end

local function clearPendingApply()
    pendingApply = nil
    if not applyBusy then apply.Text = "Připravit náhled změny…" end
end

local function setApplyBusy(busy)
    applyBusy = busy
    apply.Active = not busy
    apply.AutoButtonColor = not busy
    if busy then apply.Text = "Zápis probíhá…" end
end

scan.MouseButton1Click:Connect(function()
    if applyBusy then
        status.Text = "Zápis už probíhá. Počkej na výsledek, než načteš další skript."
        return
    end
    clearPendingApply()
    loadedTarget = nil
    loadedOriginal = nil
    local selected = Selection:Get()
    local blocks = {}
    local scriptCount = 0
    local singleTarget = nil
    local singleSource = nil

    for _, obj in ipairs(selected) do
        if isScript(obj) then
            local ok, text = pcall(function() return readSource(obj) end)
            if ok and type(text) == "string" then
                scriptCount += 1
                table.insert(blocks, "-- PATH: " .. obj:GetFullName() .. "\n" .. text)
                if #selected == 1 then
                    singleTarget = obj
                    singleSource = text
                end
            end
        end
    end

    if #blocks == 0 then
        source.Text = "-- Není vybraný žádný čitelný Script / LocalScript / ModuleScript.\n"
        status.Text = "Vyber skript v Exploreru a zkus načtení znovu."
        return
    end

    source.Text = table.concat(blocks, "\n\n-- ===== DALŠÍ SKRIPT =====\n\n")
    if #selected == 1 and singleTarget then
        loadedTarget = singleTarget
        loadedOriginal = singleSource
        status.Text = string.format("Načten: %s · %d řádků. Uprav text, pak připrav náhled a potvrď druhým kliknutím. Před zápisem se znovu ověří, že se zdroj mezitím nezměnil.", singleTarget:GetFullName(), #string.split(singleSource, "\n"))
        apply.Text = "Připravit náhled změny…"
    else
        status.Text = string.format("Načteno %d skriptů jako kontext. Hromadné zapisování je záměrně zakázané; vyber a načti přesně jeden skript, pokud chceš připravit jeho změnu.", scriptCount)
        apply.Text = "Nelze použít více skriptů najednou"
    end
end)

apply.MouseButton1Click:Connect(function()
    if applyBusy then return end
    local selected = Selection:Get()
    if not loadedTarget or not isScript(loadedTarget) or #selected ~= 1 or selected[1] ~= loadedTarget then
        clearPendingApply()
        status.Text = "Pro bezpečný zápis načti přesně jeden skript a ponech jej vybraný. Kontext z více skriptů lze jen zkopírovat do Aurory."
        return
    end

    local proposed = source.Text
    if #proposed > MAX_SOURCE_BYTES then
        clearPendingApply()
        status.Text = "Navržený zdroj přesahuje limit 300 kB. Zkrať kód nebo rozděl změnu na menší kroky."
        return
    end
    if proposed == loadedOriginal then
        clearPendingApply()
        status.Text = "Text je stejný jako původní zdroj; není co měnit."
        return
    end
    if proposed:match("^%s*$") then
        clearPendingApply()
        status.Text = "Prázdný zdroj nebyl aplikován. Pokud chceš script vymazat, udělej to přímo v editoru Studio."
        return
    end

    local okCurrent, current = pcall(function() return readSource(loadedTarget) end)
    if not okCurrent or current ~= loadedOriginal then
        clearPendingApply()
        status.Text = "Zdroj se od načtení změnil nebo jej nelze přečíst. Znovu načti script a zkontroluj změny před schválením."
        return
    end

    if not pendingApply or pendingApply.target ~= loadedTarget or pendingApply.original ~= loadedOriginal or pendingApply.proposed ~= proposed then
        pendingApply = { target = loadedTarget, original = loadedOriginal, proposed = proposed }
        local oldLines = #string.split(loadedOriginal, "\n")
        local newLines = #string.split(proposed, "\n")
        apply.Text = "Potvrdit a aplikovat změnu"
        status.Text = string.format("NÁHLED ZMĚNY · %s\nPůvodně %d řádků / %d znaků → nově %d řádků / %d znaků. Zkontroluj celý obsah v editoru výše. Další kliknutí změnu zapíše s podporou Undo.", loadedTarget:GetFullName(), oldLines, #loadedOriginal, newLines, #proposed)
        return
    end

    -- Revalidate target and source immediately before recording the accepted change.
    if not loadedTarget:IsDescendantOf(game) or not isScript(loadedTarget) then
        clearPendingApply()
        status.Text = "Cílový skript už není v projektu. Změna nebyla provedena."
        return
    end
    local okLatest, latest = pcall(function() return readSource(loadedTarget) end)
    if not okLatest or latest ~= pendingApply.original then
        clearPendingApply()
        status.Text = "Zdroj se změnil těsně před potvrzením. Změna nebyla provedena; načti script znovu."
        return
    end

    local recording = nil
    local recordingOk, recordingResult = pcall(function()
        return ChangeHistoryService:TryBeginRecording("Aurora AI reviewed script change", "Aurora AI: Apply reviewed script")
    end)
    if recordingOk then recording = recordingResult end
    if not recording then
        clearPendingApply()
        status.Text = "Studio nemohlo zahájit Undo záznam (například během Play Testu). Ukonči Play Test a zkus to znovu."
        return
    end

    -- Snapshot every approved value before the yielding editor API call.
    -- The callback must not read mutable pendingApply, which another UI action could clear.
    local target = loadedTarget
    local expectedOriginal = pendingApply.original
    local approvedSource = pendingApply.proposed
    setApplyBusy(true)
    local updateOk, updateError = pcall(function()
        ScriptEditorService:UpdateSourceAsync(target, function(freshSource)
            if freshSource ~= expectedOriginal then
                error("Zdroj se změnil v průběhu zápisu; změna byla odmítnuta.")
            end
            return approvedSource
        end)
    end)
    if not updateOk then
        pcall(function() ChangeHistoryService:FinishRecording(recording, Enum.FinishRecordingOperation.Cancel) end)
        setApplyBusy(false)
        clearPendingApply()
        status.Text = "Zápis se nepodařil; Studio se pokusilo změnu vrátit. Detail: " .. string.sub(tostring(updateError), 1, 280)
        return
    end

    -- A failure to close the Undo recording does not mean the source update failed.
    local commitOk, commitError = pcall(function()
        ChangeHistoryService:FinishRecording(recording, Enum.FinishRecordingOperation.Commit)
    end)
    setApplyBusy(false)
    if not commitOk then
        -- Try to close a dangling recording. Then inspect actual source before telling the user what happened.
        pcall(function() ChangeHistoryService:FinishRecording(recording, Enum.FinishRecordingOperation.Cancel) end)
        local inspectOk, actualSource = pcall(function() return readSource(target) end)
        clearPendingApply()
        if inspectOk and actualSource == expectedOriginal then
            loadedOriginal = expectedOriginal
            source.Text = expectedOriginal
            status.Text = "Undo záznam se nepodařilo dokončit, ale ověřila jsem, že původní zdroj zůstal zachovaný. Zkus to znovu později. Detail: " .. string.sub(tostring(commitError), 1, 180)
        elseif inspectOk and actualSource == approvedSource then
            loadedOriginal = approvedSource
            source.Text = approvedSource
            status.Text = "Zdroj byl aktualizován, ale Studio nedokončilo Undo záznam. Změnu zkontroluj a ověř historii ručně. Detail: " .. string.sub(tostring(commitError), 1, 180)
        else
            status.Text = "Studio nedokončilo Undo záznam a výsledek zápisu nelze bezpečně ověřit. Otevři cílový skript a zkontroluj jej ručně před další úpravou. Detail: " .. string.sub(tostring(commitError), 1, 180)
        end
        return
    end
    loadedOriginal = approvedSource
    source.Text = approvedSource
    clearPendingApply()
    status.Text = "Změna byla aplikována na jeden vybraný skript. V Change History / Undo můžeš akci vrátit. Zkontroluj script a spusť testy ve Studio."
end)

toggle.Click:Connect(function()
    widget.Enabled = not widget.Enabled
end)
