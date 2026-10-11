local identity = require("../src/shared/Identity")
assert(identity.clearance == "FICTIONAL / ABYSS-999")
print(identity.operation .. " / " .. identity.callsign)
