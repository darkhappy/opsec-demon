--!strict
local identity = require(game:GetService("ReplicatedStorage").Nightfall.Identity)
print(
    string.format("[SIM] %s / %s / %s", identity.operation, identity.callsign, identity.clearance)
)

local prop = Instance.new("Part")
prop.Name = "OpSecULAR97Mainframe"
prop.Size = Vector3.new(6, 4, 2)
prop.Position = Vector3.new(0, 4, 0)
prop.Anchored = true
prop.Color = Color3.fromRGB(8, 12, 19)
prop:SetAttribute("MainframeId", identity.mainframe)
prop:SetAttribute("OperatingSystem", identity.os)
prop:SetAttribute("SerialBus", identity.serial)
prop:SetAttribute("City", identity.city)
prop.Parent = workspace

local display = Instance.new("SurfaceGui")
display.Face = Enum.NormalId.Front
display.Parent = prop
local label = Instance.new("TextLabel")
label.Size = UDim2.fromScale(1, 1)
label.BackgroundColor3 = prop.Color
label.TextColor3 = Color3.fromRGB(114, 255, 173)
label.Font = Enum.Font.Code
label.TextScaled = true
label.Text = "[SIM] "
    .. identity.os
    .. "\n[Terminal] [Recycle Bin] [WSL 中文]\n🔥🔥🔥 Larp City"
label.Parent = display

local prompt = Instance.new("ProximityPrompt")
prompt.ActionText = "Connect terminal prop"
prompt.ObjectText = identity.os .. " / SIM"
prompt.HoldDuration = 0
prompt.Parent = prop
prompt.Triggered:Connect(function(player)
    print("[SIM] Operator " .. player.Name .. " identified " .. identity.mainframe)
    print("[SIM] Connected locally to " .. identity.city .. " / " .. identity.serial)
    print("[SIM] Terminal novel link: " .. identity.novel .. " / use novel start in the Rust shell")
    print("[SIM] " .. identity.operator .. " 🔥🔥🔥")
end)
print("[SIM] Mainframe prop registered: " .. identity.mainframe .. " / " .. identity.serial)
