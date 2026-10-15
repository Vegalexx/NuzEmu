console:log("=== NuzEmu : pont mGBA ===")

if emu == nil or emu:getGameCode() ~= "AGB-BPEF" then
    console:error("Charge Pokémon Émeraude français avant le script.")
    return
end

local client = socket.tcp()
local connectResult, connectError = client:connect("127.0.0.1", 18765)

if connectResult ~= 1 then
    console:error("Connexion impossible : " .. tostring(connectError))
    pcall(function() client:close() end)
    return
end

console:log("Connexion TCP établie.")

local PARTY_BASE = 0x020244EC
local PARTY_COUNT = 0x020244E9

local ORDERS = {
    "GAEM", "GAME", "GEAM", "GEMA", "GMAE", "GMEA",
    "AGEM", "AGME", "AEGM", "AEMG", "AMGE", "AMEG",
    "EGAM", "EGMA", "EAGM", "EAMG", "EMGA", "EMAG",
    "MGAE", "MGEA", "MAGE", "MAEG", "MEGA", "MEAG"
}

local SPECIES = {
    [283] = { name = "Gobou", dex = 258 }
}

local function readPokemon(slot)
    local base = PARTY_BASE + slot * 100
    local pid = emu:read32(base)
    local otid = emu:read32(base + 4)
    local key = (pid ~ otid) & 0xFFFFFFFF
    local order = ORDERS[(pid % 24) + 1]

    local words = {}
    for i = 0, 11 do
        words[i] =
            (emu:read32(base + 0x20 + i * 4) ~ key)
            & 0xFFFFFFFF
    end

    local growth = string.find(order, "G", 1, true) - 1
    local species = words[growth * 3] & 0xFFFF

    local checksum = 0
    for i = 0, 11 do
        checksum = (checksum + (words[i] & 0xFFFF)) & 0xFFFF
        checksum = (checksum + ((words[i] >> 16) & 0xFFFF)) & 0xFFFF
    end

    local stored = emu:read16(base + 0x1C)
    local known = SPECIES[species]
    local name = known and known.name or ("Espece interne " .. species)
    local dex = known and tostring(known.dex) or "null"
    local valid = checksum == stored and "true" or "false"

    return string.format(
        '{"slot":%d,"name":"%s","speciesInternal":%d,' ..
        '"nationalDex":%s,"level":%d,"hp":%d,"maxHp":%d,' ..
        '"checksumValid":%s}',
        slot + 1, name, species, dex,
        emu:read8(base + 0x54),
        emu:read16(base + 0x56),
        emu:read16(base + 0x58),
        valid
    )
end

local function makeSnapshot()
    local count = emu:read8(PARTY_COUNT)

    if count > 6 then
        error("Compteur d'equipe invalide : " .. count)
    end

    local party = {}
    for slot = 0, count - 1 do
        party[#party + 1] = readPokemon(slot)
    end

    return string.format(
        '{"gameTitle":"%s","gameCode":"%s",' ..
        '"partyCount":%d,"party":[%s]}',
        emu:getGameTitle(), emu:getGameCode(),
        count, table.concat(party, ",")
    )
end

local frameCount = 0
local lastSnapshot = ""

callbacks:add("frame", function()
    frameCount = frameCount + 1

    if frameCount % 30 ~= 0 then
        return
    end

    local ok, snapshot = pcall(makeSnapshot)
    if not ok then
        console:error("Lecture impossible : " .. tostring(snapshot))
        return
    end

    if snapshot == lastSnapshot then
        return
    end

    local sent, sendError = client:send(snapshot .. "\n")

    if sent then
        lastSnapshot = snapshot
        console:log("Equipe envoyee a NuzEmu.")
    else
        console:error("Envoi impossible : " .. tostring(sendError))
    end
end)

console:log("Pont actif. Lecture en direct demarree.")
