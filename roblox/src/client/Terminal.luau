-- NIGHTFALL / local terminal theater. Rojo owns this module.
local Players = game:GetService("Players")
local Tween = game:GetService("TweenService")
local Input = game:GetService("UserInputService")
local Run = game:GetService("RunService")
local shared = game:GetService("ReplicatedStorage"):WaitForChild("Nightfall")
local catalog = require(shared:WaitForChild("Catalog"))
local identity = require(shared:WaitForChild("Identity"))
local session =
    require(shared:WaitForChild("Session")).new(catalog, require(shared:WaitForChild("Story")))
local pg = Players.LocalPlayer:WaitForChild("PlayerGui")
for _, name in { "NightfallTerminal", "NightfallDossier" } do
    local old = pg:FindFirstChild(name)
    if old then
        old:Destroy()
    end
end
local C = {
    bg = Color3.fromRGB(8, 9, 14),
    panel = Color3.fromRGB(14, 16, 23),
    raised = Color3.fromRGB(22, 24, 33),
    edge = Color3.fromRGB(43, 46, 59),
    muted = Color3.fromRGB(131, 140, 157),
    text = Color3.fromRGB(226, 230, 239),
    accent = Color3.fromRGB(255, 72, 93),
    mint = Color3.fromRGB(125, 244, 197),
    amber = Color3.fromRGB(255, 188, 105),
}
local palettes = {
    crimson = C.accent,
    ghost = Color3.fromRGB(105, 213, 255),
    venom = Color3.fromRGB(170, 245, 102),
}
local accents = {}
local function make(class, parent, props)
    local obj = Instance.new(class)
    for k, v in props do
        obj[k] = v
    end
    obj.Parent = parent
    return obj
end
local function frame(parent, name, x, y, w, h, color)
    return make("Frame", parent, {
        Name = name,
        Position = UDim2.fromOffset(x, y),
        Size = UDim2.fromOffset(w, h),
        BackgroundColor3 = color or C.panel,
        BorderSizePixel = 0,
    })
end
local function label(parent, name, text, x, y, w, h, size, color, font)
    return make("TextLabel", parent, {
        Name = name,
        Text = text,
        Position = UDim2.fromOffset(x, y),
        Size = UDim2.fromOffset(w, h),
        BackgroundTransparency = 1,
        TextColor3 = color or C.text,
        TextSize = size or 14,
        Font = font or Enum.Font.Code,
        TextXAlignment = Enum.TextXAlignment.Left,
        TextYAlignment = Enum.TextYAlignment.Center,
    })
end
local function accent(obj, property)
    table.insert(accents, { obj, property or "TextColor3" })
    obj[property or "TextColor3"] = C.accent
    return obj
end
local function border(obj, color)
    return make("UIStroke", obj, { Color = color or C.edge, Thickness = 1, Transparency = 0.1 })
end
local function corner(obj, r)
    make("UICorner", obj, { CornerRadius = UDim.new(0, r or 10) })
end
local function panel(parent, name, x, y, w, h)
    local p = frame(parent, name, x, y, w, h)
    corner(p)
    border(p)
    return p
end
local function button(parent, name, text, x, y, w, h, action)
    local b = make("TextButton", parent, {
        Name = name,
        Text = text,
        Position = UDim2.fromOffset(x, y),
        Size = UDim2.fromOffset(w, h),
        BackgroundColor3 = C.raised,
        BorderSizePixel = 0,
        TextColor3 = C.text,
        TextSize = 13,
        Font = Enum.Font.Code,
        AutoButtonColor = false,
    })
    corner(b, 6)
    border(b)
    b.MouseEnter:Connect(function()
        Tween:Create(b, TweenInfo.new(0.15), { BackgroundColor3 = Color3.fromRGB(36, 39, 51) })
            :Play()
    end)
    b.MouseLeave:Connect(function()
        Tween:Create(b, TweenInfo.new(0.15), { BackgroundColor3 = C.raised }):Play()
    end)
    b.Activated:Connect(action)
    return b
end
local gui = make("ScreenGui", pg, {
    Name = "NightfallTerminal",
    ResetOnSpawn = false,
    IgnoreGuiInset = true,
    DisplayOrder = 20,
    ZIndexBehavior = Enum.ZIndexBehavior.Sibling,
})
local backdrop = frame(gui, "Backdrop", 0, 0, 0, 0, C.bg)
backdrop.Size = UDim2.fromScale(1, 1)
make("UIGradient", backdrop, {
    Color = ColorSequence.new({
        ColorSequenceKeypoint.new(0, Color3.fromRGB(28, 13, 23)),
        ColorSequenceKeypoint.new(0.5, C.bg),
        ColorSequenceKeypoint.new(1, Color3.fromRGB(10, 21, 26)),
    }),
    Rotation = 25,
})
local canvas = frame(backdrop, "Canvas", 0, 0, 1440, 900)
canvas.BackgroundTransparency = 1
canvas.AnchorPoint = Vector2.new(0.5, 0.5)
canvas.Position = UDim2.fromScale(0.5, 0.5)
local scale = make("UIScale", canvas, { Scale = 1 })
for x = 0, 1440, 48 do
    local l = frame(canvas, "Grid", x, 0, 1, 900, C.edge)
    l.BackgroundTransparency = 0.85
end
for y = 0, 900, 48 do
    local l = frame(canvas, "Grid", 0, y, 1440, 1, C.edge)
    l.BackgroundTransparency = 0.88
end
local badge = panel(canvas, "Badge", 32, 58, 176, 25)
accent(label(badge, "Text", "  // ABYSS DIVISION", 0, 0, 176, 25, 11))
label(canvas, "Title", "NIGHTFALL", 28, 85, 750, 65, 58, C.text, Enum.Font.GothamBlack)
local subtitle = label(
    canvas,
    "Subtitle",
    "OPSEC DEMON / PORCELAIN NIGHTFALL / OPERATOR CONSOLE v2.0",
    32,
    154,
    950,
    25,
    13,
    C.muted
)
local live = label(canvas, "Live", "●  LOCAL SESSION", 1110, 88, 280, 24, 13, C.mint)
local clockText = label(canvas, "Clock", "", 1110, 119, 280, 24, 12, C.muted)
local rule = accent(frame(canvas, "Rule", 32, 193, 1376, 2), "BackgroundColor3")
local nav = frame(canvas, "Navigation", 32, 208, 1376, 40)
nav.BackgroundTransparency = 1
local activeTab = "terminal"
local deck, input, submitLine
local tabs = {}
local function setTab(name)
    activeTab = name
    if deck then
        deck.Visible = name == "commands"
    end
    for key, b in tabs do
        b.TextColor3 = key == name and C.accent or C.muted
    end
end
tabs.terminal = button(nav, "TerminalTab", "01 / TERMINAL", 0, 0, 184, 36, function()
    setTab("terminal")
end)
tabs.commands = button(nav, "CommandsTab", "02 / COMMANDS", 196, 0, 184, 36, function()
    setTab("commands")
end)
tabs.story = button(nav, "StoryTab", "03 / LOST SIGNALS", 392, 0, 200, 36, function()
    setTab("terminal")
    submitLine(session.scene and "novel status" or "novel start")
end)
local notice = label(
    nav,
    "Notice",
    "FICTIONAL SYSTEMS / REAL PROTAGONIST ENERGY",
    720,
    0,
    656,
    36,
    12,
    C.muted
)
notice.TextXAlignment = Enum.TextXAlignment.Right
local operator = panel(canvas, "Operator", 32, 264, 280, 422)
accent(label(operator, "Section", "01 / OPERATOR SEAL", 18, 14, 244, 25, 12))
local seal = frame(operator, "Seal", 16, 52, 248, 226)
seal.BackgroundTransparency = 1
local rings, ringStrokes = {}, {}
for i, d in { 206, 184, 148 } do
    local r = frame(seal, "Ring" .. i, 124, 111, d, d)
    r.AnchorPoint = Vector2.new(0.5, 0.5)
    r.BackgroundTransparency = 1
    corner(r, 999)
    local s = border(r, C.accent)
    s.Transparency = i == 2 and 0.65 or 0.3
    table.insert(ringStrokes, s)
    table.insert(rings, r)
    local dot = accent(frame(r, "Orbit", d / 2 - 3, -3, 6, 6), "BackgroundColor3")
    corner(dot, 999)
end
local glyph = frame(seal, "Demon", 74, 65, 100, 95)
glyph.BackgroundTransparency = 1
local function segment(x, y, w, h, rotation, color)
    local s = frame(glyph, "Stroke", x, y, w, h, color)
    s.Rotation = rotation or 0
    corner(s, 2)
    if not color then
        accent(s, "BackgroundColor3")
    end
end
segment(4, -3, 7, 40, -28)
segment(88, -3, 7, 40, 28)
segment(12, 24, 76, 5)
segment(3, 41, 5, 40, -18)
segment(90, 41, 5, 40, 18)
segment(17, 78, 66, 5)
segment(17, 44, 25, 6, 15, C.mint)
segment(59, 44, 25, 6, -15, C.mint)
segment(46, 57, 8, 10, 45)
segment(30, 68, 40, 3)
for i = 1, 5 do
    segment(28 + i * 7, 69, 3, 8)
end
local mascotText = accent(label(seal, "Mascot", "D E M O N", 0, 216, 248, 20, 12))
mascotText.TextXAlignment = Enum.TextXAlignment.Center
local sceneArt = label(seal, "Scene", "", 0, 28, 248, 170, 17, C.mint)
sceneArt.TextXAlignment = Enum.TextXAlignment.Center
sceneArt.Visible = false
local arts = {
    prostreet = "PROSTREET\n   ______\n _/|_||_|\\_\n(o)------(o)\nSIM RACE FEED",
    vulcan = "TEAMGROUP\n╔══════════╗\n║ VULCAN Z ║\n╚══════════╝\n||||||||||||",
    ["mo-krill"] = "  o     o\n /|\\   /|\\\n / \\   / \\\nMO & KRILL",
    tweeter = " \\_/ \\_/\n [o] [o]\n |  ~  |\n |( O )|\nTWEETER",
    krillion = "· * · * ·\n * · * · *\n· * · * ·\nA THOUSAND LIGHTS",
    ["chinese-demon"] = " /\\     /\\\n( o  v  o )\n  \\___/\nSIGNAL DEMON",
    porcelain = "    (  )\n     )(\n    /  \\\n   /____\\\n  / o  o \\\n /________\\",
}
local callsignText =
    label(operator, "Callsign", "KRILLION", 18, 298, 244, 30, 24, C.text, Enum.Font.GothamBold)
local rankText = label(operator, "Rank", "GHOST IN THE WIRES", 18, 333, 244, 23, 11, C.muted)
label(operator, "Clearance", "CLEARANCE  FICTIONAL / 999", 18, 369, 244, 20, 11, C.mint)
local mission = panel(canvas, "Mission", 32, 700, 280, 139)
accent(label(mission, "Section", "02 / ACTIVE DIRECTIVE", 18, 10, 244, 24, 12))
local missionTitle = label(mission, "Title", "", 18, 40, 244, 24, 13)
local missionBody = label(mission, "Body", "", 18, 67, 244, 57, 12, C.muted)
missionBody.TextWrapped = true
missionBody.TextYAlignment = Enum.TextYAlignment.Top
local console = panel(canvas, "Console", 328, 264, 756, 575)
local consoleHead = frame(console, "Header", 0, 0, 756, 48, C.raised)
corner(consoleHead)
local consoleTitle = accent(label(consoleHead, "Title", "  >_ GHOST CONSOLE", 14, 0, 470, 48, 14))
label(consoleHead, "Local", "CLIENT LOCAL / SIM", 548, 0, 190, 48, 11, C.mint).TextXAlignment =
    Enum.TextXAlignment.Right
local output = make("ScrollingFrame", console, {
    Name = "Output",
    Position = UDim2.fromOffset(20, 64),
    Size = UDim2.new(1, -40, 1, -166),
    BackgroundTransparency = 1,
    BorderSizePixel = 0,
    ScrollBarThickness = 3,
    ScrollBarImageColor3 = C.accent,
    AutomaticCanvasSize = Enum.AutomaticSize.Y,
    CanvasSize = UDim2.new(),
    ScrollingDirection = Enum.ScrollingDirection.Y,
})
make("UIListLayout", output, { Padding = UDim.new(0, 9), SortOrder = Enum.SortOrder.LayoutOrder })
make("UIPadding", output, { PaddingBottom = UDim.new(0, 18) })
local outputRows, order = {}, 0
local function append(text, color)
    if text == "" then
        return
    end
    order += 1
    local row = make("TextLabel", output, {
        Name = "Line" .. order,
        Text = text:sub(1, 24000),
        Size = UDim2.new(1, -8, 0, 0),
        AutomaticSize = Enum.AutomaticSize.Y,
        BackgroundTransparency = 1,
        Font = Enum.Font.Code,
        TextSize = 14,
        TextColor3 = color or C.text,
        TextWrapped = true,
        TextXAlignment = Enum.TextXAlignment.Left,
        TextYAlignment = Enum.TextYAlignment.Top,
        LayoutOrder = order,
        RichText = false,
    })
    table.insert(outputRows, row)
    if #outputRows > 80 then
        table.remove(outputRows, 1):Destroy()
    end
    task.defer(function()
        if output.Parent then
            output.CanvasPosition =
                Vector2.new(0, math.max(0, output.AbsoluteCanvasSize.Y - output.AbsoluteSize.Y))
        end
    end)
end
local suggestion = make("TextButton", console, {
    Name = "Suggestion",
    Text = "TAB / click complete / ↑ ↓ history / Ctrl+L clear",
    Position = UDim2.new(0, 20, 1, -91),
    Size = UDim2.new(1, -40, 0, 20),
    BackgroundTransparency = 1,
    TextColor3 = C.muted,
    TextSize = 11,
    Font = Enum.Font.Code,
    TextXAlignment = Enum.TextXAlignment.Left,
    AutoButtonColor = false,
})
local inputBar = frame(console, "InputBar", 18, 0, 720, 48, C.bg)
inputBar.Position = UDim2.new(0, 18, 1, -66)
inputBar.Size = UDim2.new(1, -36, 0, 48)
corner(inputBar, 8)
border(inputBar)
local prompt = accent(label(inputBar, "Prompt", "❯", 14, 0, 24, 48, 22))
input = make("TextBox", inputBar, {
    Name = "CommandInput",
    Position = UDim2.fromOffset(42, 0),
    Size = UDim2.new(1, -148, 1, 0),
    BackgroundTransparency = 1,
    Text = "",
    PlaceholderText = "Type a command. Become the lore.",
    PlaceholderColor3 = C.muted,
    TextColor3 = C.text,
    Font = Enum.Font.Code,
    TextSize = 14,
    TextXAlignment = Enum.TextXAlignment.Left,
    ClearTextOnFocus = false,
    MultiLine = false,
})
local submit = button(inputBar, "Execute", "EXEC ↵", 614, 7, 96, 34, function()
    local text = input.Text
    input.Text = ""
    submitLine(text)
end)
submit.Position = UDim2.new(1, -104, 0, 7)
local right = panel(canvas, "Telemetry", 1100, 264, 308, 255)
accent(label(right, "Section", "03 / SIGNAL INTELLIGENCE", 18, 12, 272, 24, 12))
local graphTitle =
    label(right, "GraphTitle", "SYNTHETIC SIGNAL / SIM", 18, 43, 272, 20, 11, C.muted)
local graph = frame(right, "Graph", 18, 80, 272, 109, C.bg)
corner(graph, 4)
for y = 0, 3 do
    local l = frame(graph, "Grid", 0, y * 36, 272, 1, C.edge)
    l.BackgroundTransparency = 0.55
end
local bars = {}
for i = 1, 40 do
    local b = accent(frame(graph, "Sample" .. i, (i - 1) * 6.8, 108, 4, 30), "BackgroundColor3")
    b.AnchorPoint = Vector2.new(0, 1)
    table.insert(bars, b)
end
local graphCaption =
    label(right, "Caption", "DECORATIVE FEED / NO NETWORK TRAFFIC", 18, 200, 272, 19, 10, C.muted)
local graphValue = label(right, "Value", "SIGNAL RAIN / LOCAL", 18, 223, 272, 19, 11, C.mint)
local stats = panel(canvas, "Reactor", 1100, 533, 308, 143)
accent(label(stats, "Section", "04 / REACTOR STATE", 18, 10, 272, 24, 12))
local bankText = label(stats, "Bank", "VIRTUAL BANK OFFLINE", 18, 43, 272, 20, 12)
local track = frame(stats, "Track", 18, 70, 272, 4, C.edge)
corner(track, 3)
local fill = accent(frame(track, "Fill", 0, 0, 1, 4), "BackgroundColor3")
local energyText = label(stats, "Energy", "LARP ENERGY 000 / 999", 18, 86, 272, 20, 12, C.mint)
local eggText = label(stats, "Egg", "GOLDEN GOOSE EMPTY NEST", 18, 112, 272, 20, 11, C.muted)
local quick = panel(canvas, "QuickActions", 1100, 690, 308, 149)
accent(label(quick, "Section", "05 / OPERATOR ACTIONS", 18, 9, 272, 24, 12))
button(quick, "Ping", "UPLINK", 18, 43, 130, 38, function()
    submitLine(session.uplink and "mainframe ping" or "mainframe local")
end)
button(quick, "Ritual", "RITUAL", 160, 43, 130, 38, function()
    submitLine("ritual")
end)
button(quick, "Oracle", "ORACLE", 18, 94, 130, 38, function()
    submitLine("oracle")
end)
button(quick, "Overdrive", "OVERDRIVE", 160, 94, 130, 38, function()
    submitLine("pedal stomp")
end)
deck = frame(console, "CommandDeck", 10, 52, 736, 505, C.panel)
deck.Size = UDim2.new(1, -20, 1, -62)
deck.Visible = false
local search = make("TextBox", deck, {
    Name = "Search",
    Position = UDim2.fromOffset(10, 8),
    Size = UDim2.new(1, -20, 0, 42),
    BackgroundColor3 = C.bg,
    BorderSizePixel = 0,
    Text = "",
    PlaceholderText = "Search commands / lore / simulations…",
    TextColor3 = C.text,
    PlaceholderColor3 = C.muted,
    Font = Enum.Font.Code,
    TextSize = 14,
    ClearTextOnFocus = false,
})
corner(search, 6)
border(search)
local commandList = make("ScrollingFrame", deck, {
    Name = "List",
    Position = UDim2.fromOffset(10, 62),
    Size = UDim2.new(1, -20, 1, -68),
    BackgroundTransparency = 1,
    BorderSizePixel = 0,
    ScrollBarThickness = 3,
    ScrollBarImageColor3 = C.accent,
    CanvasSize = UDim2.new(),
    AutomaticCanvasSize = Enum.AutomaticSize.Y,
})
make(
    "UIListLayout",
    commandList,
    { Padding = UDim.new(0, 8), SortOrder = Enum.SortOrder.LayoutOrder }
)
local commandRows = {}
for i, item in catalog do
    local row = button(commandList, "Command" .. i, "", 0, 0, 710, 66, function()
        setTab("terminal")
        input.Text = item[1]
        input:CaptureFocus()
    end)
    row.Size = UDim2.new(1, -8, 0, 66)
    row.LayoutOrder = i
    accent(label(row, "Command", item[1], 14, 7, 480, 23, 14))
    label(row, "Description", item[2], 14, 33, 640, 22, 11, C.muted)
    local tag = label(row, "Tag", item[3], 0, 8, 90, 22, 10, C.mint)
    tag.Position = UDim2.new(1, -106, 0, 8)
    tag.TextXAlignment = Enum.TextXAlignment.Right
    table.insert(commandRows, { row, string.lower(table.concat(item, " ")) })
end
search:GetPropertyChangedSignal("Text"):Connect(function()
    for _, item in commandRows do
        item[1].Visible = item[2]:find(search.Text:lower(), 1, true) ~= nil
    end
    commandList.CanvasPosition = Vector2.zero
end)
local footer = label(
    canvas,
    "Footer",
    "● FICTIONAL CLEARANCE / NO SHELL EXECUTION / SESSION MEMORY ONLY",
    32,
    855,
    1000,
    24,
    11,
    C.muted
)
local version = label(canvas, "Version", "NIGHTFALL / LUAU / 002", 1120, 855, 288, 24, 11, C.muted)
version.TextXAlignment = Enum.TextXAlignment.Right
local flash = frame(canvas, "Flash", 0, 0, 1440, 900, C.accent)
flash.Size = UDim2.fromScale(1, 1)
flash.BackgroundTransparency = 1
flash.ZIndex = 20
local toast = panel(canvas, "Toast", 440, 90, 560, 60)
toast.Visible = false
toast.ZIndex = 21
local toastText = label(toast, "Text", "", 16, 0, 528, 60, 16, C.accent, Enum.Font.GothamBold)
toastText.TextXAlignment = Enum.TextXAlignment.Center
local particles = {}
for i = 1, 28 do
    local p = frame(
        canvas,
        "Particle" .. i,
        720,
        450,
        4 + (i % 3) * 2,
        4 + (i % 3) * 2,
        i % 2 == 0 and C.mint or C.accent
    )
    p.BackgroundTransparency = 1
    p.ZIndex = 22
    table.insert(particles, p)
end
local effectSerial = 0
local function effect(name)
    if not name then
        return
    end
    if name == "clear" then
        for _, r in outputRows do
            r:Destroy()
        end
        outputRows = {}
        return
    end
    if name == "exit" then
        backdrop.Visible = false
        return
    end
    if name == "rhythm" then
        input:ReleaseFocus()
        return
    end
    if name == "story" then
        return
    end
    flash.BackgroundColor3 = C.accent
    flash.BackgroundTransparency = 0.94
    Tween:Create(flash, TweenInfo.new(0.65), { BackgroundTransparency = 1 }):Play()
    if name == "blast" or name == "cinematic" then
        effectSerial += 1
        local serial = effectSerial
        toast.Visible = true
        toastText.Text = name == "blast" and "SIM / PORCELAIN OVERDRIVE"
            or "THE MAINFRAME KNOWS YOUR NAME"
        for i, p in particles do
            p.Position = UDim2.fromScale(0.5, 0.5)
            p.BackgroundTransparency = 0
            local angle = i / #particles * math.pi * 2
            Tween:Create(p, TweenInfo.new(0.9, Enum.EasingStyle.Quart, Enum.EasingDirection.Out), {
                Position = UDim2.fromScale(
                    0.5 + math.cos(angle) * 0.35,
                    0.5 + math.sin(angle) * 0.35
                ),
                Rotation = i * 45,
                BackgroundTransparency = 1,
            }):Play()
        end
        task.delay(1.6, function()
            if effectSerial == serial then
                toast.Visible = false
            end
        end)
    end
end
local lastTheme = ""
local function repaint()
    if lastTheme == session.theme then
        return
    end
    lastTheme = session.theme
    C.accent = palettes[session.theme]
    for _, item in accents do
        if item[1].Parent then
            item[1][item[2]] = C.accent
        end
    end
    for _, s in ringStrokes do
        s.Color = C.accent
    end
    output.ScrollBarImageColor3 = C.accent
    commandList.ScrollBarImageColor3 = C.accent
    setTab(activeTab)
end
local historyIndex, draft = 1, ""
submitLine = function(line)
    if not line:find("%S") then
        return
    end
    setTab("terminal")
    append(session.callsign:lower() .. "@nightfall ❯ " .. line, C.muted)
    local text, fx, ok = session:run(line)
    append(text, ok and C.text or C.amber)
    effect(fx)
    repaint()
    historyIndex = #session.history + 1
    draft = ""
    gui:SetAttribute("Commands", session.commands)
    gui:SetAttribute("Errors", session.errors)
    gui:SetAttribute("LastCommand", line)
end
input.FocusLost:Connect(function(enter)
    if enter then
        local text = input.Text
        input.Text = ""
        submitLine(text)
    end
end)
local completionOptions = {}
for _, item in catalog do
    table.insert(completionOptions, item[1])
end
for _, s in
    {
        "theme ghost",
        "theme venom",
        "mode prostreet",
        "mode vulcan",
        "mode sdvx",
        "mode demon",
        "novel choose 2",
        "novel reset",
        "novel status",
        "expansion release 64",
        "headphones select ath-r70xa",
        "poopsec off",
        "band off",
        "tailscale milk",
        "ssd status",
        "coins status",
        "sdvx status",
        "hoi4 status",
        "rt60 status",
        "mascot demon",
        "mascot tweeter",
        "mascot krillion",
        "mascot chinese-demon",
    }
do
    table.insert(completionOptions, s)
end
local matchIndex, lastPrefix, matches = 0, "", {}
local function complete()
    local prefix = input.Text:lower():gsub("\t", "")
    if prefix ~= lastPrefix then
        matches = {}
        for _, candidate in completionOptions do
            if candidate:sub(1, #prefix) == prefix then
                table.insert(matches, candidate)
            end
        end
        matchIndex = 0
    end
    if #matches > 0 then
        matchIndex = matchIndex % #matches + 1
        input.Text = matches[matchIndex]
        lastPrefix = input.Text
        input:CaptureFocus()
        input.CursorPosition = #input.Text + 1
    end
end
suggestion.Activated:Connect(complete)

input:GetPropertyChangedSignal("Text"):Connect(function()
    if #input.Text > 4096 then
        input.Text = input.Text:sub(1, 4096)
    end
    local prefix, best = input.Text:lower():gsub("\t", ""), nil
    if prefix ~= "" then
        for _, candidate in completionOptions do
            if candidate:sub(1, #prefix) == prefix and candidate ~= prefix then
                best = candidate
                break
            end
        end
    end
    suggestion.Text = best and "TAB → " .. best or "TAB complete / ↑ ↓ history / Ctrl+L clear"
end)
Input.InputBegan:Connect(function(key, processed)
    local focused = Input:GetFocusedTextBox()
    if focused == input then
        if key.KeyCode == Enum.KeyCode.Up then
            if historyIndex == #session.history + 1 then
                draft = input.Text
            end
            historyIndex = math.max(1, historyIndex - 1)
            input.Text = session.history[historyIndex] or ""
            input.CursorPosition = #input.Text + 1
        elseif key.KeyCode == Enum.KeyCode.Down then
            historyIndex = math.min(#session.history + 1, historyIndex + 1)
            input.Text = session.history[historyIndex] or draft
            input.CursorPosition = #input.Text + 1
        elseif key.KeyCode == Enum.KeyCode.Tab then
            complete()
        elseif
            key.KeyCode == Enum.KeyCode.L
            and (
                Input:IsKeyDown(Enum.KeyCode.LeftControl)
                or Input:IsKeyDown(Enum.KeyCode.RightControl)
            )
        then
            submitLine("clear")
        end
        return
    end
    if focused or processed then
        return
    end
    if key.KeyCode == Enum.KeyCode.Backquote then
        backdrop.Visible = not backdrop.Visible
    elseif key.KeyCode == Enum.KeyCode.Return and backdrop.Visible then
        input:CaptureFocus()
    elseif key.KeyCode == Enum.KeyCode.G and Input:IsKeyDown(Enum.KeyCode.LeftControl) then
        submitLine("pedal stomp")
    elseif session.mode == "sdvx" and backdrop.Visible then
        local map = {
            [Enum.KeyCode.D] = "bt-a",
            [Enum.KeyCode.F] = "bt-b",
            [Enum.KeyCode.J] = "bt-c",
            [Enum.KeyCode.K] = "bt-d",
            [Enum.KeyCode.Q] = "fx-l",
            [Enum.KeyCode.E] = "fx-r",
        }
        if map[key.KeyCode] then
            submitLine("sdvx hit " .. map[key.KeyCode])
        elseif key.KeyCode == Enum.KeyCode.Left then
            submitLine("sdvx knob left -5")
        elseif key.KeyCode == Enum.KeyCode.Right then
            submitLine("sdvx knob right 5")
        end
    end
end)
local launcher = button(gui, "Launcher", "N / OPEN CONSOLE", 18, 0, 180, 38, function()
    backdrop.Visible = true
end)
launcher.Position = UDim2.new(0, 18, 1, -56)
launcher.ZIndex = 30
launcher.Visible = false
local rhythm = frame(console, "RhythmButtons", 20, 0, 710, 38)
rhythm.Position = UDim2.new(0, 20, 1, -142)
rhythm.BackgroundTransparency = 1
rhythm.Visible = false
for i, name in { "bt-a", "bt-b", "bt-c", "bt-d", "fx-l", "fx-r" } do
    button(rhythm, "Hit" .. i, name:upper(), (i - 1) * 112, 0, 104, 32, function()
        submitLine("sdvx hit " .. name)
    end)
end
local lastViewport = Vector2.zero
local function resize(override)
    local view = typeof(override) == "Vector2" and override
        or workspace.CurrentCamera and workspace.CurrentCamera.ViewportSize
        or Vector2.new(1440, 900)
    if view.X < 100 or view.Y < 100 then
        return
    end
    lastViewport = view
    local compact = view.X / view.Y < 1.25
    local w, h = compact and 800 or 1440, compact and 1100 or 900
    canvas.Size = UDim2.fromOffset(w, h)
    scale.Scale = math.min(view.X / w, view.Y / h)
    operator.Visible = not compact
    mission.Visible = not compact
    right.Visible = not compact
    stats.Visible = not compact
    console.Position = UDim2.fromOffset(compact and 24 or 328, 264)
    console.Size = UDim2.fromOffset(compact and 752 or 756, compact and 650 or 575)
    consoleHead.Size = UDim2.new(1, 0, 0, 48)
    quick.Position = UDim2.fromOffset(compact and 24 or 1100, compact and 928 or 690)
    quick.Size = UDim2.fromOffset(compact and 752 or 308, compact and 94 or 149)
    quick.Oracle.Position = UDim2.fromOffset(compact and 310 or 18, compact and 43 or 94)
    quick.Overdrive.Position = UDim2.fromOffset(compact and 456 or 160, compact and 43 or 94)
    nav.Size = UDim2.fromOffset(w - 64, 40)
    rule.Size = UDim2.fromOffset(w - 64, 2)
    notice.Visible = not compact
    live.Visible = not compact
    clockText.Visible = not compact
    version.Visible = not compact
    subtitle.Size = UDim2.fromOffset(w - 64, 25)
    subtitle.Text = compact and "OPSEC DEMON / FICTIONAL OPERATOR CONSOLE"
        or "OPSEC DEMON / PORCELAIN NIGHTFALL / OPERATOR CONSOLE v2.0"
    footer.Position = UDim2.fromOffset(32, compact and 1040 or 855)
    footer.Size = UDim2.fromOffset(w - 64, 24)
    footer.Text = compact and "FICTION / NO SHELL / SESSION MEMORY ONLY"
        or "● FICTIONAL CLEARANCE / NO SHELL EXECUTION / SESSION MEMORY ONLY"
    toast.Position = UDim2.fromOffset((w - 560) / 2, 90)
end
backdrop:GetPropertyChangedSignal("AbsoluteSize"):Connect(resize)
resize()
repaint()
setTab("terminal")
append("NIGHTFALL // BOOT SEQUENCE COMPLETE", C.accent)
append(
    "SUMMONING THE CORE ................... OK\nDECODING THE FORBIDDEN SIGIL .......... OK\nCALIBRATING PROTAGONIST ENERGY ........ EXCESSIVE",
    C.muted
)
append(
    "WELCOME BACK, KRILLION.\nThe cursor blinks. The city holds its breath.\nSomewhere in the wires, a machine remembers you."
)
append(
    "Fictional terminal theater in Roblox.\nUse help, or browse the COMMANDS deck.\nTry mainframe local, novel start, or pedal stomp.",
    C.mint
)
append(
    "SIM / " .. (identity.os or "NIGHTFALL") .. " / " .. (identity.mainframe or "LOCAL MAINFRAME"),
    C.muted
)
local elapsed = 0
Run.RenderStepped:Connect(function(dt)
    elapsed += dt
    if elapsed < 0.05 then
        return
    end
    elapsed = 0
    if workspace.CurrentCamera and workspace.CurrentCamera.ViewportSize ~= lastViewport then
        resize()
    end
    local t = session:age()
    for _, event in session:tick() do
        append(event, C.mint)
        effect("pulse")
    end
    launcher.Visible = not backdrop.Visible
    if not backdrop.Visible then
        return
    end
    for i, r in rings do
        r.Rotation = t * (i % 2 == 0 and -8 or 6) * (i / 2)
    end
    glyph.Position = UDim2.fromOffset(74, 65 + math.sin(t * 1.4) * 3)
    callsignText.Text = session.callsign
    rankText.Text = session:rank()
    mascotText.Text = (
        session.poopsec and "PORCELAIN ARCHDEMON"
        or session.mode ~= "demon" and session.mode .. " / SIM"
        or session.mascot
    ):upper()
    glyph.Visible = session.mode == "demon" and session.mascot == "demon" and not session.poopsec
    sceneArt.Visible = not glyph.Visible
    sceneArt.Text = session.poopsec and arts.porcelain
        or session.mode == "sdvx" and string.format(
            "SDVX / SIM\nL %03d / R %03d\nCOMBO %d\nSCORE %d",
            session.left,
            session.right,
            session.combo,
            session.score
        )
        or arts[session.mode]
        or arts[session.mascot]
        or ""
    consoleTitle.Text = "  >_ "
        .. (session.poopsec and "SEWER MAINFRAME" or session.mode:upper() .. " CONSOLE")
    prompt.Text = session.icons and "❯" or ">"
    local briefing = session:briefing()
    missionTitle.Text = briefing[1]
    missionBody.Text = briefing[2]
    bankText.Text = session.bank and string.format("VIRTUAL BANK %03d / 512 GB", session.used)
        or "VIRTUAL BANK OFFLINE"
    fill.Size = UDim2.fromScale(session.used / 512, 1)
    energyText.Text = string.format("LARP ENERGY %03d / 999", session.larp)
    eggText.Text = session.hatchAt
            and string.format(
                "HATCH CHANNEL %.1f / 2.0s",
                math.min(2, os.clock() - session.hatchAt)
            )
        or session.eggAt and string.format(
            "GOLDEN GOOSE %d SOULS",
            400 + math.floor((os.clock() - session.eggAt) * 80 / 60)
        )
        or "GOLDEN GOOSE EMPTY NEST"
    clockText.Text = os.date("!%H:%M:%S")
        .. string.format(" UTC / %02d:%02d", math.floor(t / 60), math.floor(t % 60))
    rhythm.Visible = session.mode == "sdvx" and not deck.Visible
    output.Size = UDim2.new(1, -40, 1, session.mode == "sdvx" and -214 or -166)
    if session.graph == "room" then
        graphTitle.Text = "SABINE DECAY / IDEALIZED MODEL"
        graphCaption.Text = "0 dB → -60 dB / NOT MEASURED AUDIO"
        graphValue.Text = string.format("RT60 %.3f s", session:rt60())
    elseif session.graph == "metrics" then
        graphTitle.Text = "COMMAND OUTCOMES / ACTUAL SESSION"
        graphCaption.Text = "ACCEPTED / ERROR / LAST 40 COMMANDS"
        graphValue.Text = string.format("%d COMMANDS / %d ERRORS", session.commands, session.errors)
    elseif session.graph == "headphones" then
        graphTitle.Text = "INVENTED EQ / " .. session.headphones:upper()
        graphCaption.Text = "20 Hz → 20 kHz / UNMEASURED PROFILE"
        graphValue.Text = "DISPLAY MODEL / NO AUDIO CONFIGURATION"
    else
        graphTitle.Text = "SYNTHETIC SIGNAL / SIM"
        graphCaption.Text = "DECORATIVE FEED / NO NETWORK TRAFFIC"
        graphValue.Text = "LOCAL / " .. (session.uplink and "SIM UPLINK ONLINE" or "UPLINK OFFLINE")
    end
    for i, b in bars do
        local value, color = 0, C.accent
        if session.graph == "room" then
            value = 1 - (i - 1) / 39
        elseif session.graph == "headphones" then
            value = (session:eq(20 * 1000 ^ ((i - 1) / 39)) + 4) / 8
        elseif session.graph == "metrics" then
            local result = session.outcomes[i]
            value = result == nil and 0.02 or result and 0.8 or 0.35
            color = result == false and C.amber or C.mint
        else
            value = 0.12
                + math.abs(math.sin(i * 0.45 + t * 1.8) * math.cos(i * 0.19 - t * 0.6)) * 0.85
        end
        b.Size = UDim2.fromOffset(4, math.max(2, value * 100))
        b.BackgroundColor3 = color
    end
end)
game:GetService("ProximityPromptService").PromptTriggered:Connect(function(proximityPrompt)
    if proximityPrompt.Parent and proximityPrompt.Parent.Name == "OpSecULAR97Mainframe" then
        backdrop.Visible = true
        submitLine("opsecular boot")
        submitLine("opsecular terminal")
    end
end)
return { Submit = submitLine, Resize = resize, Session = session, Complete = complete }
