use rustikv::bffp::{Command, encode_command};

// Frame layout (from rustikv bffp.rs): total_len(4) | op(1) | flags(1) | key_len(2)
// | key | value_len(4) | value | [ttl(4) if flag&1]. Write op code = 2, HAS_TTL flag = 1.
#[test]
fn write_with_ttl_byte_layout_is_stable() {
    let bytes = encode_command(Command::Write("k".into(), "v".into(), Some(60)));
    // total_len = op(1)+flags(1)+key_len(2)+value_len(4)+key(1)+value(1)+ttl(4) = 14
    assert_eq!(&bytes[0..4], &14u32.to_be_bytes());
    assert_eq!(bytes[4], 2, "Write op code");
    assert_eq!(bytes[5], 1, "HAS_TTL flag");
    assert_eq!(&bytes[6..8], &1u16.to_be_bytes(), "key_len");
    assert_eq!(bytes[8], b'k');
    assert_eq!(&bytes[9..13], &1u32.to_be_bytes(), "value_len");
    assert_eq!(bytes[13], b'v');
    assert_eq!(&bytes[14..18], &60u32.to_be_bytes(), "ttl seconds");
}
