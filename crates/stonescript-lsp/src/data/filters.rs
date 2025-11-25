//! Search filters for items and foes

/// Item and foe search filters
pub const ITEM_FILTERS: &[&str] = &[
    // Elements
    "poison", "vigor", "aether", "fire", "air", "ice",
    
    // Star levels (examples - actual usage: *1, *2, etc.)
    "*1", "*2", "*3", "*4", "*5", "*6", "*7", "*8", "*9",
    
    // Enchantment levels (examples - actual usage: +1, +2, etc.)
    "+1", "+2", "+3", "+4", "+5", "+6", "+7", "+8", "+9", "+10",
];

pub const FOE_FILTERS: &[&str] = &[
    // Types
    "arachnid", "serpent", "insect", "machine", "humanoid", "elemental",
    
    // Attributes
    "boss", "phase1", "phase2", "phase3",
    "spawner", "flying", "slow", "ranged", "explode", "swarm",
    "unpushable", "undamageable",
    
    // Immunities/Resistances
    "magic_resist", "magic_vulnerability",
    "immune_to_stun", "immune_to_ranged",
    "immune_to_debuff_damage", "immune_to_physical",
];

pub const ALL_FILTERS: &[&str] = &[
    "poison", "vigor", "aether", "fire", "air", "ice",
    "arachnid", "serpent", "insect", "machine", "humanoid", "elemental",
    "boss", "phase1", "phase2", "phase3",
    "spawner", "flying", "slow", "ranged", "explode", "swarm",
    "unpushable", "undamageable",
    "magic_resist", "magic_vulnerability",
    "immune_to_stun", "immune_to_ranged",
    "immune_to_debuff_damage", "immune_to_physical",
];
