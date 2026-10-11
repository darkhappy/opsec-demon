-- Pure local state. No remotes, shell execution, telemetry collection or persistence.
local Session = {}
Session.__index = Session
local Opsecular = if script then require(script.Parent.Opsecular) else require("./Opsecular")

local countries = {
    GER = "INDUSTRIAL CIRCUIT",
    SOV = "SNOWBOUND SIGNAL",
    USA = "ARSENAL OF NEON",
    ENG = "ISLAND FREQUENCY",
    FRA = "MIDNIGHT RELAY",
    ITA = "ALPINE AMPLIFIER",
    JAP = "PACIFIC PULSE",
    POL = "AMBER UPLINK",
    CHI = "JADE MAINFRAME",
    PHI = "ARCHIPELAGO OVERDRIVE",
}
local missions = {
    { "ACT I / THE SIGNAL", "mainframe local, then mainframe ping. Make the machine answer." },
    { "ACT II / THE ARCHIVE", "bucket ls. Every legend leaves a paper trail." },
    {
        "ACT III / THE RELAY",
        "bucket put signal.txt acquired, then bucket sync. Route the fiction.",
    },
    { "ACT IV / CLAIM YOUR NAME", "callsign YOUR_NAME. Leave a name in the mythology." },
    { "ACT V / THE OPERATOR", "Build something real. The console is yours now." },
}
local phases = {
    "CHALK SIGILS",
    "SOUL CONDUIT",
    "PATRON WHISPERS",
    "VEIL FRACTURE",
    "SUMMONING",
    "WARD RESET",
}
local buffs = {
    "fire rate",
    "max ammo",
    "cooldown reduction",
    "weapon damage",
    "max health",
    "spirit power",
    "bullet resist",
    "spirit resist",
    "move speed",
    "ability range",
}
local amounts = {
    { 1.5, 3, 0.5, 3, 15, 2, 0.5, 0.5, 0.1, 0.25 },
    { 2, 5, 0.75, 4, 20, 3, 0.75, 0.75, 0.2, 0.5 },
    { 2.5, 7, 1, 6, 30, 4, 1, 1, 0.3, 0.75 },
}
local weightedBuffs = { 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 5, 5, 6, 6, 7, 7, 8, 8, 9, 10 }
local desktop = {
    discord = true,
    rips = true,
    shell = true,
    export = true,
    unset = true,
    cd = true,
    dac = true,
    soundtrack = true,
    ["..advancedttk"] = true,
}

local function check(condition, message)
    if not condition then
        error(message, 0)
    end
end
local function number(text, low, high, integer)
    local value = tonumber(text)
    check(
        value
            and value == value
            and value >= low
            and value <= high
            and (not integer or value % 1 == 0),
        string.format(
            "Expected %s from %s to %s",
            integer and "whole number" or "finite number",
            low,
            high
        )
    )
    return value
end
local function words(line)
    local result, buffer, quote = {}, "", nil
    local active = false
    for i = 1, #line do
        local c = line:sub(i, i)
        if quote then
            if c == quote then
                quote = nil
            else
                buffer ..= c
            end
        elseif c == '"' or c == "'" then
            quote = c
            active = true
        elseif c:match("%s") then
            if active then
                table.insert(result, buffer)
                buffer = ""
                active = false
            end
        else
            buffer ..= c
            active = true
        end
    end
    check(not quote, "Unclosed quote")
    if active then
        table.insert(result, buffer)
    end
    return result
end
local function choice(value, options)
    check(table.find(options, value), "Expected " .. table.concat(options, " / "))
    return value
end

function Session.new(catalog, story, clock)
    local self = setmetatable({}, Session)
    self.catalog = catalog
    self.pages = string.split(story:gsub("\r", ""), "\n---\n")
    self.clock = clock or os.clock
    self.opsecular = Opsecular.new()
    self.yoggies = table.create(5, true)
    self.started = self.clock()
    self.callsign, self.theme, self.mode, self.mascot = "KRILLION", "crimson", "demon", "demon"
    self.history, self.outcomes, self.buffTotals = {}, {}, {}
    self.commands, self.success, self.errors, self.larp = 0, 0, 0, 0
    self.mission, self.souls, self.hatched = 1, 2400, 0
    self.bank, self.used, self.uplink, self.poopsec, self.icons = false, 0, false, false, true
    self.bucket = {
        ["igloo-manifest.txt"] = "SIM: igloo lights online; dance floor armed.",
        ["penguin-passport.txt"] = "SIM: WADDLE-01 / snowflake clearance",
    }
    self.syncs, self.rolls, self.detonations = 0, 0, 0
    self.room = { 5, 4, 3, 0.2 }
    self.left, self.right, self.combo, self.score = 50, 50, 0, 0
    self.balance, self.coins, self.price = 100000, 0, 100
    self.country, self.completed = "PHI", 0
    self.band, self.pedal, self.headphones = true, false, "ath-r70xa"
    self.graph, self.listening = "signal", 0
    return self
end

function Session:age()
    return math.max(0, self.clock() - self.started)
end
function Session:rank()
    return self.poopsec and "PORCELAIN ARCHDEMON"
        or (
            self.success < 5 and "GHOST IN THE WIRES"
            or self.success < 15 and "MAINFRAME REVENANT"
            or self.success < 30 and "NEON ARCHDEMON"
            or "ROOT OF ALL LORE"
        )
end
function Session:briefing()
    return missions[self.mission]
end
function Session:rt60()
    local w, l, h, a = table.unpack(self.room)
    return 0.161 * w * l * h / (2 * (w * l + w * h + l * h) * a)
end
function Session:eq(frequency)
    local f = math.log10(frequency)
    if self.headphones == "ath-r70xa" then
        return 0.6 * (f - 3) + 1.2 * math.exp(-((f - 3.5) / 0.28) ^ 2)
    end
    return 2 / (1 + math.exp((f - 2.5) * 5)) - 1.1 * math.exp(-((f - 3.7) / 0.32) ^ 2)
end
function Session:tick()
    local events, now = {}, self.clock()
    if self.hatchAt and now >= self.hatchAt + 2 then
        local accrued = math.floor((self.hatchAt + 2 - self.eggAt) * 80 / 60)
        local rolls = math.floor(accrued / 80)
        local age = self.hatchAt + 2 - self.started
        local tier = age >= 1800 and 3 or age >= 600 and 2 or 1
        for _ = 1, rolls do
            local i = weightedBuffs[math.random(1, #weightedBuffs)]
            self.buffTotals[i] = (self.buffTotals[i] or 0) + amounts[tier][i]
        end
        self.souls += 400 + accrued
        self.hatched += 1
        self.eggAt, self.hatchAt = nil, nil
        table.insert(
            events,
            string.format(
                "SIM / EGG HATCHED / +%d souls / %d fictional buff rolls / penalties cleared",
                400 + accrued,
                rolls
            )
        )
    end
    if self.focusAt and now >= self.focusAt + 20 then
        self.focusAt = nil
        self.completed += 1
        table.insert(events, "SIM / FOCUS COMPLETE / " .. self.focusName)
    end
    return events
end
function Session:storyPage()
    if not self.scene then
        return "LOST SIGNALS / novel start to enter the original story"
    end
    local prompt = self.scene == 4 and "[1] Knock and listen. [2] Restore power and identify."
        or self.scene == 7 and "[1] Keep the recording. [2] Accept the bargain."
        or self.scene >= 11 and "THE END / novel reset"
        or "novel next"
    return self.pages[self.scene] .. "\n\n" .. prompt
end

function Session:dispatch(a)
    local c, v = a[1], a[2] or "status"
    local now = self.clock()
    if c == "opsecular" then
        return self.opsecular:command(a)
    end
    if c == "yoggies" then
        local date = os.date("!*t")
        local count = date.month == 12 and date.day == 25 and 10 or 5
        for i = #self.yoggies + 1, count do
            self.yoggies[i] = true
        end
        for i = #self.yoggies, count + 1, -1 do
            self.yoggies[i] = nil
        end
        if v == "holidays" then
            return "SIM / BERRY CALENDAR / Christmas Eve Dec 24 / Christmas Day Dec 25 / New Year's Eve Dec 31 / New Year's Day Jan 1\nRoblox port uses UTC date for the Christmas tray bonus."
        end
        if v == "use" then
            local index = a[3] and number(a[3], 1, count, true) or table.find(self.yoggies, true)
            check(index and self.yoggies[index], "Yoggie used or tray empty; yoggies refill")
            self.yoggies[index] = false
        elseif v == "refill" then
            for i = 1, count do
                self.yoggies[i] = true
            end
        else
            choice(v, { "status", "show", "all" })
        end
        local out = { "SIM / YOGGIES / UTC seasonal tray" }
        for i, ready in self.yoggies do
            table.insert(
                out,
                string.format(
                    "%02d / %s / berry sparkle prop",
                    i,
                    ready and "(o*o) READY" or "(   ) USED"
                )
            )
        end
        return table.concat(out, "\n")
    end
    if desktop[c] then
        return "UNAVAILABLE IN ROBLOX / "
            .. c
            .. "\nDesktop /bin/sh, environment, USB discovery, WAV rendering and the Rust combat parser stay in the native terminal. No desktop operation was performed."
    end
    if c == "help" then
        local out = {
            "OPSEC DEMON / COMMAND UPLINK",
            "Use the searchable COMMANDS deck. Quotes group arguments. All SIM state is session-only.",
        }
        for _, item in self.catalog do
            table.insert(out, item[1] .. "  /  " .. item[2])
        end
        return table.concat(out, "\n")
    elseif c == "status" then
        return string.format(
            "SESSION / %.1fs / %d commands / %d accepted / %d errors\n%s / %s\nUplink %s / virtual bank %d GB / SIM score %d",
            self:age(),
            self.commands,
            self.success,
            self.errors,
            self.callsign,
            self:rank(),
            self.uplink and "local simulation" or "offline",
            self.used,
            self.score
        )
    elseif c == "dossier" then
        return "DOSSIER / FICTIONAL ABYSS-999\n"
            .. self.callsign
            .. " / "
            .. self:rank()
            .. "\nCHAOTIC TERMINAL / WANTED FOR EXCESSIVE PROTAGONIST ENERGY\nRoblox Luau / private local session"
    elseif c == "callsign" then
        if a[2] then
            check(
                #a[2] <= 24 and a[2]:match("^[%w_-]+$"),
                "Callsign: 1..24 letters, numbers, _ or -"
            )
            self.callsign = a[2]:upper()
        end
        return "IDENTITY / " .. self.callsign
    elseif c == "theme" then
        if a[2] then
            self.theme = choice(a[2], { "crimson", "ghost", "venom" })
        end
        return "PALETTE / " .. self.theme
    elseif c == "mode" then
        if a[2] then
            self.mode = choice(a[2], { "demon", "prostreet", "vulcan", "sdvx" })
        end
        return "SIM SCENE / " .. self.mode
    elseif c == "mascot" then
        if a[2] then
            self.mascot =
                choice(a[2], { "demon", "mo-krill", "chinese-demon", "tweeter", "krillion" })
        end
        return "SIM COMPANION / " .. self.mascot
    elseif c == "poopsec" or c == "icons" then
        if a[2] then
            self[c] = choice(a[2], { "on", "off" }) == "on"
        else
            self[c] = not self[c]
        end
        return c:upper() .. " / " .. (self[c] and "ONLINE" or "OFFLINE")
    elseif c == "mission" then
        check(not a[2] or a[2] == "next", "mission [next]")
        if a[2] then
            self.mission = math.min(#missions, self.mission + 1)
        end
        return "SIM / " .. table.concat(self:briefing(), "\n")
    elseif c == "oracle" then
        local prophecies = {
            "THEY BUILT A FIREWALL. YOU BUILT A CHARACTER ARC.",
            "THE MAINFRAME DOES NOT DREAM. UNTIL YOU LOG IN.",
            "PROPHECY IS JUST A TODO WITH BETTER TYPOGRAPHY.",
            "THE CITY HAS A MILLION TERMINALS. THIS ONE IS YOURS.",
        }
        return "SIM / INCOMING TRANSMISSION\n"
            .. (
                self.poopsec and "THE FIREWALL HAS BEEN FLUSHED. FICTIONALLY."
                or prophecies[self.commands % #prophecies + 1]
            )
    elseif c == "cinematic" then
        return "SIM / SUMMONING THE CORE\nDECODING THE FORBIDDEN SIGIL\nCALIBRATING PROTAGONIST ENERGY\nWELCOME BACK, "
            .. self.callsign,
            "cinematic"
    elseif c == "matrix" then
        local rows, out = number(a[2] or "12", 1, 64, true), { "SIM / DECORATIVE SIGNAL RAIN" }
        for _ = 1, rows do
            local row = ""
            for _ = 1, 48 do
                local index = math.random(1, 8)
                row ..= string.sub("01ABCDEF", index, index)
            end
            table.insert(out, row)
        end
        return table.concat(out, "\n"), "pulse"
    elseif c == "novel" then
        if v == "start" or v == "reset" then
            self.scene = 1
            self.listening = 0
        elseif v == "next" then
            check(self.scene, "novel start first")
            check(self.scene ~= 4 and self.scene ~= 7, "Choose 1 or 2 before continuing")
            check(self.scene < 11, "Story ended; novel reset")
            self.scene = self.scene == 10 and (self.listening == 2 and 11 or 12) or self.scene + 1
        elseif v == "choose" then
            check(self.scene == 4 or self.scene == 7, "No pending story choice")
            choice(a[3], { "1", "2" })
            if a[3] == "1" then
                self.listening += 1
            end
            self.scene += 1
        else
            check(v == "status", "novel start|next|choose 1|2|status|reset")
        end
        return self:storyPage(), "story"
    elseif c == "psyche" then
        return "FICTIONAL CHARACTER ANALYSIS\nListening choices: "
            .. self.listening
            .. "/2\nArmor is a coping metaphor; the blue cup is continuity; Tweeter is witness; Krillion is plural memory. This describes the protagonist, not the reader."
    elseif c == "mainframe" then
        if v == "connect" then
            return "UNAVAILABLE / Roblox cannot open the native raw TCP uplink. Use mainframe local."
        end
        if v == "local" then
            self.uplink = true
        elseif v == "disconnect" then
            self.uplink = false
        elseif v == "ping" then
            check(self.uplink, "mainframe local first")
            local payload = "DEMON::UPLINK::CHECK"
            check(payload == string.reverse(string.reverse(payload)), "echo mismatch")
            return "SIM / LOCAL ECHO VERIFIED / client memory only / no network RTT"
        else
            check(v == "status", "mainframe local|ping|disconnect")
        end
        return "SIM / UPLINK " .. (self.uplink and "LOCAL" or "OFFLINE")
    elseif c == "egg" then
        if v == "start" then
            check(not self.eggAt, "Already holding an egg")
            check(self.souls >= 800, "Egg costs 800 souls")
            self.souls -= 800
            self.eggAt = now
        elseif v == "hatch" then
            check(self.eggAt and not self.hatchAt, "Need an incubating egg with no active hatch")
            self.hatchAt = now
            return "SIM / HATCH CHANNEL / 2 seconds", "pulse"
        else
            check(v == "status", "egg start|status|hatch")
        end
        local out = {
            string.format("SIM / GOLDEN GOOSE / wallet %d / hatched %d", self.souls, self.hatched),
            self.eggAt
                    and string.format(
                        "INCUBATING / %d souls / -15%% damage / +1m sprint / +1 OOC regen",
                        400 + math.floor((now - self.eggAt) * 80 / 60)
                    )
                or "EMPTY NEST / egg start / 800 souls",
        }
        for i, total in self.buffTotals do
            table.insert(out, buffs[i] .. " +" .. total .. " / fictional permanent buff")
        end
        return table.concat(out, "\n")
    elseif c == "ritual" then
        return string.format(
            "SIM / CURSED APPLE / %s\nCycle %d / phase %d%% / fictional veil pressure %d%%",
            phases[math.floor(self:age() / 20) % 6 + 1],
            math.floor(self:age() / 120) + 1,
            math.floor(self:age() % 20) * 5,
            40 + math.floor(self:age() * 7) % 57
        )
    elseif c == "bido" then
        check(not a[2] or v == "status", "bido [status]")
        if not a[2] then
            self.bidoAt = now
        end
        return "SIM / LITTLE BIDO / "
            .. (self.bidoAt and now - self.bidoAt < 3 and "ACTIVE / 3s decoy shuffle" or "IDLE")
    elseif c == "bucket" then
        local name = a[3]
        if v == "put" or v == "get" or v == "rm" then
            check(
                name
                    and #name <= 128
                    and name ~= ""
                    and not name:find("..", 1, true)
                    and not name:find("[/\\%c]"),
                "Invalid object name"
            )
        end
        if v == "put" then
            check(a[4], "bucket put NAME TEXT")
            local body = table.concat(a, " ", 4)
            check(#body <= 4096, "Object exceeds 4096 bytes")
            local count = 0
            for _ in self.bucket do
                count += 1
            end
            check(self.bucket[name] or count < 100, "Bucket full / 100 objects")
            self.bucket[name] = body
            return "SIM / STORED " .. name .. " / memory only"
        elseif v == "get" then
            check(self.bucket[name], "Object not found")
            return "SIM / " .. name .. "\n" .. self.bucket[name]
        elseif v == "rm" then
            check(self.bucket[name], "Object not found")
            self.bucket[name] = nil
            return "SIM / REMOVED " .. name
        elseif v == "sync" then
            self.syncs += 1
            return "SIM / SNOWFLAKE HANDSHAKE / mock sync " .. self.syncs .. " / no network traffic"
        elseif v == "ls" or v == "status" then
            local out = { "SIM / CLUB-PENGUIN / in-memory inventory" }
            for key, body in self.bucket do
                table.insert(out, key .. " / " .. #body .. " B")
            end
            table.sort(out)
            return table.concat(out, "\n")
        end
        error("bucket status|ls|put NAME TEXT|get NAME|rm NAME|sync", 0)
    elseif c == "tailscale" then
        choice(v, { "status", "milk", "routes", "churn" })
        if v == "churn" then
            self.churnAt = now
        end
        return "SIM / DAIRY MESH / "
            .. (self.churnAt and now - self.churnAt < 3 and "CHURNING" or "PASTEURIZED")
            .. "\ncow-node -> mainframe -> club-penguin / 3 imaginary peers / no Tailscale integration"
    elseif c == "expansion" or c == "mcp" then
        if v == "expand" then
            self.bank = true
        elseif v == "allocate" or v == "release" then
            check(self.bank, "expansion expand first")
            local n = number(a[3], 1, 512, true)
            check(
                v == "allocate" and self.used + n <= 512 or v == "release" and self.used >= n,
                "Virtual capacity exceeded"
            )
            self.used += v == "allocate" and n or -n
        else
            check(v == "status", "expansion status|expand|allocate GB|release GB")
        end
        return string.format(
            "SIM / VULCAN Z / %s / %d / 512 GB virtual / %d free",
            self.bank and "ONLINE" or "OFFLINE",
            self.used,
            512 - self.used
        )
    elseif c == "ssd" then
        choice(v, { "status", "roll", "explode" })
        local phase = self:age() % 24
        local risk = 4 + math.min(phase, 24 - phase) * 0.5
        if v == "roll" then
            self.rolls += 1
        end
        local blast = v == "explode" or v == "roll" and math.random() * 100 < risk
        if blast then
            self.detonations += 1
        end
        return string.format(
            "SIM / IBM EBAY 480 GB DISASTER PROP\n%.1f%% invented odds / %d rolls / %d cartoon detonations\nNo device was inspected.",
            risk,
            self.rolls,
            self.detonations
        ),
            blast and "blast" or nil
    elseif c == "rt60" or c == "room" then
        if v == "set" then
            self.room = {
                number(a[3], 0.1, 1000),
                number(a[4], 0.1, 1000),
                number(a[5], 0.1, 1000),
                number(a[6], 0.001, 1),
            }
        else
            check(v == "status", "rt60 set WIDTH LENGTH HEIGHT ABSORPTION")
        end
        self.graph = "room"
        return string.format(
            "MODEL / SABINE RT60 %.3fs\n%.2f x %.2f x %.2fm / absorption %.3f\nIdealized diffuse-field broadband model; no measured audio.",
            self:rt60(),
            table.unpack(self.room)
        )
    elseif c == "graph" then
        self.graph = choice(a[2] or "signal", { "signal", "room", "metrics", "headphones", "next" })
        if self.graph == "next" then
            self.graph = "metrics"
        end
        return "SCOPE / " .. self.graph
    elseif c == "sdvx" then
        if v == "play" then
            self.mode = "sdvx"
            return "SIM / PHANTOM RHYTHM DECK\nD F J K = BT / Q E = FX / arrow keys adjust knobs when input is unfocused / Enter returns to input",
                "rhythm"
        elseif v == "knob" then
            local side = choice(a[3], { "left", "right" })
            self[side] = math.clamp(self[side] + number(a[4], -32768, 32767, true), 0, 100)
        elseif v == "hit" then
            choice(a[3], { "bt-a", "bt-b", "bt-c", "bt-d", "fx-l", "fx-r" })
            self.combo = math.min(1000000, self.combo + 1)
            self.score = math.min(999999999, self.score + 1000)
            return "SIM / CRITICAL / combo " .. self.combo .. " / score " .. self.score, "pulse"
        else
            check(v == "status", "sdvx play|status|knob left/right DELTA|hit BUTTON")
        end
        return string.format(
            "SIM / SDVX / knobs %d / %d / combo %d / score %d\nPretend CRITICAL judgments; no real timing or game integration.",
            self.left,
            self.right,
            self.combo,
            self.score
        )
    elseif c == "coins" then
        if v == "mint" then
            check(self.balance >= self.price, "Insufficient paper dollars")
            self.balance -= self.price
            self.coins += 1
        elseif v == "pump" then
            check(self.price < 100000000, "Cartoon price cap reached")
            self.price = math.min(100000000, math.ceil(self.price * 1.25))
        else
            choice(v, { "status", "generate" })
        end
        return string.format(
            "SIM / $POOPSECDEMONLARP\nPaper $%.2f / %d coins / fake price $%.2f\nFixed $1000 startup fixture. No money or exchange integration.",
            self.balance / 100,
            self.coins,
            self.price / 100
        )
    elseif c == "focus" then
        choice(v, { "status", "calculate" })
        local size = number(a[3] or "20", 1, 1000000, true)
        return string.format(
            "SIM / FORD FOCUS ST / RE5 STORAGE FANTASY\n%d GB imagined game / %d imaginary 512 GB slots\nNo claim of real compatibility.",
            size,
            math.ceil(size / 512)
        )
    elseif c == "hoi4" then
        if v == "list" then
            local out = {}
            for code, name in countries do
                table.insert(out, code .. " / " .. name)
            end
            table.sort(out)
            return "SIM / COUNTRY DECK\n" .. table.concat(out, "\n")
        elseif v == "select" then
            local code = (a[3] or ""):upper()
            code = code == "UK" and "ENG" or code == "USSR" and "SOV" or code
            check(countries[code], "Unknown country; hoi4 list")
            self.country = code
            self.focusAt = nil
            self.completed = 0
        elseif v == "focus" then
            local name = a[3] and table.concat(a, " ", 3) or countries[self.country]
            check(#name <= 64, "Focus name max 64 characters")
            self.focusName = name
            self.focusAt = now
        else
            check(v == "status", "hoi4 list|select CODE|focus [NAME]|status")
        end
        return string.format(
            "SIM / %s / %s\nFocus %s / %d%% / %d complete / fictional 20-second timer",
            self.country,
            countries[self.country],
            self.focusName or "none",
            self.focusAt and math.min(100, math.floor((now - self.focusAt) * 5)) or 0,
            self.completed
        )
    elseif c == "band" then
        if v == "on" or v == "off" then
            self.band = v == "on"
        elseif v == "drummer" then
            check(a[3] == "explode" and self.band, "band on, then band drummer explode")
            return "SIM / DRUMMER CONFETTI / cartoon only / silent", "blast"
        else
            check(v == "status", "band on|off|status|drummer explode")
        end
        return "SIM / SILENT BAND / "
            .. (self.band and "ON" or "OFF")
            .. "\nJOHN / PAUL / GEORGE / RINGO / original cartoon tribute / no playback"
    elseif c == "pedal" then
        if v == "stomp" then
            self.pedal = true
            self.larp = math.min(999, self.larp + 15)
        elseif v == "release" then
            self.pedal = false
        else
            check(v == "status", "pedal stomp|release|status")
        end
        return "SIM / PEDAL "
            .. (self.pedal and "DOWN" or "UP")
            .. " / LARP "
            .. self.larp
            .. "/999",
            v == "stomp" and "blast" or nil
    elseif c == "headphones" then
        if v == "select" then
            self.headphones = choice(a[3], { "ath-r70xa", "hd58x" })
        else
            choice(v, { "status", "list" })
        end
        self.graph = "headphones"
        return "MODEL / "
            .. self.headphones
            .. "\nath-r70xa / hd58x / invented EQ curves / unmeasured / no audio configuration"
    elseif c == "music" then
        choice(v, { "status", "demo", "live" })
        if v == "live" then
            return "UNAVAILABLE / Desktop now-playing integrations stay in the native terminal. music demo is decorative only."
        end
        self.graph = "signal"
        return "SIM / SYNTHETIC SPECTRUM / no audio playback or desktop media connection"
    elseif c == "pwd" then
        return "ROBLOX / LocalPlayer.PlayerGui.NightfallTerminal / private session, no filesystem"
    elseif c == "history" then
        return table.concat(self.history, "\n")
    elseif c == "doctor" then
        return "PORT CAPABILITIES\nLuau local command engine / history / completion / story / simulations / modeled graphs\nNo /bin/sh, filesystem, raw TCP, USB, desktop music or WAV rendering.\nState resets on rejoin. UI is fictional theater, not an anonymity boundary or hardware diagnostic."
    elseif c == "clear" then
        return "", "clear"
    elseif c == "exit" then
        return "SESSION MINIMIZED / reopen with the launcher", "exit"
    end
    error("Unknown command. Roblox has no /bin/sh fallback. Use help or the COMMANDS deck.", 0)
end

function Session:run(line)
    if #line > 4096 then
        return "ERROR / command exceeds 4096 bytes", nil, false
    end
    if not line:find("%S") then
        return "", nil, true
    end
    table.insert(self.history, line)
    if #self.history > 200 then
        table.remove(self.history, 1)
    end
    self.commands += 1
    local ok, text, effect = pcall(function()
        return self:dispatch(words(line))
    end)
    if ok then
        self.success += 1
    else
        self.errors += 1
        text = "ERROR / " .. tostring(text)
    end
    table.insert(self.outcomes, ok)
    if #self.outcomes > 40 then
        table.remove(self.outcomes, 1)
    end
    return text, effect, ok
end
return Session
