-- In-memory OS props, matching the native terminal's fictional operating system.
local Opsecular = {}
Opsecular.__index = Opsecular
function Opsecular.new()
    return setmetatable(
        { booted = false, bin = {}, operator = "mo-krill", spirit = 0, poutine = 0, explosions = 0 },
        Opsecular
    )
end
function Opsecular:support()
    local krillion = self.operator == "krillion"
    return string.format(
        "SIM / INVENTED BALANCE / %s\nLane pressure %d / heal %d / spirit %d\nNo live game stats or gameplay effects.",
        self.operator,
        krillion and 97 or 72,
        (krillion and 120 or 100) + self.spirit * (krillion and 12 or 3),
        self.spirit
    )
end
function Opsecular:command(a)
    local v = a[2] or "desktop"
    if v == "desktop" or v == "status" then
        return string.format(
            "SIM / OPSECULAR 97 / %s\n[Terminal] [Recycle Bin] [WSL 中文] [Lost Signals] [Poutine]\nOperator %s / spirit %d / poutine %d/5\nopsecular boot | terminal | wsl | recycle ls|put NAME|restore NAME|empty\nopsecular operator krillion|mo-krill | spirit N | support | poutine next|reset | weep N",
            self.booted and "ONLINE" or "OFFLINE",
            self.operator,
            self.spirit,
            self.poutine
        )
    end
    if v == "boot" then
        self.booted = true
        return "SIM / OPSECULAR 97 BOOTED / LARP CITY / SERIAL LOOPBACK", "cinematic"
    end
    assert(self.booted, "opsecular boot first")
    if v == "terminal" then
        return "SIM / OPSECULAR97::LARP-CITY::MAINFRAME\nLost Signals / novel start\nLocal serial loopback prop / no network or hardware connection."
    elseif v == "wsl" then
        return "SIM / WSL 中文 / 欢迎操作员 / 主机已连接 / 回收站就绪\nConsole illustration; real WSL stays in the native terminal."
    elseif v == "operator" then
        assert(a[3] == "krillion" or a[3] == "mo-krill", "Unknown operator")
        self.operator = a[3]
        return self:support()
    elseif v == "spirit" then
        local n = tonumber(a[3])
        assert(n and n >= 0 and n <= 10000 and n % 1 == 0, "Spirit must be 0..10000")
        self.spirit = n
        return self:support()
    elseif v == "support" then
        return self:support()
    elseif v == "recycle" then
        local action, name = a[3], a[4]
        if action == "ls" then
            local out = {}
            for item in self.bin do
                table.insert(out, item)
            end
            table.sort(out)
            return "SIM / RECYCLE BIN / " .. table.concat(out, ", ")
        end
        if action == "empty" then
            self.bin = {}
            return "SIM / BIN EMPTIED / no host files changed"
        end
        assert(
            name and #name > 0 and #name <= 80 and not name:find("%c"),
            "Prop name: 1..80 printable characters"
        )
        if action == "put" then
            local count = 0
            for _ in self.bin do
                count += 1
            end
            assert(count < 32, "Recycle bin full / 32 props")
            self.bin[name] = true
            return "SIM / RECYCLED " .. name
        elseif action == "restore" then
            assert(self.bin[name], "No such recycled prop")
            self.bin[name] = nil
            return "SIM / RESTORED " .. name
        end
    elseif v == "poutine" then
        if a[3] == "reset" then
            self.poutine = 0
            return "SIM / KITCHEN RESET"
        end
        assert(a[3] == "next" and self.poutine < 5, "poutine next|reset / five stages")
        local steps = {
            "Wash and cut potatoes; rinse and dry fries.",
            "Fictional fryer: 160 C, drain, then crisp at 190 C.",
            "Whisk butter and flour; add stock and simmer gravy.",
            "Add squeaky cheese curds; ladle gravy over hot fries.",
            "Bon appétit, opérateur!",
        }
        self.poutine += 1
        return "SIM / POUTINE " .. self.poutine .. "/5 / " .. steps[self.poutine]
    elseif v == "weep" then
        local n = tonumber(a[3])
        assert(n and n >= 1 and n <= 16 and n % 1 == 0, "weep 1..16")
        self.explosions += n
        return "SIM / TERMINAL WEEPS / " .. n .. " cartoon detonations / mainframe respawned",
            "blast"
    end
    error("Unknown OpSecULAR prop command; opsecular desktop", 0)
end
return Opsecular
