console:log("=== NuzEmu / mGBA diagnostic ===")

if emu == nil then
    console:error("Aucun jeu charge dans mGBA.")
else
    console:log("Titre : " .. emu:getGameTitle())
    console:log("Code jeu : " .. emu:getGameCode())
    console:log("Taille ROM : " .. tostring(emu:romSize()))

    if emu.memory ~= nil then
        console:log("Domaines memoire disponibles :")

        for name, domain in pairs(emu.memory) do
            console:log(
                tostring(name)
                .. " | "
                .. domain:name()
                .. " | taille="
                .. tostring(domain:size())
            )
        end
    else
        console:warn("Domaines memoire indisponibles.")
    end
end

console:log("=== Fin du diagnostic ===")
