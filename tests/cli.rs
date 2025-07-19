use chatdelta_base::cli::Args;
use clap::Parser;

#[test]
fn test_args_parsing() {
    let args = Args::parse_from(["chatdelta", "Hello"]);
    assert_eq!(args.prompt.unwrap(), "Hello");
}

#[test]
fn test_args_validate_empty() {
    let args = Args::parse_from(["chatdelta"]);
    assert!(args.validate().is_err());
}
