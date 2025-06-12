use clap::Parser;
use crate::Args;

#[test]
fn test_args_parsing() {
    // Test basic argument parsing
    let args = Args::try_parse_from(["chatdelta", "Hello, world!"])
        .expect("Failed to parse basic arguments");
    assert_eq!(args.prompt, "Hello, world!");
    assert!(args.log.is_none());

    // Test with log file path
    let args = Args::try_parse_from(["chatdelta", "Hello, world!", "--log", "interaction.log"])
        .expect("Failed to parse arguments with log path");
    assert_eq!(args.prompt, "Hello, world!");
    assert_eq!(args.log.unwrap().to_str().unwrap(), "interaction.log");

    // Test with empty prompt (should fail)
    assert!(Args::try_parse_from(["chatdelta", ""]).is_err());

    // Test with only program name (should fail)
    assert!(Args::try_parse_from(["chatdelta"]).is_err());
}
