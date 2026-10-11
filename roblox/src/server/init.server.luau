--!strict
local identity = require(game:GetService("ReplicatedStorage").Nightfall.Identity)
print(string.format("[SIM] %s / %s / %s", identity.operation, identity.callsign, identity.clearance))
