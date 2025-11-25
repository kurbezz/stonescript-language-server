//! Location identifiers for StoneScript

/// Location identifiers used in the game
pub const LOCATIONS: &[&str] = &[
    "rocky",    // Rocky Plateau
    "cave",     // Caves of Fear
    "deadwood", // Deadwood Canyon
    "fungus",   // Fungus Grotto
    "icy",      // Icy Ridge
    "temple",   // Temple of the Body
    "desert",   // Sandstone Chasm
    "halls",    // Haunted Halls
    "tree",     // Haunted Tree
    "nagaraja", // Nagaraja Temple
    "bronze",   // Bronze Guardian
    "titanium", // Titanium Guardian
    "obsidian", // Obsidian Guardian
    "arcane",   // Arcane Golem
];

/// Location full names for documentation
pub const LOCATION_NAMES: &[(&str, &str)] = &[
    ("rocky", "Rocky Plateau"),
    ("cave", "Caves of Fear"),
    ("deadwood", "Deadwood Canyon"),
    ("fungus", "Fungus Grotto"),
    ("icy", "Icy Ridge"),
    ("temple", "Temple of the Body"),
    ("desert", "Sandstone Chasm"),
    ("halls", "Haunted Halls"),
    ("tree", "Haunted Tree"),
    ("nagaraja", "Nagaraja Temple"),
    ("bronze", "Bronze Guardian"),
    ("titanium", "Titanium Guardian"),
    ("obsidian", "Obsidian Guardian"),
    ("arcane", "Arcane Golem"),
];

/// Get full name for a location ID
pub fn get_location_name(id: &str) -> Option<&'static str> {
    LOCATION_NAMES
        .iter()
        .find(|(loc_id, _)| *loc_id == id)
        .map(|(_, name)| *name)
}
