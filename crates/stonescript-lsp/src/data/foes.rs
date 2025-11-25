//! Foe identifiers for StoneScript

/// Foe identifiers used in the game
pub const FOES: &[&str] = &[
    // Rocky Plateau
    "poena",
    "pallas",
    // Caves of Fear
    "bolesh",
    "alpha_beetle",
    "beetle",
    // Deadwood Canyon
    "xyloalgia",
    "wooden_chest",
    "angry_chest",
    // Fungus Grotto
    "agaricus",
    "fungling",
    // Icy Ridge
    "aurora",
    "frostling",
    // Temple of the Body
    "bodyguard",
    // Sandstone Chasm
    "fire_phantom",
    "sandworm",
    // Haunted Halls
    "kerfuffle",
    "haunted_armor",
    "cursed_book",
    // Haunted Tree
    "nagaraja",
    "tree",
    // Nagaraja Temple
    "nagaraja_head",
    // Bosses
    "bronze_guardian",
    "titanium_guardian",
    "obsidian_guardian",
    "arcane_golem",
    // Other
    "treasure",
    "chest",
];

/// Foe full names for documentation
pub const FOE_NAMES: &[(&str, &str)] = &[
    ("poena", "Poena"),
    ("pallas", "Pallas"),
    ("bolesh", "Bolesh"),
    ("alpha_beetle", "Alpha Beetle"),
    ("beetle", "Beetle"),
    ("xyloalgia", "Xyloalgia"),
    ("wooden_chest", "Wooden Chest"),
    ("angry_chest", "Angry Chest"),
    ("agaricus", "Agaricus"),
    ("fungling", "Fungling"),
    ("aurora", "Aurora"),
    ("frostling", "Frostling"),
    ("bodyguard", "Bodyguard"),
    ("fire_phantom", "Fire Phantom"),
    ("sandworm", "Sandworm"),
    ("kerfuffle", "Kerfuffle"),
    ("haunted_armor", "Haunted Armor"),
    ("cursed_book", "Cursed Book"),
    ("nagaraja", "Nagaraja"),
    ("tree", "Tree"),
    ("nagaraja_head", "Nagaraja Head"),
    ("bronze_guardian", "Bronze Guardian"),
    ("titanium_guardian", "Titanium Guardian"),
    ("obsidian_guardian", "Obsidian Guardian"),
    ("arcane_golem", "Arcane Golem"),
    ("treasure", "Treasure"),
    ("chest", "Chest"),
];

/// Get full name for a foe ID
pub fn get_foe_name(id: &str) -> Option<&'static str> {
    FOE_NAMES
        .iter()
        .find(|(foe_id, _)| *foe_id == id)
        .map(|(_, name)| *name)
}
