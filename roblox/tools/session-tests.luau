local Session = require("../src/shared/Session")
local catalog = require("../src/shared/Catalog")
local story = require("../src/shared/Story")
local now = 0
local function fresh()
    return Session.new(catalog, story, function()
        return now
    end)
end
local checks = 0
local function expect(value, message)
    checks += 1
    assert(value, message)
end
local function run(s, command, expected)
    local text, effect, ok = s:run(command)
    expect(ok == (expected ~= false), command .. " -> " .. text)
    return text, effect
end

local s = fresh()
run(s, "expansion allocate 1", false)
run(s, "expansion expand")
run(s, "expansion allocate 512")
run(s, "expansion allocate 1", false)
expect(s.used == 512, "Over-capacity mutation")
run(s, "expansion release 513", false)
run(s, "expansion release 512")
expect(s.used == 0, "Release bank")
run(s, 'bucket put "signal note.txt" "hello world"')
expect(
    run(s, 'bucket get "signal note.txt"'):find("hello world", 1, true),
    "Quoted object roundtrip"
)
run(s, "bucket put ../escape nope", false)
run(s, "bucket rm missing", false)
run(s, 'bucket put name "unterminated', false)
run(s, "rt60 set 5 4 3 0.2")
expect(math.abs(s:rt60() - 0.161 * 60 / (94 * 0.2)) < 1e-9, "Sabine reference")
run(s, "rt60 set 6 4 3 0", false)
expect(s.room[1] == 5, "Room model commits only valid inputs")
run(s, "coins generate")
run(s, "coins mint")
run(s, "coins generate")
expect(s.balance == 99900 and s.coins == 1, "Generate must not refill ledger")
run(s, "sdvx knob left -100")
expect(s.left == 0, "Knob clamps")
run(s, "sdvx hit invalid", false)
expect(s.score == 0, "Invalid hit adds no points")
run(s, "sdvx hit bt-a")
expect(s.score == 1000, "Valid hit")
run(s, "egg hatch", false)
run(s, "egg start")
run(s, "egg start", false)
now = 61
run(s, "egg hatch")
run(s, "egg hatch", false)
now = 100
expect(#s:tick() == 1, "Late poll emits one hatch")
expect(s.souls == 2084 and s.hatched == 1, "Payout fixed to channel completion")
expect(#s:tick() == 0, "Hatch must not repeat")
run(s, "hoi4 focus TEST")
now = 121
expect(#s:tick() == 1 and s.completed == 1, "Focus completes")
expect(#s:tick() == 0, "Focus must not repeat")
for _, command in
    {
        "shell echo hello",
        "mainframe connect 127.0.0.1:8000",
        "dac serial",
        "soundtrack render output.wav",
        "music live",
    }
do
    expect(run(s, command):find("UNAVAILABLE", 1, true), "Desktop boundary: " .. command)
end
run(s, "ls -lah", false)
expect(run(s, "matrix 3"):match("\n([^\n]+)"):len() == 48, "Fixed signal width")
for _, route in { "1", "2" } do
    local n = fresh()
    expect(#n.pages == 12, "All original story pages copied")
    run(n, "novel start")
    for _ = 1, 3 do
        run(n, "novel next")
    end
    run(n, "novel next", false)
    run(n, "novel choose " .. route)
    for _ = 1, 2 do
        run(n, "novel next")
    end
    run(n, "novel choose " .. route)
    for _ = 1, 3 do
        run(n, "novel next")
    end
    expect(n.scene == (route == "1" and 11 or 12), "Story ending route")
    run(n, "novel next", false)
end
local demo = fresh()
run(demo, "opsecular terminal", false)
run(demo, "opsecular boot")
run(demo, "opsecular recycle put story.sav")
expect(run(demo, "opsecular recycle ls"):find("story.sav", 1, true), "Recycle inventory")
run(demo, "opsecular recycle restore story.sav")
run(demo, "opsecular recycle restore story.sav", false)
run(demo, "opsecular operator krillion")
expect(run(demo, "opsecular spirit 10000"):find("120120", 1, true), "Invented support formula")
run(demo, "opsecular spirit 10001", false)
expect(demo.opsecular.spirit == 10000, "Invalid spirit preserves state")
run(demo, "yoggies use 1")
run(demo, "yoggies use 1", false)
run(demo, "yoggies refill")
expect(demo.yoggies[1], "Berry refill")
for _, item in catalog do
    if item[1] == "novel choose 1" then
        while demo.scene < 4 do
            run(demo, "novel next")
        end
    end
    run(demo, item[1])
end
for _ = 1, 240 do
    run(demo, "status")
end
expect(#demo.history == 200 and #demo.outcomes == 40, "Session retention bounds")
print(string.format("NIGHTFALL / %d session checks passed", checks))
