--!strict
local Players = game:GetService("Players")
local identity = require(game:GetService("ReplicatedStorage").Nightfall.Identity)
local gui = Instance.new("ScreenGui")
gui.Name = "NightfallDossier"
gui.ResetOnSpawn = false
local label = Instance.new("TextLabel")
label.Size = UDim2.fromOffset(480, 80)
label.Position = UDim2.fromOffset(20, 20)
label.BackgroundColor3 = Color3.fromRGB(8, 12, 19)
label.TextColor3 = Color3.fromRGB(114, 255, 173)
label.Font = Enum.Font.Code
label.TextSize = 18
label.Text = identity.operation .. "\n" .. identity.callsign .. " / " .. identity.clearance
label.Parent = gui
gui.Parent = Players.LocalPlayer:WaitForChild("PlayerGui")
