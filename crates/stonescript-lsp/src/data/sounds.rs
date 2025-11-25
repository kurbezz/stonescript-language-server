//! Sound effect IDs from StoneScript manual Appendix B

pub const SOUND_EFFECTS: &[&str] = &[
    // UI
    "buy", "sell", "craft", "equip", "activate",
    "click", "hover", "error",
    
    // Combat
    "hit", "slash", "bash", "stab", "shoot",
    "block", "dodge", "parry", "critical",
    "miss", "armor_break",
    
    // Magic
    "cast", "fireball", "lightning", "ice_blast",
    "heal", "buff", "debuff", "poison",
    
    // Items
    "potion_drink", "shield_equip", "weapon_equip",
    "treasure_open", "coin_pickup",
    
    // Environment
    "footstep", "door_open", "door_close",
    "chest_open", "lever_pull",
    
    // Enemies
    "enemy_spawn", "enemy_death", "boss_roar",
    "spider_attack", "skeleton_rattle",
    
    // Player
    "level_up", "death", "respawn",
    "low_health", "mp_low",
    
    // Common game sounds
    "success", "failure", "warning",
    "notification", "achievement",
];

// Note: Manual lists ~500 sounds. This is a representative subset.
// Full list can be added from Appendix B if needed.
