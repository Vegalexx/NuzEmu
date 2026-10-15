console:log("=== NuzEmu : lecture du premier Pokemon ===")

local base = 0x020244EC

local orders = {
    "GAEM", "GAME", "GEAM", "GEMA", "GMAE", "GMEA",
    "AGEM", "AGME", "AEGM", "AEMG", "AMGE", "AMEG",
    "EGAM", "EGMA", "EAGM", "EAMG", "EMGA", "EMAG",
    "MGAE", "MGEA", "MAGE", "MAEG", "MEGA", "MEAG"
}

local function read32(address)
    return emu:read32(address)
end

local ok, err = pcall(function()
    local count = emu:read8(0x020244E9)
    console:log("Nombre de Pokemon : " .. count)

    if count < 1 or count > 6 then
        console:error("Compteur d'equipe inattendu.")
        return
    end

    local personality = read32(base)
    local trainerId = read32(base + 4)
    local key = (personality ~ trainerId) & 0xFFFFFFFF

    local orderIndex = (personality % 24) + 1
    local order = orders[orderIndex]
    local growthIndex = string.find(order, "G", 1, true)

    local encryptedGrowthWord =
        read32(base + 32 + (growthIndex - 1) * 12)

    local speciesId = (encryptedGrowthWord ~ key) & 0xFFFF
    local level = emu:read8(base + 84)

    console:log(string.format(
        "Personnalite : %08X", personality
    ))
    console:log("Ordre des donnees : " .. order)
    console:log("ID espece decode : " .. speciesId)
    console:log("Niveau : " .. level)
end)

if not ok then
    console:error("Erreur de lecture : " .. tostring(err))
end

console:log("=== Fin de lecture ===")