fn main() {
    let mut lexer = nere_internal::lexer::Lexer::from(("test".to_string(), "1".to_string()));
    let tokens = lexer.scan_tokens();

    for token in tokens.iter() {
        println!("{token}");
    }
}
