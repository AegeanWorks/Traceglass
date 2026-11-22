use std::io::Cursor;
use traceglass::input::{read_trace, InputError};

#[test]
fn reads_bounded_utf8_trace() {
    let events = read_trace(Cursor::new(b"start\tready\n"), 32).unwrap();
    assert_eq!(events[0].name, "start");
    assert_eq!(events[0].value, "ready");
}

#[test]
fn rejects_oversized_invalid_utf8_and_bad_records() {
    assert!(matches!(
        read_trace(Cursor::new(b"a\t123"), 4),
        Err(InputError::TooLarge { limit: 4 })
    ));
    assert!(matches!(
        read_trace(Cursor::new([0xff]), 4),
        Err(InputError::InvalidUtf8)
    ));
    assert!(matches!(
        read_trace(Cursor::new(b"invalid"), 16),
        Err(InputError::InvalidTrace(_))
    ));
}
