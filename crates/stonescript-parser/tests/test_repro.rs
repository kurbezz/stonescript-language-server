
#[test]
fn test_repro_boss_bar() {
    let source = r#"
func BossStats()
  BossHPLost = (BossTHP - BossHP) //health lost
  ?BossHPLost >= BossTHP
    BossHPLost = BossTHP
"#;
    let result = stonescript_parser::parse_source(source);
    if let Err(e) = result {
        panic!("Parse error: {}", e);
    }
}
